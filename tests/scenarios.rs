use class_hierarchy::{
    classifiers::{
        Attribute, Class, ClassValue, Classifier, ClassifierChildValue, ClassifierFeat,
        ClassifierValue, DataType, ModelElement, ModelElementChildValue, ModelElementFeat,
        ModelElementValue, Package, PackageValue, Reference, StructuralFeature,
        StructuralFeatureChildValue, StructuralFeatureFeat, StructuralFeatureValue,
    },
    package::{ClassHierarchy, ClassHierarchyLog},
    references::{ClassId, ClassSupertypesEdge, Instance, Ref, ReferenceManager, Refs},
};
use moirai_crdt::{
    counter::resettable_counter::Counter,
    flag::ew_flag::EWFlag,
    list::{eg_walker::List, nested_list::NestedList},
    policy::LwwPolicy,
};
use moirai_macros::typed_graph::Arc;
use moirai_protocol::{
    broadcast::{message::EventMessage, tcsb::Tcsb},
    crdt::query::Read,
    replica::{IsReplica, Replica},
    state::{po_log::VecLog, sink::ObjectPath},
};
use petgraph::visit::EdgeRef;

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

fn insert_top_level_class(
    replica: &mut ZooReplica,
    pos: usize,
    first_name_char: char,
) -> EventMessage<ClassHierarchy> {
    replica
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::ClassifierFeat(ClassifierFeat::ModelElementFeat(
                        ModelElementFeat::Name(List::Insert {
                            content: first_name_char,
                            pos: 0,
                        }),
                    )),
                ))),
            },
        )))
        .expect("top-level class insertion should be enabled")
}

fn insert_top_level_datatype(
    replica: &mut ZooReplica,
    pos: usize,
    first_name_char: char,
) -> EventMessage<ClassHierarchy> {
    replica
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos,
                value: Box::new(ModelElement::Classifier(Classifier::DataType(
                    DataType::ClassifierFeat(ClassifierFeat::ModelElementFeat(
                        ModelElementFeat::Name(List::Insert {
                            content: first_name_char,
                            pos: 0,
                        }),
                    )),
                ))),
            },
        )))
        .expect("top-level datatype insertion should be enabled")
}

fn append_top_level_class_name(
    replica: &mut ZooReplica,
    pos: usize,
    name_pos: usize,
    ch: char,
) -> EventMessage<ClassHierarchy> {
    replica
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::ClassifierFeat(ClassifierFeat::ModelElementFeat(
                        ModelElementFeat::Name(List::Insert {
                            content: ch,
                            pos: name_pos,
                        }),
                    )),
                ))),
            },
        )))
        .expect("class name update should be enabled")
}

fn delete_top_level_element(replica: &mut ZooReplica, pos: usize) -> EventMessage<ClassHierarchy> {
    replica
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Delete { pos },
        )))
        .expect("top-level delete should be enabled")
}

fn insert_attribute_feature(
    replica: &mut ZooReplica,
    class_pos: usize,
    feature_pos: usize,
    name_char: char,
) -> EventMessage<ClassHierarchy> {
    replica
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: class_pos,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::Features(NestedList::Insert {
                        pos: feature_pos,
                        value: StructuralFeature::Attribute(Attribute::StructuralFeatureFeat(
                            StructuralFeatureFeat::ModelElementFeat(ModelElementFeat::Name(
                                List::Insert {
                                    content: name_char,
                                    pos: 0,
                                },
                            )),
                        )),
                    }),
                ))),
            },
        )))
        .expect("attribute insertion should be enabled")
}

fn insert_reference_feature(
    replica: &mut ZooReplica,
    class_pos: usize,
    feature_pos: usize,
    name_char: char,
) -> EventMessage<ClassHierarchy> {
    replica
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: class_pos,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::Features(NestedList::Insert {
                        pos: feature_pos,
                        value: StructuralFeature::Reference(Reference::StructuralFeatureFeat(
                            StructuralFeatureFeat::ModelElementFeat(ModelElementFeat::Name(
                                List::Insert {
                                    content: name_char,
                                    pos: 0,
                                },
                            )),
                        )),
                    }),
                ))),
            },
        )))
        .expect("reference insertion should be enabled")
}

fn update_reference_upper(
    replica: &mut ZooReplica,
    class_pos: usize,
    feature_pos: usize,
    upper_inc: i32,
) -> EventMessage<ClassHierarchy> {
    replica
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: class_pos,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::Features(NestedList::Update {
                        pos: feature_pos,
                        value: StructuralFeature::Reference(Reference::StructuralFeatureFeat(
                            StructuralFeatureFeat::Upper(Counter::Inc(upper_inc)),
                        )),
                    }),
                ))),
            },
        )))
        .expect("reference upper update should be enabled")
}

