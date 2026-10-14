use class_hierarchy::{
    classifiers::{
        Attribute, AttributeValue, Class, ClassValue, ClassifierKind, ClassifierKindChildValue,
        ClassifierKindValue, ClassifierValue, ModelElement, ModelElementKind,
        ModelElementKindChildValue, ModelElementKindValue, ModelElementValue, Package,
        PackageValue, Reference, StructuralFeature, StructuralFeatureKind,
        StructuralFeatureKindChildValue, StructuralFeatureKindValue, StructuralFeatureValue,
    },
    package::{ClassHierarchy, ClassHierarchyLog, ClassHierarchyValue},
    read_as_ecore::ReadAsEcore,
    references::{ClassId, ClassSupertypesEdge, Instance, ReferenceId, ReferenceTypEdge, Refs},
};
use moirai_crdt::{
    counter::resettable_counter::Counter,
    flag::ew_flag::EWFlag,
    list::{eg_walker::List, nested_list::NestedList},
    utils::membership::{triplet_log, twins_log},
};
use moirai_macros::typed_graph::Arc;
use moirai_protocol::{crdt::query::Read, replica::IsReplica, state::log::IsLog};

/// Test 1: Concurrent assignments of distinct values to the same feature.
#[test]
fn concurrent_assigments_distinct_values_same_feature() {
    let (mut replica_a, mut replica_b) = twins_log::<ClassHierarchyLog>();

    //* INITIAL STATE *//

    let event_a_1 = replica_a.send(new_class(0)).unwrap();
    replica_b.receive(event_a_1);

    let event_a_2 = replica_a.send(new_attribute(0, 0)).unwrap();
    replica_b.receive(event_a_2);

    //* CONCURRENCY */
    let event_a_3 = replica_a
        .send(update_attribute(
            0,
            0,
            Attribute::StructuralFeatureSuper(StructuralFeature::IsOrdered(EWFlag::Enable)),
        ))
        .unwrap();
    let event_b_1 = replica_b
        .send(update_attribute(
            0,
            0,
            Attribute::StructuralFeatureSuper(StructuralFeature::IsOrdered(EWFlag::Enable)),
        ))
        .unwrap();

    //* CONVERGENCE *//

    replica_a.receive(event_b_1);
    replica_b.receive(event_a_3);

    println!("Replica A state: {:#?}", replica_a.query(&Read::new()));

    let expected_value = package_value(vec![class_value(vec![StructuralFeatureKindValue::Value(
        Box::new(StructuralFeatureKindChildValue::Attribute(AttributeValue {
            structural_feature_super: StructuralFeatureValue {
                model_element_super: ModelElementValue {
                    name: String::from(""),
                },
                lower: 0,
                upper: 0,
                is_ordered: true,
                is_unique: false,
            },
        })),
    )])]);

    assert_eq!(replica_a.query(&Read::new()).package, expected_value);
    assert_eq!(replica_b.query(&Read::new()).package, expected_value);

    //* CHECKING THE XMI */
    let xmi = String::from_utf8(replica_a.query(&ReadAsEcore::new())).unwrap();
    assert!(xmi.contains(
        "features isOrdered=\"true\" isUnique=\"false\" lower=\"0\" name=\"\" upper=\"0\""
    ));
    assert!(xmi.contains("xsi:type=\"class_hierarchy:Attribute\""));
    assert!(xmi.contains("xsi:type=\"class_hierarchy:Class\""));
}

/// Test 2: Concurrent assignment and removal of a feature value.
#[test]
fn concurrent_assignment_removal_feature_value() {
    let (mut replica_a, mut replica_b) = twins_log::<ClassHierarchyLog>();

    //* INITIAL STATE *//

    let event_a_1 = replica_a.send(new_class(0)).unwrap();
    replica_b.receive(event_a_1);

    let event_a_2 = replica_a.send(new_attribute(0, 0)).unwrap();
    replica_b.receive(event_a_2);

    //* CONCURRENCY */
    let event_a_3 = replica_a
        .send(update_attribute(
            0,
            0,
            Attribute::StructuralFeatureSuper(StructuralFeature::ModelElementSuper(
                ModelElement::Name(List::Insert {
                    content: 'a',
                    pos: 0,
                }),
            )),
        ))
        .unwrap();

    let event_b_1 = replica_b.send(delete_attribute(0, 0)).unwrap();

    //* CONVERGENCE *//

    replica_a.receive(event_b_1);
    replica_b.receive(event_a_3);

    let expected_value = package_value(vec![class_value(vec![StructuralFeatureKindValue::Value(
        Box::new(StructuralFeatureKindChildValue::Attribute(AttributeValue {
            structural_feature_super: StructuralFeatureValue {
                model_element_super: ModelElementValue {
                    name: String::from("a"),
                },
                lower: 0,
                upper: 0,
                is_ordered: false,
                is_unique: false,
            },
        })),
    )])]);

    assert_eq!(replica_a.query(&Read::new()).package, expected_value);
    assert_eq!(replica_b.query(&Read::new()).package, expected_value);

    //* CHECKING THE XMI */
    let xmi = String::from_utf8(replica_a.query(&ReadAsEcore::new())).unwrap();
    assert!(xmi.contains(
        "features isOrdered=\"false\" isUnique=\"false\" lower=\"0\" name=\"a\" upper=\"0\""
    ));
    assert!(xmi.contains("xsi:type=\"class_hierarchy:Attribute\""));
    assert!(xmi.contains("xsi:type=\"class_hierarchy:Class\""));
}

