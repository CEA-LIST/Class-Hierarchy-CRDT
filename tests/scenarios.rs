use class_hierarchy::{
    classifiers::{
        Attribute, Class, Classifier, ClassifierFeat, DataType, ModelElement, ModelElementFeat,
        Package, Reference, StructuralFeature, StructuralFeatureFeat,
    },
    package::{ClassHierarchy, ClassHierarchyLog, ClassHierarchyValue},
    references::{
        AttributeId, AttributeTypEdge, ClassId, DataTypeId, Instance, Ref, Refs, SchemaViolation,
    },
};
use moirai_crdt::{
    counter::resettable_counter::Counter,
    flag::ew_flag::EWFlag,
    list::{eg_walker::List, nested_list::NestedList},
    utils::membership::twins_log,
};
use moirai_macros::typed_graph::Arc;
use moirai_protocol::{
    broadcast::tcsb::{IsTcsbTest, Tcsb},
    crdt::query::Read,
    replica::{IsReplica, Replica, ReplicaIdx},
    state::sink::ObjectPath,
    utils::translate_ids::TranslateIds,
};
use petgraph::{Direction, Graph, graph::DiGraph};

struct Vf2GraphView<'a>(&'a DiGraph<Instance, class_hierarchy::references::Ref>);

impl<'a> vf2::Graph for Vf2GraphView<'a> {
    type NodeLabel = Instance;
    type EdgeLabel = class_hierarchy::references::Ref;

    fn is_directed(&self) -> bool {
        true
    }

    fn node_count(&self) -> usize {
        self.0.node_count()
    }

    fn node_label(&self, node: vf2::NodeIndex) -> Option<&Self::NodeLabel> {
        self.0.node_weight(petgraph::graph::NodeIndex::new(node))
    }

    fn neighbors(
        &self,
        node: vf2::NodeIndex,
        direction: vf2::Direction,
    ) -> impl Iterator<Item = vf2::NodeIndex> {
        self.0
            .neighbors_directed(
                petgraph::graph::NodeIndex::new(node),
                match direction {
                    vf2::Direction::Outgoing => Direction::Outgoing,
                    vf2::Direction::Incoming => Direction::Incoming,
                },
            )
            .map(|neighbor| neighbor.index())
    }

    fn contains_edge(&self, source: vf2::NodeIndex, target: vf2::NodeIndex) -> bool {
        self.0.contains_edge(
            petgraph::graph::NodeIndex::new(source),
            petgraph::graph::NodeIndex::new(target),
        )
    }

    fn edge_label(
        &self,
        source: vf2::NodeIndex,
        target: vf2::NodeIndex,
    ) -> Option<&Self::EdgeLabel> {
        self.0
            .find_edge(
                petgraph::graph::NodeIndex::new(source),
                petgraph::graph::NodeIndex::new(target),
            )
            .and_then(|edge| self.0.edge_weight(edge))
    }
}

// fn format_instance(instance: &Instance) -> String {
//     match instance {
//         Instance::AttributeId(id) => format!("AttributeId({})", id.0),
//         Instance::ReferenceId(id) => format!("ReferenceId({})", id.0),
//         Instance::ClassId(id) => format!("ClassId({})", id.0),
//         Instance::DataTypeId(id) => format!("DataTypeId({})", id.0),
//         _ => unreachable!(),
//     }
// }

