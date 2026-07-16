/// Auto-generated code by 🅰🆁🅰🅲🅷🅽🅴 - do not edit directly
mod __package {
    pub use moirai_protocol::crdt::query::Read;
    pub use moirai_protocol::crdt::eval::EvalNested;
    pub use moirai_protocol::state::log::IsLog;
    pub use moirai_protocol::clock::version_vector::Version;
    pub use moirai_protocol::event::Event as ProtocolEvent;
    pub use moirai_protocol::crdt::query::QueryOperation;
    pub use moirai_protocol::state::sink::SinkEffect;
    pub use moirai_protocol::state::effect_context::EffectContext;
    pub use moirai_protocol::broadcast::internalizer::Interner;
    pub use moirai_protocol::broadcast::internalizer::InternalizeOp;
    pub use moirai_protocol::state::sink::SinkCollector;
    pub use moirai_crdt::policy::FairPolicy;
    pub use moirai_protocol::state::po_log::VecLog;
    pub use moirai_protocol::crdt::pure_crdt::PureCRDT;
    pub use crate::references::*;
}
#[derive(Debug, Clone)]
pub enum ClassHierarchy {
    Package(crate::classifiers::Package),
    AddReference(__package::Refs),
    RemoveReference(__package::Refs),
}
#[derive(Debug)]
pub enum ClassHierarchyRejection {
    Package(<crate::classifiers::PackageLog as __package::IsLog>::Rejection),
    AddReference(
        <__package::VecLog<
            __package::ReferenceManager<__package::FairPolicy>,
        > as __package::IsLog>::Rejection,
    ),
    RemoveReference(
        <__package::VecLog<
            __package::ReferenceManager<__package::FairPolicy>,
        > as __package::IsLog>::Rejection,
    ),
}
impl std::fmt::Display for ClassHierarchyRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Package(error) => write!(f, "{}: {}", "Package", error),
            Self::AddReference(error) => write!(f, "AddReference: {}", error),
            Self::RemoveReference(error) => write!(f, "RemoveReference: {}", error),
        }
    }
}
#[derive(Debug, Clone, Default)]
pub struct ClassHierarchyValue {
    pub package: crate::classifiers::PackageValue,
    pub refs: <__package::ReferenceManager<
        __package::FairPolicy,
    > as __package::PureCRDT>::Value,
}
#[derive(Debug, Clone, Default)]
pub struct ClassHierarchyLog {
    package_log: crate::classifiers::PackageLog,
    reference_manager_log: __package::VecLog<
        __package::ReferenceManager<__package::FairPolicy>,
    >,
}
impl ClassHierarchyLog {
    pub fn package_log(&self) -> &crate::classifiers::PackageLog {
        &self.package_log
    }
    pub fn reference_manager_log(
        &self,
    ) -> &__package::VecLog<__package::ReferenceManager<__package::FairPolicy>> {
        &self.reference_manager_log
    }
}
impl __package::IsLog for ClassHierarchyLog {
    type Value = ClassHierarchyValue;
    type Op = ClassHierarchy;
    type Rejection = ClassHierarchyRejection;
    fn is_enabled(&self, op: &Self::Op) -> Result<(), Self::Rejection> {
        match op {
            ClassHierarchy::Package(o) => {
                self.package_log.is_enabled(o).map_err(ClassHierarchyRejection::Package)
            }
            ClassHierarchy::AddReference(o) => {
                self.reference_manager_log
                    .is_enabled(&__package::ReferenceManager::AddArc(o.clone()))
                    .map_err(ClassHierarchyRejection::AddReference)
            }
            ClassHierarchy::RemoveReference(o) => {
                self.reference_manager_log
                    .is_enabled(&__package::ReferenceManager::RemoveArc(o.clone()))
                    .map_err(ClassHierarchyRejection::RemoveReference)
            }
        }
    }
    fn effect(
        &mut self,
        event: __package::ProtocolEvent<Self::Op>,
        _ctx: &mut __package::EffectContext<'_>,
    ) {
        let mut sink = __package::SinkCollector::new();
        {
            let mut ctx = __package::EffectContext::root(
                "class_hierarchy",
                Some(&mut sink),
            );
            match event.op().clone() {
                ClassHierarchy::Package(o) => {
                    let child_event = __package::ProtocolEvent::unfold(event.clone(), o);
                    ctx.with_field(
                        "package",
                        |ctx| {
                            self.package_log.effect(child_event, ctx);
                        },
                    );
                }
                ClassHierarchy::AddReference(o) => {
                    let mut ctx = __package::EffectContext::silent();
                    self.reference_manager_log
                        .effect(
                            __package::ProtocolEvent::unfold(
                                event.clone(),
                                __package::ReferenceManager::AddArc(o),
                            ),
                            &mut ctx,
                        );
                }
                ClassHierarchy::RemoveReference(o) => {
                    let mut ctx = __package::EffectContext::silent();
                    self.reference_manager_log
                        .effect(
                            __package::ProtocolEvent::unfold(
                                event.clone(),
                                __package::ReferenceManager::RemoveArc(o),
                            ),
                            &mut ctx,
                        );
                }
            }
        }
        let mut reference_effect_disambiguator = 0u32;
        for sink in sink.into_sinks() {
            match sink.effect() {
                __package::SinkEffect::Create | __package::SinkEffect::Update => {
                    let vertex_ops = sink
                        .kind()
                        .and_then(|kind| __package::instance_from_sink_kind(
                            kind,
                            sink.path(),
                        ))
                        .map(|instance| __package::ReferenceManager::AddVertex {
                            id: instance,
                        });
                    if let Some(o) = vertex_ops {
                        reference_effect_disambiguator += 1;
                        let mut ctx = __package::EffectContext::silent();
                        self.reference_manager_log
                            .effect(
                                __package::ProtocolEvent::unfold_with_disambiguator(
                                    event.clone(),
                                    reference_effect_disambiguator,
                                    o,
                                ),
                                &mut ctx,
                            );
                    }
                }
                __package::SinkEffect::Delete => {
                    reference_effect_disambiguator += 1;
                    let mut ctx = __package::EffectContext::silent();
                    self.reference_manager_log
                        .effect(
                            __package::ProtocolEvent::unfold_with_disambiguator(
                                event.clone(),
                                reference_effect_disambiguator,
                                __package::ReferenceManager::DeleteSubtree {
                                    prefix: sink.path().clone(),
                                },
                            ),
                            &mut ctx,
                        );
                }
            }
        }
    }
    fn stabilize(&mut self, version: &__package::Version) {
        self.package_log.stabilize(version);
        self.reference_manager_log.stabilize(version);
    }
    fn redundant_by_parent(&mut self, version: &__package::Version, conservative: bool) {
        self.package_log.redundant_by_parent(version, conservative);
        self.reference_manager_log.redundant_by_parent(version, conservative);
    }
    fn is_default(&self) -> bool {
        self.reference_manager_log.is_default() && self.package_log.is_default()
    }
}
impl __package::EvalNested<__package::Read<<Self as __package::IsLog>::Value>>
for ClassHierarchyLog {
    fn execute_query(
        &self,
        _q: __package::Read<<Self as __package::IsLog>::Value>,
    ) -> <__package::Read<
        <Self as __package::IsLog>::Value,
    > as __package::QueryOperation>::Response {
        ClassHierarchyValue {
            package: self.package_log.execute_query(__package::Read::new()),
            refs: self.reference_manager_log.execute_query(__package::Read::new()),
        }
    }
}
impl __package::InternalizeOp for ClassHierarchy {
    fn internalize(self, interner: &__package::Interner) -> Self {
        match self {
            ClassHierarchy::Package(op) => ClassHierarchy::Package(op.clone()),
            ClassHierarchy::AddReference(op) => {
                ClassHierarchy::AddReference(op.internalize(interner))
            }
            ClassHierarchy::RemoveReference(op) => {
                ClassHierarchy::RemoveReference(op.internalize(interner))
            }
        }
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub struct ReadAsEcore;
impl __package::QueryOperation for ReadAsEcore {
    type Response = Vec<u8>;
}
impl ReadAsEcore {
    pub fn new() -> Self {
        Self
    }
}
impl __package::EvalNested<ReadAsEcore> for ClassHierarchyLog {
    fn execute_query(
        &self,
        _q: ReadAsEcore,
    ) -> <ReadAsEcore as __package::QueryOperation>::Response {
        let mut document_root = xml_builder::XMLElement::new("xmi:XMI");
        document_root.add_attribute("xmi:version", "2.0");
        document_root.add_attribute("xmlns:xmi", "http://www.omg.org/XMI");
        document_root
            .add_attribute("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance");
        document_root
            .add_attribute(
                "xmlns:class_hierarchy",
                "http://www.example.org/class_hierarchy",
            );
        document_root
            .add_child(xml_builder::XMLElement::new("class_hierarchy:Package"))
            .expect("adding a root object to the XMI document should not fail");
        let mut xml = xml_builder::XMLBuilder::new()
            .version(xml_builder::XMLVersion::XML1_0)
            .encoding("UTF-8".into())
            .build();
        xml.set_root_element(document_root);
        let mut writer = Vec::new();
        xml.generate(&mut writer)
            .expect("writing Ecore XMI to an in-memory buffer should not fail");
        writer
    }
}
