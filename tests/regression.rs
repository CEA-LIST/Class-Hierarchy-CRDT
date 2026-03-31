use class_hierarchy::{
    classifiers::{
        Attribute, Class, Classifier, ClassifierKind, ModelElement, ModelElementKind, Package,
        Reference, StructuralFeature, StructuralFeatureKind,
    },
    package::{ClassHierarchy, ClassHierarchyLog, ClassHierarchyValue},
    references::{AttributeId, AttributeTypEdge, ClassId, Instance, Ref, Refs, SchemaViolation},
    utils::graph_view::Vf2GraphView,
};
use moirai_crdt::{
    counter::resettable_counter::Counter,
    flag::ew_flag::EWFlag,
    list::{eg_walker::List, nested_list::NestedList},
};
use moirai_macros::typed_graph::Arc;
use moirai_protocol::{
    broadcast::tcsb::{IsTcsbTest, Tcsb},
    crdt::query::Read,
    replica::{IsReplica, Replica, ReplicaIdx},
    state::sink::ObjectPath,
    utils::translate_ids::TranslateIds,
};
use petgraph::Graph;

/// This test reproduce this execution trace:
/// digraph {
///     0 [ label="[Package(ModelElement(Name(Insert { content: 'i', pos: 0 })))@(0:1)]"]
///     1 [ label="[Package(Content(Insert { pos: 0, value: ClassifierKind(Class(Features(Insert { pos: 0, value: Attribute(StructuralFeature(Lower(Reset))) }))) }))@(1:1)]"]
///     2 [ label="[Package(Content(Delete { pos: 0 }))@(1:2)]"]
///     3 [ label="[AddReference(AttributeToClass(Arc { source: AttributeId(ObjectPath { root: 'class_hierarchy', segments: [Field('package'), Field('content'), ListElement(EventId { idx: ReplicaIdx(1), seq: 1, resolver: 0 => 0, 1 => 1, 2 => 2 }), Variant('classifier'), Variant('class'), Field('features'), ListElement(EventId { idx: ReplicaIdx(1), seq: 1, resolver: 0 => 0, 1 => 1, 2 => 2 }), Variant('attribute')] }), target: ClassId(ObjectPath { root: 'class_hierarchy', segments: [Field('package'), Field('content'), ListElement(EventId { idx: ReplicaIdx(1), seq: 1, resolver: 0 => 0, 1 => 1, 2 => 2 }), Variant('classifier'), Variant('class')] }), kind: AttributeTypEdge }))@(0:2)]"]
///     4 [ label="[Package(ModelElement(Name(Insert { content: 'a', pos: 1 })))@(2:1)]"]
///     5 [ label="[Package(ModelElement(Name(Insert { content: 'P', pos: 2 })))@(2:2)]"]
///     6 [ label="[Package(ModelElement(Name(Insert { content: 'h', pos: 2 })))@(2:3)]"]
///     7 [ label="[Package(Content(Update { pos: 0, value: ClassifierKind(Class(Features(Delete { pos: 0 }))) }))@(0:3)]"]
///     8 [ label="[Package(Content(Update { pos: 0, value: ClassifierKind(Class(Features(Update { pos: 0, value: Attribute(StructuralFeature(IsUnique(Enable))) }))) }))@(2:4)]"]
///     9 [ label="[Package(Content(Insert { pos: 1, value: StructuralFeatureKind(Attribute(StructuralFeature(IsUnique(Clear)))) }))@(0:4)]"]
///     0 -> 1 [ ]  1 -> 2 [ ]  0 -> 2 [ ]  0 -> 3 [ ]  1 -> 3 [ ]  3 -> 4 [ ]  1 -> 4 [ ]  4 -> 5 [ ]  3 -> 5 [ ]  1 -> 5 [ ]  5 -> 6 [ ]  3 -> 6 [ ] 	1 ->	6 [ ]	3 ->	7 [ ]	1 ->	7 [ ]	6 ->	7 [ ]	6 ->	8 [ ]	3 ->	8 [ ]	1 ->	8 [ ]	7 ->	9 [ ]	1 ->	9 [ ]	6 ->	9 [ ]
/// }
#[test]
fn error_case() {
    let mut replica_a = Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::new("a".to_string());
    let mut replica_b = Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::new("b".to_string());
    let mut replica_c = Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::new("c".to_string());

    let e0 = replica_a
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Insert {
                content: 'i',
                pos: 0,
            }),
        )))
        .unwrap();

    replica_b.receive(e0.clone());

    let e1 = replica_b
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 0,
                value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                    Class::Features(NestedList::Insert {
                        pos: 0,
                        value: StructuralFeatureKind::Attribute(Attribute::StructuralFeatureSuper(
                            StructuralFeature::Lower(
                                moirai_crdt::counter::resettable_counter::Counter::Reset,
                            ),
                        )),
                    }),
                ))),
            },
        )))
        .unwrap();

    let e2 = replica_b
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Delete { pos: 0 },
        )))
        .unwrap();

    replica_a.receive(e1.clone());

    let class_path = ObjectPath::new("class_hierarchy")
        .field("package")
        .field("content")
        .list_element(e1.event().id().clone())
        .variant("classifier")
        .variant("class");
    let attribute_path = class_path
        .clone()
        .field("features")
        .list_element(e1.event().id().clone())
        .variant("attribute");

    let e3 = replica_a
        .send(ClassHierarchy::AddReference(Refs::AttributeToClass(Arc {
            source: AttributeId(attribute_path),
            target: ClassId(class_path.clone()),
            kind: AttributeTypEdge,
        })))
        .unwrap();

    replica_c.receive(e0.clone());
    replica_c.receive(e1.clone());
    replica_c.receive(e3.clone());

    let e4 = replica_c
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Insert {
                content: 'a',
                pos: 1,
            }),
        )))
        .unwrap();

    let e5 = replica_c
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Insert {
                content: 'P',
                pos: 2,
            }),
        )))
        .unwrap();

    let e6 = replica_c
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Insert {
                content: 'h',
                pos: 2,
            }),
        )))
        .unwrap();

    replica_a.receive(e6.clone());

    let e7 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: 0,
                value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                    Class::Features(NestedList::Delete { pos: 0 }),
                ))),
            },
        )))
        .unwrap();

    let e8 = replica_c
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: 0,
                value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                    Class::Features(NestedList::Update {
                        pos: 0,
                        value: StructuralFeatureKind::Attribute(Attribute::StructuralFeatureSuper(
                            StructuralFeature::IsUnique(EWFlag::Enable),
                        )),
                    }),
                ))),
            },
        )))
        .unwrap();

    let e9 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 1,
                value: Box::new(ModelElementKind::StructuralFeature(
                    StructuralFeatureKind::Attribute(Attribute::StructuralFeatureSuper(
                        StructuralFeature::IsUnique(EWFlag::Clear),
                    )),
                )),
            },
        )))
        .unwrap();

    for event in [e0.clone(), e3.clone(), e7.clone(), e9.clone()] {
        replica_b.receive(event.clone());
        replica_c.receive(event);
    }
    for event in [e1.clone(), e2.clone()] {
        replica_a.receive(event.clone());
        replica_c.receive(event);
    }
    for event in [e4.clone(), e5.clone(), e6.clone(), e8.clone()] {
        replica_a.receive(event.clone());
        replica_b.receive(event);
    }

    assert_eq!(
        replica_a.query(Read::<ClassHierarchyValue>::new()).package,
        replica_b.query(Read::<ClassHierarchyValue>::new()).package
    );
    assert_eq!(
        replica_b.query(Read::<ClassHierarchyValue>::new()).package,
        replica_c.query(Read::<ClassHierarchyValue>::new()).package
    );

    let a_refs = replica_a.query(Read::<ClassHierarchyValue>::new()).refs;
    let b_refs = replica_b.query(Read::<ClassHierarchyValue>::new()).refs;
    let c_refs = replica_c.query(Read::<ClassHierarchyValue>::new()).refs;
    assert_eq!(a_refs.node_count(), b_refs.node_count());
    assert_eq!(b_refs.node_count(), c_refs.node_count());
    assert_eq!(a_refs.edge_count(), b_refs.edge_count());
    assert_eq!(b_refs.edge_count(), c_refs.edge_count());
}