#[test]
fn conflicting_ref_type_max() {
    let (mut replica_a, mut replica_b) = twins_log::<ClassHierarchyLog>();

    let a_1 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 0,
                value: Box::new(ModelElement::Classifier(Classifier::Class(Class::New))),
            },
        )))
        .unwrap();
    let a_2 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 1,
                value: Box::new(ModelElement::Classifier(Classifier::DataType(
                    DataType::New,
                ))),
            },
        )))
        .unwrap();
    let a_3 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: 0,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::Features(NestedList::Insert {
                        pos: 0,
                        value: StructuralFeature::Attribute(Attribute::New),
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

    let read_a = replica_a.query(Read::new());
    let read_b = replica_b.query(Read::new());

    assert_eq!(read_a.package, read_b.package);
    assert!(
        vf2::isomorphisms(&Vf2GraphView(&read_a.refs), &Vf2GraphView(&read_b.refs))
            .default_eq()
            .first()
            .is_some()
    );
    assert_eq!(read_a.refs.node_count(), 3);
    assert_eq!(read_a.refs.edge_count(), 1);
    assert_eq!(read_b.refs.node_count(), 3);
    assert_eq!(read_b.refs.edge_count(), 1);
}

#[test]
fn vertex_cascade_creation_deletion() {
    let mut replica_a = Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::new("a".to_string());

    replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 0,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
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

    println!("{:#?}", replica_a.query(Read::new()).package);
    println!(
        "{:#?}",
        replica_a
            .query(Read::new())
            .refs
            .node_weights()
            .map(|n| match n {
                Instance::AttributeId(id) => format!("{} (AttributeId)", id.0),
                Instance::ReferenceId(id) => format!("{} (ReferenceId)", id.0),
                Instance::ClassId(id) => format!("{} (ClassId)", id.0),
                Instance::DataTypeId(id) => format!("{} (DataTypeId)", id.0),
            })
            .collect::<Vec<_>>()
            .join(",")
    );
    assert_eq!(replica_a.query(Read::new()).refs.node_count(), 0);
}
#[test]
fn zoo() {
    let (mut replica_a, mut replica_b) = twins_log::<ClassHierarchyLog>();

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
        replica_a.query(Read::<ClassHierarchyValue>::new()).package,
        replica_b.query(Read::<ClassHierarchyValue>::new()).package
    );

    println!(
        "{:#?}",
        replica_a.query(Read::<ClassHierarchyValue>::new()).package
    );
    assert_eq!(
        replica_a
            .query(Read::<ClassHierarchyValue>::new())
            .package
            .content
            .len(),
        1
    );

    println!("Vertices in the graph after events a1-a6:");
    for vertex in replica_a
        .query(Read::<ClassHierarchyValue>::new())
        .refs
        .node_weights()
    {
        match vertex {
            Instance::AttributeId(id) => println!("{} (AttributeId)", id.0),
            Instance::ReferenceId(id) => println!("{} (ReferenceId)", id.0),
            Instance::ClassId(id) => println!("{} (ClassId)", id.0),
            Instance::DataTypeId(id) => println!("{} (DataTypeId)", id.0),
        }
    }
}