/// Test 3: Concurrent deletion of a model element and update of one of its features.
#[test]
fn concurrent_deletion_model_element_update_features() {
    let (mut replica_a, mut replica_b) = twins_log::<ClassHierarchyLog>();

    //* INITIAL STATE *//

    let event_a_1 = replica_a.send(new_class(0)).unwrap();
    replica_b.receive(event_a_1);

    let event_a_2 = replica_a.send(new_attribute(0, 0)).unwrap();
    replica_b.receive(event_a_2);

    //* CONCURRENCY */
    let event_a_3 = replica_a
        .send(update_attribute(
            0,
            0,
            Attribute::StructuralFeatureSuper(StructuralFeature::Upper(Counter::Inc(1))),
        ))
        .unwrap();

    let event_b_1 = replica_b.send(delete_class(0)).unwrap();

    //* CONVERGENCE *//

    replica_a.receive(event_b_1);
    replica_b.receive(event_a_3);

    let expected_value = package_value(vec![class_value(vec![StructuralFeatureKindValue::Value(
        Box::new(StructuralFeatureKindChildValue::Attribute(AttributeValue {
            structural_feature_super: StructuralFeatureValue {
                model_element_super: ModelElementValue {
                    name: String::new(),
                },
                lower: 0,
                upper: 1,
                is_ordered: false,
                is_unique: false,
            },
        })),
    )])]);

    assert_eq!(replica_a.query(&Read::new()).package, expected_value);
    assert_eq!(replica_b.query(&Read::new()).package, expected_value);

    //* CHECKING THE XMI */
    let xmi = String::from_utf8(replica_a.query(&ReadAsEcore::new())).unwrap();

    println!("XMI: {}", xmi);

    assert!(xmi.contains(
        "features isOrdered=\"false\" isUnique=\"false\" lower=\"0\" name=\"\" upper=\"1\""
    ));
    assert!(xmi.contains("xsi:type=\"class_hierarchy:Attribute\""));
    assert!(xmi.contains("xsi:type=\"class_hierarchy:Class\""));
}

/// Test 4: Concurrent addition and removal of a reference.
#[test]
fn concurrent_addition_removal_reference() {
    let (mut replica_a, mut replica_b) = twins_log::<ClassHierarchyLog>();

    //* INITIAL STATE *//

    let event_a_1 = replica_a.send(new_class(0)).unwrap();
    replica_b.receive(event_a_1);

    let event_a_2 = replica_a.send(new_class(1)).unwrap();
    replica_b.receive(event_a_2);

    let value = replica_a.query(&Read::new());

    let mut class_ids = value.refs.node_weights().filter_map(|n| match &n {
        Instance::ClassId(class_id) => Some(class_id.clone()),
        _ => None,
    });
    let class_id_1 = class_ids.nth(0).unwrap();
    let class_id_2 = class_ids.nth(0).unwrap();

    // The second class is a supertype of the first one
    let event_a_3 = replica_a
        .send(supertype(class_id_1.clone(), class_id_2.clone()))
        .unwrap();
    replica_b.receive(event_a_3);

    //* CONCURRENCY */
    // B removes the supertype reference from the first class to the second class, while A adds it
    let event_a_4 = replica_a
        .send(supertype(class_id_1.clone(), class_id_2.clone()))
        .unwrap();
    let event_b_1 = replica_b
        .send(remove_supertype(class_id_1, class_id_2))
        .unwrap();

    // //* CONVERGENCE *//

    replica_a.receive(event_b_1);
    replica_b.receive(event_a_4);

    assert_eq!(replica_a.query(&Read::new()).refs.node_weights().count(), 2);
    assert_eq!(replica_a.query(&Read::new()).refs.edge_weights().count(), 1);
    assert_eq!(replica_b.query(&Read::new()).refs.node_weights().count(), 2);
    assert_eq!(replica_b.query(&Read::new()).refs.edge_weights().count(), 1);

    // //* CHECKING THE XMI */
    let xmi = String::from_utf8(replica_a.query(&ReadAsEcore::new())).unwrap();
    // The supertype reference should be present in the XMI
    assert!(xmi.contains("supertypes=\"#"));
}