#[test]
fn error_case_2() {
    let mut replicas = (0..8)
        .map(|i| Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::new(i.to_string()))
        .collect::<Vec<_>>();

    let e0 = replicas[1]
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Insert {
                content: 'X',
                pos: 0,
            }),
        )))
        .unwrap();

    let e1 = replicas[4]
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Insert {
                content: 'c',
                pos: 0,
            }),
        )))
        .unwrap();

    let e2 = replicas[5]
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 0,
                value: Box::new(ModelElementKind::StructuralFeature(
                    StructuralFeatureKind::Reference(Reference::StructuralFeatureSuper(
                        StructuralFeature::IsOrdered(EWFlag::Enable),
                    )),
                )),
            },
        )))
        .unwrap();

    let e3 = replicas[5]
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 1,
                value: Box::new(ModelElementKind::StructuralFeature(
                    StructuralFeatureKind::Reference(Reference::StructuralFeatureSuper(
                        StructuralFeature::Upper(
                            moirai_crdt::counter::resettable_counter::Counter::Dec(239_778),
                        ),
                    )),
                )),
            },
        )))
        .unwrap();

    replicas[7].receive(e2.clone());
    let e4 = replicas[7]
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Insert {
                content: 'p',
                pos: 0,
            }),
        )))
        .unwrap();

    replicas[0].receive(e2.clone());
    let e5 = replicas[0]
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 0,
                value: Box::new(ModelElementKind::Package(Package::Content(
                    NestedList::Insert {
                        pos: 0,
                        value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                            Class::Features(NestedList::Insert {
                                pos: 0,
                                value: StructuralFeatureKind::Attribute(
                                    Attribute::StructuralFeatureSuper(
                                        StructuralFeature::IsOrdered(EWFlag::Disable),
                                    ),
                                ),
                            }),
                        ))),
                    },
                ))),
            },
        )))
        .unwrap();

    let e9 = replicas[5]
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Delete { pos: 1 },
        )))
        .unwrap();

    let e6 = replicas[7]
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Delete { pos: 0 }),
        )))
        .unwrap();

    replicas[7].receive(e5.clone());
    let class_from_e5 = ObjectPath::new("class_hierarchy")
        .field("package")
        .field("content")
        .list_element(e5.event().id().clone())
        .variant("package")
        .field("content")
        .list_element(e5.event().id().clone())
        .variant("classifier")
        .variant("class");
    let ref_from_e2 = ObjectPath::new("class_hierarchy")
        .field("package")
        .field("content")
        .list_element(e2.event().id().clone())
        .variant("structuralfeature")
        .variant("reference");
    let e7 = replicas[7]
        .send(ClassHierarchy::AddReference(Refs::ReferenceToClass(Arc {
            source: class_hierarchy::references::ReferenceId(ref_from_e2.clone()),
            target: ClassId(class_from_e5.clone()),
            kind: class_hierarchy::references::ReferenceTypEdge,
        })))
        .unwrap();

    let e11 = replicas[0]
        .send(ClassHierarchy::AddReference(Refs::ReferenceToClass(Arc {
            source: class_hierarchy::references::ReferenceId(ref_from_e2),
            target: ClassId(class_from_e5),
            kind: class_hierarchy::references::ReferenceTypEdge,
        })))
        .unwrap();

    replicas[3].receive(e2.clone());
    replicas[3].receive(e4.clone());
    replicas[3].receive(e6.clone());
    replicas[3].receive(e5.clone());
    replicas[3].receive(e7.clone());
    let e8 = replicas[3]
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Insert {
                content: 'u',
                pos: 0,
            }),
        )))
        .unwrap();

    replicas[7].receive(e8.clone());
    let e10 = replicas[7]
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Delete { pos: 0 }),
        )))
        .unwrap();

    replicas[1].receive(e2.clone());
    replicas[1].receive(e4.clone());
    replicas[1].receive(e6.clone());
    replicas[1].receive(e5.clone());
    replicas[1].receive(e7.clone());
    replicas[1].receive(e8.clone());
    replicas[1].receive(e10.clone());
    let e12 = replicas[1]
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Delete { pos: 0 },
        )))
        .unwrap();

    let e14 = {
        replicas[5].receive(e12.clone());
        replicas[5]
            .send(ClassHierarchy::Package(Package::ModelElementSuper(
                ModelElement::Name(List::Insert {
                    content: 'B',
                    pos: 0,
                }),
            )))
            .unwrap()
    };

    replicas[3].receive(e10.clone());
    let e19 = replicas[3]
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: 0,
                value: Box::new(ModelElementKind::Package(Package::Content(
                    NestedList::Update {
                        pos: 0,
                        value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                            Class::Features(NestedList::Update {
                                pos: 0,
                                value: StructuralFeatureKind::Attribute(
                                    Attribute::StructuralFeatureSuper(
                                        StructuralFeature::ModelElementSuper(ModelElement::Name(
                                            List::Insert {
                                                content: 'K',
                                                pos: 0,
                                            },
                                        )),
                                    ),
                                ),
                            }),
                        ))),
                    },
                ))),
            },
        )))
        .unwrap();

    let e15 = replicas[0]
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Insert {
                content: 'N',
                pos: 0,
            }),
        )))
        .unwrap();

    let e13 = replicas[1]
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Delete { pos: 0 }),
        )))
        .unwrap();

    for event in [
        e0.clone(),
        e2.clone(),
        e3.clone(),
        e4.clone(),
        e5.clone(),
        e6.clone(),
        e7.clone(),
        e8.clone(),
        e9.clone(),
        e10.clone(),
        e12.clone(),
        e13.clone(),
        e14.clone(),
    ] {
        replicas[2].receive(event);
    }
    let e17 = replicas[2]
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Delete { pos: 0 }),
        )))
        .unwrap();

    let e16 = replicas[0]
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Delete { pos: 0 },
        )))
        .unwrap();

    let e18 = replicas[0]
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Insert {
                content: 'q',
                pos: 0,
            }),
        )))
        .unwrap();

    let all_events = vec![
        e0, e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18, e19,
    ];

    for target in 0..replicas.len() {
        for event in &all_events {
            if event.event().id().origin_id() == target.to_string() {
                continue;
            }
            replicas[target].receive(event.clone());
        }
    }

    let graph = replicas[0].query(Read::new()).refs;
    for replica in &replicas[1..] {
        vf2::isomorphisms(
            &Vf2GraphView(&graph),
            &Vf2GraphView(&replica.query(Read::new()).refs),
        )
        .first()
        .expect("graphs should be isomorphic");
    }
}