fn update_reference_lower(
    replica: &mut ZooReplica,
    class_pos: usize,
    feature_pos: usize,
    lower_inc: i32,
) -> EventMessage<ClassHierarchy> {
    replica
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: class_pos,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::Features(NestedList::Update {
                        pos: feature_pos,
                        value: StructuralFeature::Reference(Reference::StructuralFeatureFeat(
                            StructuralFeatureFeat::Lower(Counter::Inc(lower_inc)),
                        )),
                    }),
                ))),
            },
        )))
        .expect("reference lower update should be enabled")
}

fn delete_feature(
    replica: &mut ZooReplica,
    class_pos: usize,
    feature_pos: usize,
) -> EventMessage<ClassHierarchy> {
    replica
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: class_pos,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::Features(NestedList::Delete { pos: feature_pos }),
                ))),
            },
        )))
        .expect("feature delete should be enabled")
}

fn update_is_abstract(
    replica: &mut ZooReplica,
    class_pos: usize,
    op: EWFlag,
) -> EventMessage<ClassHierarchy> {
    replica
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: class_pos,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::IsAbstract(op),
                ))),
            },
        )))
        .expect("is_abstract update should be enabled")
}

fn package_value(replica: &ZooReplica) -> PackageValue {
    replica.query(Read::new()).package
}

fn top_level_len(replica: &ZooReplica) -> usize {
    package_value(replica).content.len()
}

fn top_level_class(replica: &ZooReplica, pos: usize) -> ClassValue {
    match &replica.query(Read::new()).package.content[pos] {
        ModelElementValue::Value(inner) => match inner.as_ref() {
            ModelElementChildValue::Classifier(classifier) => match classifier {
                ClassifierValue::Value(inner) => match inner.as_ref() {
                    ClassifierChildValue::Class(class) => class.clone(),
                    other => panic!("expected a Class at top-level position {pos}, got {other:?}"),
                },
                other => panic!("expected a concrete Classifier value, got {other:?}"),
            },
            other => panic!("expected a Classifier, got {other:?}"),
        },
        other => panic!("expected a concrete ModelElement value, got {other:?}"),
    }
}

fn top_level_class_name(replica: &ZooReplica, pos: usize) -> String {
    top_level_class(replica, pos)
        .classifier_feat
        .model_element_feat
        .name
        .iter()
        .collect()
}

fn feature_len(replica: &ZooReplica, class_pos: usize) -> usize {
    top_level_class(replica, class_pos).features.len()
}

fn reference_feature(
    replica: &ZooReplica,
    class_pos: usize,
    feature_pos: usize,
) -> class_hierarchy::classifiers::ReferenceValue {
    match &top_level_class(replica, class_pos).features[feature_pos] {
        StructuralFeatureValue::Value(inner) => match inner.as_ref() {
            StructuralFeatureChildValue::Reference(reference) => reference.clone(),
            other => panic!("expected a Reference feature, got {other:?}"),
        },
        other => panic!("expected a concrete StructuralFeature value, got {other:?}"),
    }
}

fn reference_upper(replica: &ZooReplica, class_pos: usize, feature_pos: usize) -> i32 {
    reference_feature(replica, class_pos, feature_pos)
        .structural_feature_feat
        .upper
}

fn reference_lower(replica: &ZooReplica, class_pos: usize, feature_pos: usize) -> i32 {
    reference_feature(replica, class_pos, feature_pos)
        .structural_feature_feat
        .lower
}

fn is_abstract(replica: &ZooReplica, class_pos: usize) -> bool {
    top_level_class(replica, class_pos).is_abstract
}

fn assert_model_convergence(replica_a: &ZooReplica, replica_b: &ZooReplica) {
    let a = replica_a.query(Read::new());
    let b = replica_b.query(Read::new());

    assert_eq!(a.package, b.package);
    assert_eq!(a.refs.node_count(), b.refs.node_count());
    assert_eq!(a.refs.edge_count(), b.refs.edge_count());
}

fn class_id(name: &str) -> ClassId {
    ClassId(ObjectPath::new("zoo").map_entry(name.to_string()))
}

fn class_supertypes_ref(source: &str, target: &str) -> Refs {
    Refs::ClassSupertypes(Arc {
        source: class_id(source),
        target: class_id(target),
        kind: ClassSupertypesEdge,
    })
}

fn add_vertex(replica: &mut RefReplica, id: Instance) -> EventMessage<ReferenceManager<LwwPolicy>> {
    replica
        .send(ReferenceManager::AddVertex { id })
        .expect("AddVertex should be enabled")
}