/// Test 5: Addition of a reference to a concurrently deleted model element.
#[test]
fn concurrent_addition_reference_delete_model_element() {
    let (mut replica_a, mut replica_b, mut replica_c) = triplet_log::<ClassHierarchyLog>();

    //* INITIAL STATE *//

    let event_a_1 = replica_a.send(new_class(0)).unwrap();
    replica_b.receive(event_a_1.clone());
    replica_c.receive(event_a_1);

    let event_a_2 = replica_a.send(new_class(1)).unwrap();
    replica_b.receive(event_a_2.clone());
    replica_c.receive(event_a_2);

    let event_a_3 = replica_a.send(new_reference(1, 0)).unwrap();
    replica_b.receive(event_a_3.clone());
    replica_c.receive(event_a_3);

    let value = replica_a.query(&Read::new());
    let reference_id: ReferenceId = value
        .refs
        .node_weights()
        .find_map(|n| match &n {
            Instance::ReferenceId(reference_id) => Some(reference_id.clone()),
            _ => None,
        })
        .unwrap();
    let class_id = value
        .refs
        .node_weights()
        .find_map(|n| match &n {
            Instance::ClassId(class_id) => Some(class_id.clone()),
            _ => None,
        })
        .unwrap();

    //* CONCURRENCY */
    let event_a_4 = replica_a
        .send(reference_to_class(reference_id.clone(), class_id.clone()))
        .unwrap();

    let event_b_1 = replica_b.send(delete_class(0)).unwrap();

    let event_c_1 = replica_c
        .send(update_class(0, Class::IsAbstract(EWFlag::Enable)))
        .unwrap();

    //* CONVERGENCE *//
    replica_a.receive(event_b_1.clone());
    replica_c.receive(event_b_1);
    replica_a.receive(event_c_1.clone());
    replica_b.receive(event_a_4.clone());
    replica_c.receive(event_a_4);
    replica_b.receive(event_c_1);

    let value_a = replica_a.query(&Read::new());
    let value_b = replica_b.query(&Read::new());
    let value_c = replica_c.query(&Read::new());

    fn count_classes(value: &ClassHierarchyValue) -> usize {
        value
            .refs
            .node_weights()
            .filter(|n| matches!(n, Instance::ClassId(_)))
            .count()
    }

    fn count_references(value: &ClassHierarchyValue) -> usize {
        value.refs.edge_weights().count()
    }

    println!("Replica A state: {:#?}", value_a);

    assert_eq!(count_classes(&value_a), 2);
    assert_eq!(count_classes(&value_b), 2);
    assert_eq!(count_classes(&value_c), 2);
    assert_eq!(count_references(&value_a), 1);
    assert_eq!(count_references(&value_b), 1);
    assert_eq!(count_references(&value_c), 1);

    // //* CHECKING THE XMI */
    let xmi = String::from_utf8(replica_a.query(&ReadAsEcore::new())).unwrap();
    println!("XMI: {}", xmi);
    // The supertype reference should be present in the XMI
    assert!(xmi.contains("typ=\"#"));
}

/// Test 6: Concurrent addition of references that together exceed the multiplicities.
#[test]
fn concurrent_addition_references_exceed_multiplicities() {
    let (mut replica_a, mut replica_b) = twins_log::<ClassHierarchyLog>();

    //* INITIAL STATE *//

    let event_a_1 = replica_a.send(new_class(0)).unwrap();
    replica_b.receive(event_a_1);

    let event_a_2 = replica_a.send(new_class(1)).unwrap();
    replica_b.receive(event_a_2);

    let event_a_3 = replica_a.send(new_reference(1, 0)).unwrap();
    replica_b.receive(event_a_3);

    let value = replica_a.query(&Read::new());

    let reference_id: ReferenceId = value
        .refs
        .node_weights()
        .find_map(|n| match &n {
            Instance::ReferenceId(reference_id) => Some(reference_id.clone()),
            _ => None,
        })
        .unwrap();
    let mut class_ids = value.refs.node_weights().filter_map(|n| match &n {
        Instance::ClassId(class_id) => Some(class_id.clone()),
        _ => None,
    });
    let class_1 = class_ids.nth(0).unwrap();
    let class_2 = class_ids.nth(0).unwrap();

    //* CONCURRENCY */
    let event_a_4 = replica_a
        .send(reference_to_class(reference_id.clone(), class_1.clone()))
        .unwrap();

    let event_b_1 = replica_b
        .send(reference_to_class(reference_id.clone(), class_2.clone()))
        .unwrap();

    //* CONVERGENCE */
    replica_a.receive(event_b_1.clone());
    replica_b.receive(event_a_4.clone());

    assert_eq!(replica_a.query(&Read::new()).refs.edge_weights().count(), 1);
    assert_eq!(replica_b.query(&Read::new()).refs.edge_weights().count(), 1);

    //* CHECKING THE XMI */
    let xmi = String::from_utf8(replica_a.query(&ReadAsEcore::new())).unwrap();
    // The reference should be present in the XMI
    assert!(xmi.contains("typ=\"#"));
}