/// This test implements this execution trace:
// digraph {
//         0       [label="[Package(Content(Insert { pos: 0, value: ClassifierKind(Class(Features(Insert { pos: 0, value: Attribute(StructuralFeature(Upper(\Inc(130404)))) }))) }))@(6:1)]"];
//         1       [label="[AddReference(AttributeToClass(Arc { source: AttributeId(ObjectPath { root: 'class_hierarchy', segments: [Field('package'), Field('\content'), ListElement(EventId { idx: ReplicaIdx(6), seq: 1, resolver: 0 => 2, 1 => 0, 2 => 1, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => \7 }), Variant('classifier'), Variant('class'), Field('features'), ListElement(EventId { idx: ReplicaIdx(6), seq: 1, resolver: 0 => \2, 1 => 0, 2 => 1, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => 7 }), Variant('attribute')] }), target: ClassId(ObjectPath { root: 'class_\hierarchy', segments: [Field('package'), Field('content'), ListElement(EventId { idx: ReplicaIdx(6), seq: 1, resolver: 0 => 2, 1 => \0, 2 => 1, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => 7 }), Variant('classifier'), Variant('class')] }), kind: AttributeTypEdge }))@(2:\1)]"];
//         0 -> 1;
//         2       [label="[Package(Content(Update { pos: 0, value: ClassifierKind(Class(IsAbstract(Clear))) }))@(1:1)]"];
//         0 -> 2;
//         3       [label="[Package(Content(Insert { pos: 1, value: ClassifierKind(Class(Classifier(ModelElement(Name(Insert { content: '1', pos: 0 }))))) }))@(\0:1)]"];
//         0 -> 3;
//         5       [label="[Package(ModelElement(Name(Insert { content: 'r', pos: 0 })))@(1:2)]"];
//         2 -> 5;
//         4       [label="[Package(ModelElement(Name(Insert { content: 'y', pos: 0 })))@(6:2)]"];
//         3 -> 4;
//         8       [label="[AddReference(AttributeToClass(Arc { source: AttributeId(ObjectPath { root: 'class_hierarchy', segments: [Field('package'), Field('\content'), ListElement(EventId { idx: ReplicaIdx(6), seq: 1, resolver: 0 => 0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => \7 }), Variant('classifier'), Variant('class'), Field('features'), ListElement(EventId { idx: ReplicaIdx(6), seq: 1, resolver: 0 => \0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => 7 }), Variant('attribute')] }), target: ClassId(ObjectPath { root: 'class_\hierarchy', segments: [Field('package'), Field('content'), ListElement(EventId { idx: ReplicaIdx(0), seq: 1, resolver: 0 => 0, 1 => \1, 2 => 2, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => 7 }), Variant('classifier'), Variant('class')] }), kind: AttributeTypEdge }))@(0:\2)]"];
//         3 -> 8;
//         6       [label="[AddReference(AttributeToClass(Arc { source: AttributeId(ObjectPath { root: 'class_hierarchy', segments: [Field('package'), Field('\content'), ListElement(EventId { idx: ReplicaIdx(0), seq: 1, resolver: 0 => 6, 1 => 0, 2 => 1, 3 => 2, 4 => 3, 5 => 4, 6 => 5, 7 => \7 }), Variant('classifier'), Variant('class'), Field('features'), ListElement(EventId { idx: ReplicaIdx(0), seq: 1, resolver: 0 => \6, 1 => 0, 2 => 1, 3 => 2, 4 => 3, 5 => 4, 6 => 5, 7 => 7 }), Variant('attribute')] }), target: ClassId(ObjectPath { root: 'class_\hierarchy', segments: [Field('package'), Field('content'), ListElement(EventId { idx: ReplicaIdx(0), seq: 1, resolver: 0 => 6, 1 => \0, 2 => 1, 3 => 2, 4 => 3, 5 => 4, 6 => 5, 7 => 7 }), Variant('classifier'), Variant('class')] }), kind: AttributeTypEdge }))@(6:\3)]"];
//         4 -> 6;
//         7       [label="[Package(Content(Update { pos: 0, value: ClassifierKind(Class(IsAbstract(Clear))) }))@(6:4)]"];
//         6 -> 7;
//         9       [label="[Package(ModelElement(Name(Insert { content: 'A', pos: 0 })))@(7:1)]"];
//         8 -> 9;
// }
#[test]
fn divergent_refs() {
    let mut replica_0 = Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::bootstrap(
        "0".to_string(),
        &["0", "1", "2", "3", "4", "5", "6", "7"],
    );
    let mut replica_1 = Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::bootstrap(
        "1".to_string(),
        &["0", "1", "2", "3", "4", "5", "6", "7"],
    );
    let mut replica_2 = Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::bootstrap(
        "2".to_string(),
        &["0", "1", "2", "3", "4", "5", "6", "7"],
    );
    let mut replica_3 = Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::bootstrap(
        "3".to_string(),
        &["0", "1", "2", "3", "4", "5", "6", "7"],
    );
    let mut replica_4 = Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::bootstrap(
        "4".to_string(),
        &["0", "1", "2", "3", "4", "5", "6", "7"],
    );
    let mut replica_5 = Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::bootstrap(
        "5".to_string(),
        &["0", "1", "2", "3", "4", "5", "6", "7"],
    );
    let mut replica_6 = Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::bootstrap(
        "6".to_string(),
        &["0", "1", "2", "3", "4", "5", "6", "7"],
    );
    let mut replica_7 = Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::bootstrap(
        "7".to_string(),
        &["0", "1", "2", "3", "4", "5", "6", "7"],
    );

    let e6_1 = replica_6
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 0,
                value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                    Class::Features(NestedList::Insert {
                        pos: 0,
                        value: StructuralFeatureKind::Attribute(Attribute::StructuralFeatureSuper(
                            StructuralFeature::Upper(Counter::Inc(1)),
                        )),
                    }),
                ))),
            },
        )))
        .unwrap();
    replica_2.receive(e6_1.clone());

    let translated_id = e6_1
        .event()
        .id()
        .translate_ids(ReplicaIdx(6), replica_2.tcsb().interner());
    let attribute_id_path = ObjectPath::new("class_hierarchy")
        .field("package")
        .field("content")
        .list_element(translated_id.clone())
        .variant("classifier")
        .variant("class")
        .field("features")
        .list_element(translated_id.clone())
        .variant("attribute");
    let class_id_path = ObjectPath::new("class_hierarchy")
        .field("package")
        .field("content")
        .list_element(translated_id.clone())
        .variant("classifier")
        .variant("class");
    let e2_1 = replica_2
        .send(ClassHierarchy::AddReference(Refs::AttributeToClass(Arc {
            source: AttributeId(attribute_id_path),
            target: ClassId(class_id_path),
            kind: AttributeTypEdge,
        })))
        .unwrap();

    replica_1.receive(e6_1.clone());

    let e1_1 = replica_1
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: 0,
                value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                    Class::IsAbstract(EWFlag::Clear),
                ))),
            },
        )))
        .unwrap();

    replica_0.receive(e6_1.clone());

    let e0_1 = replica_0
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 1,
                value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                    Class::ClassifierSuper(Classifier::ModelElementSuper(ModelElement::Name(
                        List::Insert {
                            content: '1',
                            pos: 0,
                        },
                    ))),
                ))),
            },
        )))
        .unwrap();

    let e1_2 = replica_1
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Insert {
                content: 'y',
                pos: 0,
            }),
        )))
        .unwrap();

    replica_6.receive(e0_1.clone());

    let e6_2 = replica_6
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Insert {
                pos: 0,
                content: 'y',
            }),
        )))
        .unwrap();

    let translated_e0_1_id = e0_1
        .event()
        .id()
        .translate_ids(ReplicaIdx(6), replica_6.tcsb().interner());
    let translated_e6_1_id = e6_1
        .event()
        .id()
        .translate_ids(ReplicaIdx(0), replica_0.tcsb().interner());
    assert!(translated_e6_1_id.origin_id() == "6");
    assert!(translated_e0_1_id.origin_id() == "0");

    let attribute_id_path = ObjectPath::new("class_hierarchy")
        .field("package")
        .field("content")
        .list_element(translated_e6_1_id.clone())
        .variant("classifier")
        .variant("class")
        .field("features")
        .list_element(translated_e6_1_id.clone())
        .variant("attribute");
    let class_id_path = ObjectPath::new("class_hierarchy")
        .field("package")
        .field("content")
        .list_element(translated_e0_1_id.clone())
        .variant("classifier")
        .variant("class");
    let e0_2 = replica_0
        .send(ClassHierarchy::AddReference(Refs::AttributeToClass(Arc {
            source: AttributeId(attribute_id_path),
            target: ClassId(class_id_path),
            kind: AttributeTypEdge,
        })))
        .unwrap();

    let attribute_id_path = ObjectPath::new("class_hierarchy")
        .field("package")
        .field("content")
        .list_element(e6_1.event().id().clone())
        .variant("classifier")
        .variant("class")
        .field("features")
        .list_element(e6_1.event().id().clone())
        .variant("attribute");
    let class_id_path = ObjectPath::new("class_hierarchy")
        .field("package")
        .field("content")
        .list_element(e6_1.event().id().clone())
        .variant("classifier")
        .variant("class");
    let e6_3 = replica_6
        .send(ClassHierarchy::AddReference(Refs::AttributeToClass(Arc {
            source: AttributeId(attribute_id_path),
            target: ClassId(class_id_path),
            kind: AttributeTypEdge,
        })))
        .unwrap();
    let e6_4 = replica_6
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: 0,
                value: Box::new(ModelElementKind::Classifier(ClassifierKind::Class(
                    Class::IsAbstract(EWFlag::Clear),
                ))),
            },
        )))
        .unwrap();

    replica_7.receive(e6_1.clone());
    replica_7.receive(e0_1.clone());
    replica_7.receive(e0_2.clone());

    let e7_1 = replica_7
        .send(ClassHierarchy::Package(Package::ModelElementSuper(
            ModelElement::Name(List::Insert {
                content: 'A',
                pos: 0,
            }),
        )))
        .unwrap();

    replica_0.receive(e2_1.clone());
    replica_0.receive(e1_1.clone());
    replica_0.receive(e1_2.clone());
    replica_0.receive(e6_2.clone());
    replica_0.receive(e6_3.clone());
    replica_0.receive(e6_4.clone());
    replica_0.receive(e7_1.clone());

    replica_1.receive(e2_1.clone());
    replica_1.receive(e0_1.clone());
    replica_1.receive(e6_2.clone());
    replica_1.receive(e6_3.clone());
    replica_1.receive(e6_4.clone());
    replica_1.receive(e0_2.clone());
    replica_1.receive(e7_1.clone());

    replica_2.receive(e1_1.clone());
    replica_2.receive(e1_2.clone());
    replica_2.receive(e0_1.clone());
    replica_2.receive(e0_2.clone());
    replica_2.receive(e7_1.clone());
    replica_2.receive(e6_2.clone());
    replica_2.receive(e6_3.clone());
    replica_2.receive(e6_4.clone());

    replica_3.receive(e6_1.clone());
    replica_3.receive(e2_1.clone());
    replica_3.receive(e1_1.clone());
    replica_3.receive(e1_2.clone());
    replica_3.receive(e0_1.clone());
    replica_3.receive(e6_2.clone());
    replica_3.receive(e0_2.clone());
    replica_3.receive(e6_3.clone());
    replica_3.receive(e6_4.clone());
    replica_3.receive(e7_1.clone());

    replica_4.receive(e6_1.clone());
    replica_4.receive(e2_1.clone());
    replica_4.receive(e1_1.clone());
    replica_4.receive(e1_2.clone());
    replica_4.receive(e0_1.clone());
    replica_4.receive(e6_2.clone());
    replica_4.receive(e0_2.clone());
    replica_4.receive(e6_3.clone());
    replica_4.receive(e6_4.clone());
    replica_4.receive(e7_1.clone());

    replica_5.receive(e6_1.clone());
    replica_5.receive(e2_1.clone());
    replica_5.receive(e1_1.clone());
    replica_5.receive(e1_2.clone());
    replica_5.receive(e0_1.clone());
    replica_5.receive(e6_2.clone());
    replica_5.receive(e0_2.clone());
    replica_5.receive(e6_3.clone());
    replica_5.receive(e6_4.clone());
    replica_5.receive(e7_1.clone());

    replica_6.receive(e2_1.clone());
    replica_6.receive(e1_1.clone());
    replica_6.receive(e1_2.clone());
    replica_6.receive(e0_2.clone());
    replica_6.receive(e7_1.clone());

    replica_7.receive(e2_1.clone());
    replica_7.receive(e1_1.clone());
    replica_7.receive(e1_2.clone());
    replica_7.receive(e6_2.clone());
    replica_7.receive(e6_3.clone());
    replica_7.receive(e6_4.clone());

    assert_eq!(
        &replica_0.query(Read::new()).package,
        &replica_1.query(Read::new()).package
    );
    assert!(
        vf2::isomorphisms(
            &Vf2GraphView(&replica_0.query(Read::new()).refs),
            &Vf2GraphView(&replica_1.query(Read::new()).refs)
        )
        .default_eq()
        .first()
        .is_some()
    );
    assert_eq!(
        &replica_1.query(Read::new()).package,
        &replica_2.query(Read::new()).package
    );
    assert!(
        vf2::isomorphisms(
            &Vf2GraphView(&replica_1.query(Read::new()).refs),
            &Vf2GraphView(&replica_2.query(Read::new()).refs)
        )
        .default_eq()
        .first()
        .is_some()
    );
    assert_eq!(
        &replica_2.query(Read::new()).package,
        &replica_3.query(Read::new()).package
    );

    assert!(
        vf2::isomorphisms(
            &Vf2GraphView(&replica_2.query(Read::new()).refs),
            &Vf2GraphView(&replica_3.query(Read::new()).refs)
        )
        .default_eq()
        .first()
        .is_some()
    );
    assert_eq!(
        &replica_3.query(Read::new()).package,
        &replica_4.query(Read::new()).package
    );
    assert!(
        vf2::isomorphisms(
            &Vf2GraphView(&replica_3.query(Read::new()).refs),
            &Vf2GraphView(&replica_4.query(Read::new()).refs)
        )
        .default_eq()
        .first()
        .is_some()
    );
    assert_eq!(
        &replica_4.query(Read::new()).package,
        &replica_5.query(Read::new()).package
    );
    assert!(
        vf2::isomorphisms(
            &Vf2GraphView(&replica_4.query(Read::new()).refs),
            &Vf2GraphView(&replica_5.query(Read::new()).refs)
        )
        .default_eq()
        .first()
        .is_some()
    );
    assert_eq!(
        &replica_5.query(Read::new()).package,
        &replica_6.query(Read::new()).package
    );
    assert!(
        vf2::isomorphisms(
            &Vf2GraphView(&replica_5.query(Read::new()).refs),
            &Vf2GraphView(&replica_6.query(Read::new()).refs)
        )
        .default_eq()
        .first()
        .is_some()
    );
    assert_eq!(
        &replica_6.query(Read::new()).package,
        &replica_7.query(Read::new()).package
    );
    assert!(
        vf2::isomorphisms(
            &Vf2GraphView(&replica_6.query(Read::new()).refs),
            &Vf2GraphView(&replica_7.query(Read::new()).refs)
        )
        .default_eq()
        .first()
        .is_some()
    );
    fn is_valid(graph: &Graph<Instance, Ref>) -> bool {
        let is_valid = class_hierarchy::references::validate_schema(&graph);
        let is_valid = match is_valid {
            Ok(_) => true,
            Err(violations) => {
                if violations
                    .iter()
                    .all(|v| matches!(v, SchemaViolation::BelowMin { .. }))
                {
                    true
                } else {
                    println!("Schema violations: {:?}", violations);
                    false
                }
            }
        };
        is_valid
    }
    assert!(is_valid(&replica_0.query(Read::new()).refs));
    assert!(is_valid(&replica_1.query(Read::new()).refs));
    assert!(is_valid(&replica_2.query(Read::new()).refs));
    assert!(is_valid(&replica_3.query(Read::new()).refs));
    assert!(is_valid(&replica_4.query(Read::new()).refs));
    assert!(is_valid(&replica_5.query(Read::new()).refs));
    assert!(is_valid(&replica_6.query(Read::new()).refs));
    assert!(is_valid(&replica_7.query(Read::new()).refs));
}
