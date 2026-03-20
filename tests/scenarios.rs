use class_hierarchy::{
    classifiers::{
        Attribute, Class, Classifier, ClassifierFeat, ModelElement, ModelElementFeat, Package,
        StructuralFeature, StructuralFeatureFeat,
    },
    package::{ClassHierarchy, ClassHierarchyLog},
    references::{Instance, ReferenceManager},
};
use moirai_crdt::{
    flag::ew_flag::EWFlag,
    list::{eg_walker::List, nested_list::NestedList},
    policy::LwwPolicy,
};
use moirai_protocol::{
    broadcast::tcsb::Tcsb,
    crdt::query::Read,
    replica::{IsReplica, Replica},
    state::po_log::VecLog,
};

type ZooReplica = Replica<ClassHierarchyLog, Tcsb<ClassHierarchy>>;
type RefReplica = Replica<VecLog<ReferenceManager<LwwPolicy>>, Tcsb<ReferenceManager<LwwPolicy>>>;

fn zoo_twins() -> (ZooReplica, ZooReplica) {
    (
        Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::new("a".to_string()),
        Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::new("b".to_string()),
    )
}

fn ref_twins() -> (RefReplica, RefReplica) {
    (
        Replica::<VecLog<ReferenceManager<LwwPolicy>>, Tcsb<ReferenceManager<LwwPolicy>>>::new(
            "a".to_string(),
        ),
        Replica::<VecLog<ReferenceManager<LwwPolicy>>, Tcsb<ReferenceManager<LwwPolicy>>>::new(
            "b".to_string(),
        ),
    )
}

fn assert_ref_convergence(replica_a: &RefReplica, replica_b: &RefReplica) {
    assert_eq!(
        replica_a.query(Read::new()).node_count(),
        replica_b.query(Read::new()).node_count()
    );
    assert_eq!(
        replica_a.query(Read::new()).edge_count(),
        replica_b.query(Read::new()).edge_count()
    );
}

#[test]
fn zoo() {
    let (mut replica_a, mut replica_b) = zoo_twins();

    let a1 = replica_a
        .send(ClassHierarchy::Package(Package::New))
        .unwrap();
    let a2 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 0,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::ClassifierFeat(ClassifierFeat::ModelElementFeat(
                        ModelElementFeat::Name(List::Insert {
                            content: 'Z',
                            pos: 0,
                        }),
                    )),
                ))),
            },
        )))
        .unwrap();
    let a3 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 1,
                value: Box::new(ModelElement::Classifier(Classifier::Class(Class::New))),
            },
        )))
        .unwrap();
    let a4 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: 1,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::IsAbstract(EWFlag::Enable),
                ))),
            },
        )))
        .unwrap();
    let a5 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: 1,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::Features(NestedList::Insert {
                        pos: 0,
                        value: StructuralFeature::Attribute(Attribute::New),
                    }),
                ))),
            },
        )))
        .unwrap();
    let a6 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Delete { pos: 1 },
        )))
        .unwrap();
    replica_b.receive(a1);
    replica_b.receive(a2);
    replica_b.receive(a3);
    replica_b.receive(a4);
    replica_b.receive(a5);
    replica_b.receive(a6);

    let b1 = replica_b
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: 0,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::Features(NestedList::Insert {
                        pos: 0,
                        value: StructuralFeature::Attribute(Attribute::StructuralFeatureFeat(
                            StructuralFeatureFeat::IsOrdered(EWFlag::Enable),
                        )),
                    }),
                ))),
            },
        )))
        .unwrap();

    let a7 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Delete { pos: 0 },
        )))
        .unwrap();

    replica_a.receive(b1);
    replica_b.receive(a7);

    assert_eq!(
        replica_a.query(Read::new()).package,
        replica_b.query(Read::new()).package
    );

    println!("{:#?}", replica_a.query(Read::new()).package);
    assert_eq!(replica_a.query(Read::new()).package.content.len(), 1);

    println!("Vertices in the graph after events a1-a6:");
    for vertex in replica_a.query(Read::new()).refs.node_weights() {
        match vertex {
            Instance::AttributeId(id) => println!("{} (AttributeId)", id.0),
            Instance::ReferenceId(id) => println!("{} (ReferenceId)", id.0),
            Instance::ClassId(id) => println!("{} (ClassId)", id.0),
            Instance::DataTypeId(id) => println!("{} (DataTypeId)", id.0),
        }
    }
}
