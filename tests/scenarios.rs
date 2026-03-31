use class_hierarchy::{
    classifiers::{
        Attribute, Class, Classifier, ClassifierKind, DataType, ModelElement, ModelElementKind,
        Package, StructuralFeature, StructuralFeatureKind,
    },
    package::{ClassHierarchy, ClassHierarchyLog},
    references::{AttributeId, AttributeTypEdge, ClassId, DataTypeId, Refs},
    utils::graph_view::Vf2GraphView,
};
use moirai_crdt::{
    flag::ew_flag::EWFlag,
    list::{eg_walker::List, nested_list::NestedList},
    utils::membership::twins_log,
};
use moirai_macros::typed_graph::Arc;
use moirai_protocol::{
    broadcast::tcsb::Tcsb,
    crdt::query::Read,
    replica::{IsReplica, Replica},
};

#[test]
fn conflicting_ref_type_max() {
    let (mut replica_a, mut replica_b) = twins_log::<ClassHierarchyLog>();

    let a_1 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 0,
                value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                    Class::New,
                ))),
            },
        )))
        .unwrap();
    let a_2 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 1,
                value: Box::new(ModelElementKind::Classifier(ClassifierKind::DataType(
                    DataType::New,
                ))),
            },
        )))
        .unwrap();
    let a_3 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: 0,
                value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                    Class::Features(NestedList::Insert {
                        pos: 0,
                        value: StructuralFeatureKind::Attribute(Attribute::New),
                    }),
                ))),
            },
        )))
        .unwrap();
    replica_b.receive(a_1.clone());
    replica_b.receive(a_2.clone());
    replica_b.receive(a_3.clone());

    let read = replica_b.query(Read::new());

    let attribute_path = read.refs.raw_nodes()[2].weight.vertex_path();
    let class_path = read.refs.raw_nodes()[1].weight.vertex_path();

    let b_1 = replica_b
        .send(ClassHierarchy::AddReference(Refs::AttributeToClass(Arc {
            source: AttributeId(attribute_path.clone()),
            target: ClassId(class_path.clone()),
            kind: AttributeTypEdge,
        })))
        .unwrap();
    let datatype_path = read.refs.raw_nodes()[0].weight.vertex_path();
    let a_4 = replica_a
        .send(ClassHierarchy::AddReference(Refs::AttributeToDataType(
            Arc {
                source: AttributeId(attribute_path.clone()),
                target: DataTypeId(datatype_path.clone()),
                kind: AttributeTypEdge,
            },
        )))
        .unwrap();

    replica_a.receive(b_1.clone());
    replica_b.receive(a_4.clone());

    let state_a = replica_a.query(Read::new());
    let state_b = replica_b.query(Read::new());

    let is_isomorph = vf2::isomorphisms(&Vf2GraphView(&state_a.refs), &Vf2GraphView(&state_b.refs))
        .default_eq()
        .first()
        .is_some();

    assert_eq!(state_a.refs.node_count(), 3);
    assert_eq!(state_a.refs.edge_count(), 1);
    assert_eq!(state_b.refs.node_count(), 3);
    assert_eq!(state_b.refs.edge_count(), 1);
    assert_eq!(state_a.package, state_b.package);
    assert!(is_isomorph);
}

#[test]
fn vertex_cascade_creation_deletion() {
    let mut replica_a = Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::new("a".to_string());

    replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 0,
                value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                    Class::IsAbstract(EWFlag::Enable),
                ))),
            },
        )))
        .unwrap();
    assert_eq!(replica_a.query(Read::new()).refs.node_count(), 1);
    replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::delete(0),
        )))
        .unwrap();

    let state_a = replica_a.query(Read::new());
    assert_eq!(state_a.refs.node_count(), 0);
}

#[test]
fn simple_class_hierarchy() {
    let (mut replica_a, mut replica_b) = twins_log::<ClassHierarchyLog>();

    let a1 = replica_a
        .send(ClassHierarchy::Package(Package::New))
        .unwrap();
    let a2 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 0,
                value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                    Class::ClassifierSuper(Classifier::ModelElementSuper(ModelElement::Name(
                        List::Insert {
                            content: 'Z',
                            pos: 0,
                        },
                    ))),
                ))),
            },
        )))
        .unwrap();
    let a3 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 1,
                value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                    Class::New,
                ))),
            },
        )))
        .unwrap();
    let a4 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: 1,
                value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                    Class::IsAbstract(EWFlag::Enable),
                ))),
            },
        )))
        .unwrap();
    let a5 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: 1,
                value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                    Class::Features(NestedList::Insert {
                        pos: 0,
                        value: StructuralFeatureKind::Attribute(Attribute::New),
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
                value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                    Class::Features(NestedList::Insert {
                        pos: 0,
                        value: StructuralFeatureKind::Attribute(Attribute::StructuralFeatureSuper(
                            StructuralFeature::IsOrdered(EWFlag::Enable),
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

    let state_a = replica_a.query(Read::new());
    let state_b = replica_b.query(Read::new());

    let is_isomorph = vf2::isomorphisms(&Vf2GraphView(&state_a.refs), &Vf2GraphView(&state_b.refs))
        .default_eq()
        .first()
        .is_some();

    assert_eq!(state_a.package, state_b.package);
    assert!(is_isomorph);
}