/// This test reproduce this execution trace:
/// digraph {
///         0       [label="[Package(ModelElementFeat(Name(Insert { content: 'X', pos: 0 })))@(1:1)]"];
///         12      [label="[Package(Content(Delete { pos: 0 }))@(1:2)]"];
///         0 -> 12;
///         1       [label="[Package(ModelElementFeat(Name(Insert { content: 'c', pos: 0 })))@(4:1)]"];
///         2       [label="[Package(Content(Insert { pos: 0, value: StructuralFeature(Reference(StructuralFeatureFeat(IsOrdered(Enable)))) }))@(5:1)]"];
///         3       [label="[Package(Content(Insert { pos: 1, value: StructuralFeature(Reference(StructuralFeatureFeat(Upper(Dec(239778))))) }))@(5:2)]"];
///         2 -> 3;
///         4       [label="[Package(ModelElementFeat(Name(Insert { content: 'p', pos: 0 })))@(7:1)]"];
///         2 -> 4;
///         5       [label="[Package(Content(Insert { pos: 0, value: Package(Content(Insert { pos: 0, value: Classifier(Class(Features(Insert { pos: 0, value: \
/// Attribute(StructuralFeatureFeat(IsOrdered(Disable))) }))) })) }))@(0:1)]"];
///         2 -> 5;
///         9       [label="[Package(Content(Delete { pos: 1 }))@(5:3)]"];
///         3 -> 9;
///         6       [label="[Package(ModelElementFeat(Name(Delete { pos: 0 })))@(7:2)]"];
///         4 -> 6;
///         7       [label="[AddReference(ReferenceToClass(Arc { source: ReferenceId(ObjectPath { root: 'class_hierarchy', segments: [Field('package'), Field('\
/// content'), ListElement(EventId { idx: ReplicaIdx(6), seq: 1, resolver: 0 => 7, 1 => 0, 2 => 1, 3 => 2, 4 => 3, 5 => 4, 6 => 5, 7 => \
/// 6 }), Variant('structuralfeature'), Variant('reference')] }), target: ClassId(ObjectPath { root: 'class_hierarchy', segments: [Field('\
/// package'), Field('content'), ListElement(EventId { idx: ReplicaIdx(1), seq: 1, resolver: 0 => 7, 1 => 0, 2 => 1, 3 => 2, 4 => 3, \
/// 5 => 4, 6 => 5, 7 => 6 }), Variant('package'), Field('content'), ListElement(EventId { idx: ReplicaIdx(1), seq: 1, resolver: 0 => \
/// 7, 1 => 0, 2 => 1, 3 => 2, 4 => 3, 5 => 4, 6 => 5, 7 => 6 }), Variant('classifier'), Variant('class')] }), kind: ReferenceTypEdge }))@(\
/// 7:3)]"];
///         5 -> 7;
///         11      [label="[AddReference(ReferenceToClass(Arc { source: ReferenceId(ObjectPath { root: 'class_hierarchy', segments: [Field('package'), Field('\
/// content'), ListElement(EventId { idx: ReplicaIdx(5), seq: 1, resolver: 0 => 0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => \
/// 7 }), Variant('structuralfeature'), Variant('reference')] }), target: ClassId(ObjectPath { root: 'class_hierarchy', segments: [Field('\
/// package'), Field('content'), ListElement(EventId { idx: ReplicaIdx(0), seq: 1, resolver: 0 => 0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, \
/// 5 => 5, 6 => 6, 7 => 7 }), Variant('package'), Field('content'), ListElement(EventId { idx: ReplicaIdx(0), seq: 1, resolver: 0 => \
/// 0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => 7 }), Variant('classifier'), Variant('class')] }), kind: ReferenceTypEdge }))@(\
/// 0:2)]"];
///         5 -> 11;
///         6 -> 7;
///         8       [label="[Package(ModelElementFeat(Name(Insert { content: 'u', pos: 0 })))@(3:1)]"];
///         7 -> 8;
///         10      [label="[Package(ModelElementFeat(Name(Delete { pos: 0 })))@(7:4)]"];
///         8 -> 10;
///         14      [label="[Package(ModelElementFeat(Name(Insert { content: 'B', pos: 0 })))@(5:4)]"];
///         9 -> 14;
///         10 -> 12;
///         19      [label="[Package(Content(Update { pos: 0, value: Package(Content(Update { pos: 0, value: Classifier(Class(Features(Update { pos: 0, value: \
/// Attribute(StructuralFeatureFeat(ModelElementFeat(Name(Insert { content: 'K', pos: 0 })))) }))) })) }))@(3:2)]"];
///         10 -> 19;
///         15      [label="[Package(ModelElementFeat(Name(Insert { content: 'N', pos: 0 })))@(0:3)]"];
///         11 -> 15;
///         13      [label="[Package(ModelElementFeat(Name(Delete { pos: 0 })))@(1:3)]"];
///         12 -> 13;
///         13 -> 14;
///         17      [label="[Package(ModelElementFeat(Name(Delete { pos: 0 })))@(2:1)]"];
///         14 -> 17;
///         16      [label="[Package(Content(Delete { pos: 0 }))@(0:4)]"];
///         15 -> 16;
///         18      [label="[Package(ModelElementFeat(Name(Insert { content: 'q', pos: 0 })))@(0:5)]"];
///         16 -> 18;
/// }
/// This test reproduce this execution trace:
/// digraph {
///     0 [ label="[Package(ModelElementFeat(Name(Insert { content: 'i', pos: 0 })))@(0:1)]"]
///     1 [ label="[Package(Content(Insert { pos: 0, value: Classifier(Class(Features(Insert { pos: 0, value: Attribute(StructuralFeatureFeat(Lower(Reset))) }))) }))@(1:1)]"]
///     2 [ label="[Package(Content(Delete { pos: 0 }))@(1:2)]"]
///     3 [ label="[AddReference(AttributeToClass(Arc { source: AttributeId(ObjectPath { root: 'class_hierarchy', segments: [Field('package'), Field('content'), ListElement(EventId { idx: ReplicaIdx(1), seq: 1, resolver: 0 => 0, 1 => 1, 2 => 2 }), Variant('classifier'), Variant('class'), Field('features'), ListElement(EventId { idx: ReplicaIdx(1), seq: 1, resolver: 0 => 0, 1 => 1, 2 => 2 }), Variant('attribute')] }), target: ClassId(ObjectPath { root: 'class_hierarchy', segments: [Field('package'), Field('content'), ListElement(EventId { idx: ReplicaIdx(1), seq: 1, resolver: 0 => 0, 1 => 1, 2 => 2 }), Variant('classifier'), Variant('class')] }), kind: AttributeTypEdge }))@(0:2)]"]
///     4 [ label="[Package(ModelElementFeat(Name(Insert { content: 'a', pos: 1 })))@(2:1)]"]
///     5 [ label="[Package(ModelElementFeat(Name(Insert { content: 'P', pos: 2 })))@(2:2)]"]
///     6 [ label="[Package(ModelElementFeat(Name(Insert { content: 'h', pos: 2 })))@(2:3)]"]
///     7 [ label="[Package(Content(Update { pos: 0, value: Classifier(Class(Features(Delete { pos: 0 }))) }))@(0:3)]"]
///     8 [ label="[Package(Content(Update { pos: 0, value: Classifier(Class(Features(Update { pos: 0, value: Attribute(StructuralFeatureFeat(IsUnique(Enable))) }))) }))@(2:4)]"]
///     9 [ label="[Package(Content(Insert { pos: 1, value: StructuralFeature(Attribute(StructuralFeatureFeat(IsUnique(Clear)))) }))@(0:4)]"]
///     0 -> 1 [ ]  1 -> 2 [ ]  0 -> 2 [ ]  0 -> 3 [ ]  1 -> 3 [ ]  3 -> 4 [ ]  1 -> 4 [ ]  4 -> 5 [ ]  3 -> 5 [ ]  1 -> 5 [ ]  5 -> 6 [ ]  3 -> 6 [ ] 	1 ->	6 [ ]	3 ->	7 [ ]	1 ->	7 [ ]	6 ->	7 [ ]	6 ->	8 [ ]	3 ->	8 [ ]	1 ->	8 [ ]	7 ->	9 [ ]	1 ->	9 [ ]	6 ->	9 [ ]
/// }
#[test]
fn error_case() {
    let mut replica_a = Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::new("a".to_string());
    let mut replica_b = Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::new("b".to_string());
    let mut replica_c = Replica::<ClassHierarchyLog, Tcsb<ClassHierarchy>>::new("c".to_string());

    // 0 [Package(ModelElementFeat(Name(Insert { content: 'i', pos: 0 })))@(0:1)]
    let e0 = replica_a
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Insert {
                content: 'i',
                pos: 0,
            }),
        )))
        .unwrap();

    // 0 -> 1,2,3
    replica_b.receive(e0.clone());

    // 1 [Package(Content(Insert { pos: 0, value: Classifier(Class(Features(Insert { pos: 0, value: Attribute(StructuralFeatureFeat(Lower(Reset))) }))) }))@(1:1)]
    let e1 = replica_b
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 0,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::Features(NestedList::Insert {
                        pos: 0,
                        value: StructuralFeature::Attribute(Attribute::StructuralFeatureFeat(
                            StructuralFeatureFeat::Lower(
                                moirai_crdt::counter::resettable_counter::Counter::Reset,
                            ),
                        )),
                    }),
                ))),
            },
        )))
        .unwrap();

    // 2 [Package(Content(Delete { pos: 0 }))@(1:2)]
    let e2 = replica_b
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Delete { pos: 0 },
        )))
        .unwrap();

    // 0 -> 3, 1 -> 3
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

    // 3 [AddReference(AttributeToClass(...))@(0:2)]
    let e3 = replica_a
        .send(ClassHierarchy::AddReference(Refs::AttributeToClass(Arc {
            source: AttributeId(attribute_path),
            target: ClassId(class_path.clone()),
            kind: AttributeTypEdge,
        })))
        .unwrap();

    // 1 -> 4,5,6 and 3 -> 4,5,6
    // e3 depends on e0 transitively, and the package-name inserts below require that base state.
    replica_c.receive(e0.clone());
    replica_c.receive(e1.clone());
    replica_c.receive(e3.clone());

    // 4 [Package(ModelElementFeat(Name(Insert { content: 'a', pos: 1 })))@(2:1)]
    let e4 = replica_c
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Insert {
                content: 'a',
                pos: 1,
            }),
        )))
        .unwrap();

    // 5 [Package(ModelElementFeat(Name(Insert { content: 'P', pos: 2 })))@(2:2)]
    let e5 = replica_c
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Insert {
                content: 'P',
                pos: 2,
            }),
        )))
        .unwrap();

    // 6 [Package(ModelElementFeat(Name(Insert { content: 'h', pos: 2 })))@(2:3)]
    let e6 = replica_c
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Insert {
                content: 'h',
                pos: 2,
            }),
        )))
        .unwrap();

    // 1 -> 7, 3 -> 7, 6 -> 7
    replica_a.receive(e6.clone());

    // 7 [Package(Content(Update { pos: 0, value: Classifier(Class(Features(Delete { pos: 0 }))) }))@(0:3)]
    let e7 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: 0,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::Features(NestedList::Delete { pos: 0 }),
                ))),
            },
        )))
        .unwrap();

    // 1 -> 8, 3 -> 8, 6 -> 8
    // c already has 1,3,6
    let e8 = replica_c
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Update {
                pos: 0,
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::Features(NestedList::Update {
                        pos: 0,
                        value: StructuralFeature::Attribute(Attribute::StructuralFeatureFeat(
                            StructuralFeatureFeat::IsUnique(EWFlag::Enable),
                        )),
                    }),
                ))),
            },
        )))
        .unwrap();

    // 1 -> 9, 6 -> 9, 7 -> 9
    let e9 = replica_a
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 1,
                value: Box::new(ModelElement::StructuralFeature(
                    StructuralFeature::Attribute(Attribute::StructuralFeatureFeat(
                        StructuralFeatureFeat::IsUnique(EWFlag::Clear),
                    )),
                )),
            },
        )))
        .unwrap();

    // Final dissemination of the full trace to the non-origin replicas only.
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
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Insert {
                content: 'X',
                pos: 0,
            }),
        )))
        .unwrap();

    let e1 = replicas[4]
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Insert {
                content: 'c',
                pos: 0,
            }),
        )))
        .unwrap();

    let e2 = replicas[5]
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 0,
                value: Box::new(ModelElement::StructuralFeature(
                    StructuralFeature::Reference(Reference::StructuralFeatureFeat(
                        StructuralFeatureFeat::IsOrdered(EWFlag::Enable),
                    )),
                )),
            },
        )))
        .unwrap();

    let e3 = replicas[5]
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Insert {
                pos: 1,
                value: Box::new(ModelElement::StructuralFeature(
                    StructuralFeature::Reference(Reference::StructuralFeatureFeat(
                        StructuralFeatureFeat::Upper(
                            moirai_crdt::counter::resettable_counter::Counter::Dec(239_778),
                        ),
                    )),
                )),
            },
        )))
        .unwrap();

    replicas[7].receive(e2.clone());
    let e4 = replicas[7]
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Insert {
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
                value: Box::new(ModelElement::Package(Package::Content(
                    NestedList::Insert {
                        pos: 0,
                        value: Box::new(ModelElement::Classifier(Classifier::Class(
                            Class::Features(NestedList::Insert {
                                pos: 0,
                                value: StructuralFeature::Attribute(
                                    Attribute::StructuralFeatureFeat(
                                        StructuralFeatureFeat::IsOrdered(EWFlag::Disable),
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
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Delete { pos: 0 }),
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
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Insert {
                content: 'u',
                pos: 0,
            }),
        )))
        .unwrap();

    replicas[7].receive(e8.clone());
    let e10 = replicas[7]
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Delete { pos: 0 }),
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
            .send(ClassHierarchy::Package(Package::ModelElementFeat(
                ModelElementFeat::Name(List::Insert {
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
                value: Box::new(ModelElement::Package(Package::Content(
                    NestedList::Update {
                        pos: 0,
                        value: Box::new(ModelElement::Classifier(Classifier::Class(
                            Class::Features(NestedList::Update {
                                pos: 0,
                                value: StructuralFeature::Attribute(
                                    Attribute::StructuralFeatureFeat(
                                        StructuralFeatureFeat::ModelElementFeat(
                                            ModelElementFeat::Name(List::Insert {
                                                content: 'K',
                                                pos: 0,
                                            }),
                                        ),
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
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Insert {
                content: 'N',
                pos: 0,
            }),
        )))
        .unwrap();

    let e13 = replicas[1]
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Delete { pos: 0 }),
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
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Delete { pos: 0 }),
        )))
        .unwrap();

    let e16 = replicas[0]
        .send(ClassHierarchy::Package(Package::Content(
            NestedList::Delete { pos: 0 },
        )))
        .unwrap();

    let e18 = replicas[0]
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Insert {
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
//         0       [label="[Package(Content(Insert { pos: 0, value: Classifier(Class(Features(Insert { pos: 0, value: Attribute(StructuralFeatureFeat(Upper(\Inc(130404)))) }))) }))@(6:1)]"];
//         1       [label="[AddReference(AttributeToClass(Arc { source: AttributeId(ObjectPath { root: 'class_hierarchy', segments: [Field('package'), Field('\content'), ListElement(EventId { idx: ReplicaIdx(6), seq: 1, resolver: 0 => 2, 1 => 0, 2 => 1, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => \7 }), Variant('classifier'), Variant('class'), Field('features'), ListElement(EventId { idx: ReplicaIdx(6), seq: 1, resolver: 0 => \2, 1 => 0, 2 => 1, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => 7 }), Variant('attribute')] }), target: ClassId(ObjectPath { root: 'class_\hierarchy', segments: [Field('package'), Field('content'), ListElement(EventId { idx: ReplicaIdx(6), seq: 1, resolver: 0 => 2, 1 => \0, 2 => 1, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => 7 }), Variant('classifier'), Variant('class')] }), kind: AttributeTypEdge }))@(2:\1)]"];
//         0 -> 1;
//         2       [label="[Package(Content(Update { pos: 0, value: Classifier(Class(IsAbstract(Clear))) }))@(1:1)]"];
//         0 -> 2;
//         3       [label="[Package(Content(Insert { pos: 1, value: Classifier(Class(ClassifierFeat(ModelElementFeat(Name(Insert { content: '1', pos: 0 }))))) }))@(\0:1)]"];
//         0 -> 3;
//         5       [label="[Package(ModelElementFeat(Name(Insert { content: 'r', pos: 0 })))@(1:2)]"];
//         2 -> 5;
//         4       [label="[Package(ModelElementFeat(Name(Insert { content: 'y', pos: 0 })))@(6:2)]"];
//         3 -> 4;
//         8       [label="[AddReference(AttributeToClass(Arc { source: AttributeId(ObjectPath { root: 'class_hierarchy', segments: [Field('package'), Field('\content'), ListElement(EventId { idx: ReplicaIdx(6), seq: 1, resolver: 0 => 0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => \7 }), Variant('classifier'), Variant('class'), Field('features'), ListElement(EventId { idx: ReplicaIdx(6), seq: 1, resolver: 0 => \0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => 7 }), Variant('attribute')] }), target: ClassId(ObjectPath { root: 'class_\hierarchy', segments: [Field('package'), Field('content'), ListElement(EventId { idx: ReplicaIdx(0), seq: 1, resolver: 0 => 0, 1 => \1, 2 => 2, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => 7 }), Variant('classifier'), Variant('class')] }), kind: AttributeTypEdge }))@(0:\2)]"];
//         3 -> 8;
//         6       [label="[AddReference(AttributeToClass(Arc { source: AttributeId(ObjectPath { root: 'class_hierarchy', segments: [Field('package'), Field('\content'), ListElement(EventId { idx: ReplicaIdx(0), seq: 1, resolver: 0 => 6, 1 => 0, 2 => 1, 3 => 2, 4 => 3, 5 => 4, 6 => 5, 7 => \7 }), Variant('classifier'), Variant('class'), Field('features'), ListElement(EventId { idx: ReplicaIdx(0), seq: 1, resolver: 0 => \6, 1 => 0, 2 => 1, 3 => 2, 4 => 3, 5 => 4, 6 => 5, 7 => 7 }), Variant('attribute')] }), target: ClassId(ObjectPath { root: 'class_\hierarchy', segments: [Field('package'), Field('content'), ListElement(EventId { idx: ReplicaIdx(0), seq: 1, resolver: 0 => 6, 1 => \0, 2 => 1, 3 => 2, 4 => 3, 5 => 4, 6 => 5, 7 => 7 }), Variant('classifier'), Variant('class')] }), kind: AttributeTypEdge }))@(6:\3)]"];
//         4 -> 6;
//         7       [label="[Package(Content(Update { pos: 0, value: Classifier(Class(IsAbstract(Clear))) }))@(6:4)]"];
//         6 -> 7;
//         9       [label="[Package(ModelElementFeat(Name(Insert { content: 'A', pos: 0 })))@(7:1)]"];
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
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::Features(NestedList::Insert {
                        pos: 0,
                        value: StructuralFeature::Attribute(Attribute::StructuralFeatureFeat(
                            StructuralFeatureFeat::Upper(Counter::Inc(1)),
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
                value: Box::new(ModelElement::Classifier(Classifier::Class(
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
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::ClassifierFeat(ClassifierFeat::ModelElementFeat(
                        ModelElementFeat::Name(List::Insert {
                            content: '1',
                            pos: 0,
                        }),
                    )),
                ))),
            },
        )))
        .unwrap();

    let e1_2 = replica_1
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Insert {
                content: 'y',
                pos: 0,
            }),
        )))
        .unwrap();

    replica_6.receive(e0_1.clone());

    let e6_2 = replica_6
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Insert {
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
                value: Box::new(ModelElement::Classifier(Classifier::Class(
                    Class::IsAbstract(EWFlag::Clear),
                ))),
            },
        )))
        .unwrap();

    replica_7.receive(e6_1.clone());
    replica_7.receive(e0_1.clone());
    replica_7.receive(e0_2.clone());

    let e7_1 = replica_7
        .send(ClassHierarchy::Package(Package::ModelElementFeat(
            ModelElementFeat::Name(List::Insert {
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

#[test]
fn fuzz() {
    use moirai_fuzz::{
        config::{FuzzerConfig, RunConfig},
        fuzzer::fuzzer,
    };

    let run = RunConfig::new(0.9, 4, 1, None, None, false, false);
    let runs = vec![run.clone(); 1];

    let config = FuzzerConfig::<ClassHierarchyLog>::new(
        "class_hierarchy",
        runs,
        true,
        |a, b| {
            let package = a.package == b.package;
            if !package {
                println!("Package mismatch");
                println!("----- Package A -----");
                println!("{:#?}", a.package);
                println!("----- Package B -----");
                println!("{:#?}", b.package);
                return false;
            }

            if a.refs.node_count() == 0 && b.refs.node_count() == 0 {
                // If both graphs are empty, skip the isomorphism check to avoid false negatives due to different node IDs.
                return true;
            } else {
                let refs = vf2::isomorphisms(&Vf2GraphView(&a.refs), &Vf2GraphView(&b.refs))
                    .default_eq()
                    .first()
                    .is_some();
                if !refs {
                    println!(
                        "Graph isomorphism mismatch: nodes {} vs {}, edges {} vs {}",
                        a.refs.node_count(),
                        b.refs.node_count(),
                        a.refs.edge_count(),
                        b.refs.edge_count()
                    );
                    println!("----- Graph A -----");
                    println!("{:#?}", a.refs);
                    println!("----- Graph B -----");
                    println!("{:#?}", b.refs);
                }
                return refs;
            }
        },
        false,
    );

    fuzzer::<ClassHierarchyLog>(config);
}