fn remove_vertex(
    replica: &mut RefReplica,
    id: Instance,
) -> EventMessage<ReferenceManager<LwwPolicy>> {
    replica
        .send(ReferenceManager::RemoveVertex { id })
        .expect("RemoveVertex should be enabled")
}

fn add_arc(replica: &mut RefReplica, arc: Refs) -> EventMessage<ReferenceManager<LwwPolicy>> {
    replica
        .send(ReferenceManager::AddArc(arc))
        .expect("AddArc should be enabled")
}

fn ref_graph_edges(replica: &RefReplica) -> Vec<String> {
    let graph = replica.query(Read::new());
    let mut edges = graph
        .edge_references()
        .map(|edge| {
            let source = graph.node_weight(edge.source()).unwrap();
            let target = graph.node_weight(edge.target()).unwrap();
            let kind = match edge.weight() {
                Ref::ClassSupertypes(_) => "ClassSupertypes",
                Ref::StructuralFeatureTyp(_) => "StructuralFeatureTyp",
                Ref::ReferenceOpposite(_) => "ReferenceOpposite",
            };
            format!("{source:?} -> {target:?} ({kind})")
        })
        .collect::<Vec<_>>();
    edges.sort();
    edges
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
    assert_eq!(ref_graph_edges(replica_a), ref_graph_edges(replica_b));
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
            Instance::ClassifierId(id) => println!("{} (ClassifierId)", id.0),
            Instance::StructuralFeatureId(id) => println!("{} (StructuralFeatureId)", id.0),
            Instance::ReferenceId(id) => println!("{} (ReferenceId)", id.0),
            Instance::ClassId(id) => println!("{} (ClassId)", id.0),
        }
    }
}

#[test]
fn zoo_concurrent_insert_animal_and_keeper_classes() {
    let (mut replica_a, mut replica_b) = zoo_twins();

    let event_a = insert_top_level_class(&mut replica_a, 0, 'A');
    let event_b = insert_top_level_class(&mut replica_b, 0, 'K');

    replica_a.receive(event_b);
    replica_b.receive(event_a);

    assert_model_convergence(&replica_a, &replica_b);
    assert_eq!(top_level_len(&replica_a), 2);
}

#[test]
fn zoo_concurrent_insert_class_and_datatype_same_position() {
    let (mut replica_a, mut replica_b) = zoo_twins();

    let event_a = insert_top_level_class(&mut replica_a, 0, 'A');
    let event_b = insert_top_level_datatype(&mut replica_b, 0, 'S');

    replica_a.receive(event_b);
    replica_b.receive(event_a);

    assert_model_convergence(&replica_a, &replica_b);
    assert_eq!(top_level_len(&replica_a), 2);
}

#[test]
fn zoo_concurrent_rename_animal_class() {
    let (mut replica_a, mut replica_b) = zoo_twins();

    let init = insert_top_level_class(&mut replica_a, 0, 'A');
    replica_b.receive(init);

    let event_a = append_top_level_class_name(&mut replica_a, 0, 1, 'n');
    let event_b = append_top_level_class_name(&mut replica_b, 0, 1, 'm');

    replica_a.receive(event_b);
    replica_b.receive(event_a);

    assert_model_convergence(&replica_a, &replica_b);
    assert_eq!(top_level_class_name(&replica_a, 0).chars().count(), 3);
}

/// When one replica renames the Zoo root class while another deletes it concurrently,
/// the nested update should revive the deleted element and preserve convergence.
#[test]
fn zoo_top_level_class_update_delete_revives_element() {
    let (mut replica_a, mut replica_b) = zoo_twins();

    let init = insert_top_level_class(&mut replica_a, 0, 'A');
    replica_b.receive(init);

    let event_b = append_top_level_class_name(&mut replica_b, 0, 1, 'n');
    let event_a = delete_top_level_element(&mut replica_a, 0);

    replica_a.receive(event_b);
    replica_b.receive(event_a);

    assert_model_convergence(&replica_a, &replica_b);
    assert_eq!(top_level_len(&replica_a), 1);
    assert_eq!(top_level_class_name(&replica_a, 0).chars().count(), 1);
}

#[test]
fn zoo_concurrent_add_attribute_and_reference_to_animal() {
    let (mut replica_a, mut replica_b) = zoo_twins();

    let init = insert_top_level_class(&mut replica_a, 0, 'A');
    replica_b.receive(init);

    let event_a = insert_attribute_feature(&mut replica_a, 0, 0, 'a');
    let event_b = insert_reference_feature(&mut replica_b, 0, 0, 'h');

    replica_a.receive(event_b);
    replica_b.receive(event_a);

    assert_model_convergence(&replica_a, &replica_b);
    assert_eq!(feature_len(&replica_a, 0), 2);
}