//* HELPER FUNCTIONS */
fn new_class(idx: usize) -> <ClassHierarchyLog as IsLog>::Command {
    ClassHierarchy::Package(Package::Content(NestedList::Insert {
        pos: idx,
        op: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
            Class::New,
        ))),
    }))
}

fn update_class(idx: usize, content: Class) -> <ClassHierarchyLog as IsLog>::Command {
    ClassHierarchy::Package(Package::Content(NestedList::Update {
        pos: idx,
        op: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(content))),
    }))
}

fn delete_class(idx: usize) -> <ClassHierarchyLog as IsLog>::Command {
    ClassHierarchy::Package(Package::Content(NestedList::Delete { pos: idx }))
}

fn new_attribute(class_idx: usize, feature_idx: usize) -> <ClassHierarchyLog as IsLog>::Command {
    ClassHierarchy::Package(Package::Content(NestedList::Update {
        pos: class_idx,
        op: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
            Class::Features(NestedList::Insert {
                pos: feature_idx,
                op: StructuralFeatureKind::Attribute(Attribute::New),
            }),
        ))),
    }))
}

fn update_attribute(
    class_idx: usize,
    feature_idx: usize,
    attribute: Attribute,
) -> <ClassHierarchyLog as IsLog>::Command {
    ClassHierarchy::Package(Package::Content(NestedList::Update {
        pos: class_idx,
        op: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
            Class::Features(NestedList::Update {
                pos: feature_idx,
                op: StructuralFeatureKind::Attribute(attribute),
            }),
        ))),
    }))
}

fn delete_attribute(class_idx: usize, feature_idx: usize) -> <ClassHierarchyLog as IsLog>::Command {
    ClassHierarchy::Package(Package::Content(NestedList::Update {
        pos: class_idx,
        op: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
            Class::Features(NestedList::Delete { pos: feature_idx }),
        ))),
    }))
}

fn new_reference(class_idx: usize, feature_idx: usize) -> <ClassHierarchyLog as IsLog>::Command {
    ClassHierarchy::Package(Package::Content(NestedList::Update {
        pos: class_idx,
        op: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
            Class::Features(NestedList::Insert {
                pos: feature_idx,
                op: StructuralFeatureKind::Reference(Reference::New),
            }),
        ))),
    }))
}

fn reference_to_class(
    source: ReferenceId,
    target: ClassId,
) -> <ClassHierarchyLog as IsLog>::Command {
    ClassHierarchy::AddReference(Refs::ReferenceToClass(Arc {
        source,
        target,
        kind: ReferenceTypEdge,
    }))
}

fn supertype(source: ClassId, target: ClassId) -> <ClassHierarchyLog as IsLog>::Command {
    ClassHierarchy::AddReference(Refs::ClassToClass(Arc {
        source,
        target,
        kind: ClassSupertypesEdge,
    }))
}

fn remove_supertype(source: ClassId, target: ClassId) -> <ClassHierarchyLog as IsLog>::Command {
    ClassHierarchy::RemoveReference(Refs::ClassToClass(Arc {
        source,
        target,
        kind: ClassSupertypesEdge,
    }))
}

fn package_value(content: Vec<Box<ModelElementKindValue>>) -> PackageValue {
    PackageValue {
        model_element_super: ModelElementValue {
            name: String::from(""),
        },
        content,
    }
}

fn class_value(features: Vec<StructuralFeatureKindValue>) -> Box<ModelElementKindValue> {
    Box::new(ModelElementKindValue::Value(Box::new(
        ModelElementKindChildValue::Classifier(ClassifierKindValue::Value(Box::new(
            ClassifierKindChildValue::Class(ClassValue {
                is_abstract: false,
                classifier_super: ClassifierValue {
                    model_element_super: ModelElementValue {
                        name: String::from(""),
                    },
                },
                features,
            }),
        ))),
    )))
}