/// This mirrors the collaborative delete/update pattern from behaviortree on a Zoo association.
/// One replica deletes the `habitat` reference while the other concurrently raises its upper bound.
#[test]
fn zoo_reference_update_delete_revives_feature() {
    let (mut replica_a, mut replica_b) = zoo_twins();

    let init_class = insert_top_level_class(&mut replica_a, 0, 'A');
    replica_b.receive(init_class);
    let init_ref = insert_reference_feature(&mut replica_a, 0, 0, 'h');
    replica_b.receive(init_ref);
    let init_upper = update_reference_upper(&mut replica_a, 0, 0, 1);
    replica_b.receive(init_upper);

    let event_b = update_reference_upper(&mut replica_b, 0, 0, 1);
    let event_a = delete_feature(&mut replica_a, 0, 0);

    replica_a.receive(event_b);
    replica_b.receive(event_a);

    assert_model_convergence(&replica_a, &replica_b);
    assert_eq!(feature_len(&replica_a, 0), 1);
    assert_eq!(reference_upper(&replica_a, 0, 0), 1);
}

#[test]
fn zoo_concurrent_is_abstract_enable_and_disable_is_enable_wins() {
    let (mut replica_a, mut replica_b) = zoo_twins();

    let init = insert_top_level_class(&mut replica_a, 0, 'A');
    replica_b.receive(init);

    let event_a = update_is_abstract(&mut replica_a, 0, EWFlag::Enable);
    let event_b = update_is_abstract(&mut replica_b, 0, EWFlag::Disable);

    replica_a.receive(event_b);
    replica_b.receive(event_a);

    assert_model_convergence(&replica_a, &replica_b);
    assert!(is_abstract(&replica_a, 0));
}

#[test]
fn zoo_concurrent_lower_and_upper_updates_on_reference_merge() {
    let (mut replica_a, mut replica_b) = zoo_twins();

    let init_class = insert_top_level_class(&mut replica_a, 0, 'A');
    replica_b.receive(init_class);
    let init_ref = insert_reference_feature(&mut replica_a, 0, 0, 'k');
    replica_b.receive(init_ref);

    let event_a = update_reference_lower(&mut replica_a, 0, 0, 1);
    let event_b = update_reference_upper(&mut replica_b, 0, 0, 3);

    replica_a.receive(event_b);
    replica_b.receive(event_a);

    assert_model_convergence(&replica_a, &replica_b);
    assert_eq!(reference_lower(&replica_a, 0, 0), 1);
    assert_eq!(reference_upper(&replica_a, 0, 0), 3);
}

#[test]
fn zoo_typed_graph_concurrent_add_same_supertype_arc_is_idempotent() {
    let (mut replica_a, mut replica_b) = ref_twins();
    let animal = Instance::ClassId(class_id("Animal"));
    let mammal = Instance::ClassId(class_id("Mammal"));

    let e1 = add_vertex(&mut replica_a, animal);
    replica_b.receive(e1);
    let e2 = add_vertex(&mut replica_a, mammal);
    replica_b.receive(e2);

    let event_a = add_arc(&mut replica_a, class_supertypes_ref("Mammal", "Animal"));
    let event_b = add_arc(&mut replica_b, class_supertypes_ref("Mammal", "Animal"));

    replica_a.receive(event_b);
    replica_b.receive(event_a);

    assert_eq!(replica_a.query(Read::new()).edge_count(), 1);
    assert_ref_convergence(&replica_a, &replica_b);
}

#[test]
fn zoo_typed_graph_concurrent_remove_target_vertex_and_add_supertype_arc_hides_arc() {
    let (mut replica_a, mut replica_b) = ref_twins();
    let animal = Instance::ClassId(class_id("Animal"));
    let mammal = Instance::ClassId(class_id("Mammal"));

    let e1 = add_vertex(&mut replica_a, animal.clone());
    replica_b.receive(e1);
    let e2 = add_vertex(&mut replica_a, mammal.clone());
    replica_b.receive(e2);

    let event_a = add_arc(&mut replica_a, class_supertypes_ref("Mammal", "Animal"));
    let event_b = remove_vertex(&mut replica_b, animal);

    replica_a.receive(event_b);
    replica_b.receive(event_a);

    assert_eq!(replica_a.query(Read::new()).node_count(), 1);
    assert_eq!(replica_a.query(Read::new()).edge_count(), 0);
    assert_ref_convergence(&replica_a, &replica_b);
}
