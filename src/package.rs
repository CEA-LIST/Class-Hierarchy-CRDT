/// Auto-generated code by 🅰🆁🅰🅲🅷🅽🅴 - do not edit directly
mod __package {
    pub use crate::references::*;
    pub use moirai_protocol::clock::causal_frontier::CausalFrontier;
    pub use moirai_protocol::clock::version_vector::Version;
    pub use moirai_protocol::crdt::eval::EvalNested;
    pub use moirai_protocol::crdt::policy::FairPolicy;
    pub use moirai_protocol::crdt::query::Read;
    pub use moirai_protocol::event::Event as ProtocolEvent;
    pub use moirai_protocol::state::effect_context::EffectContext;
    pub use moirai_protocol::state::log::IsLog;
    pub use moirai_protocol::state::po_log::VecLog;
    pub use moirai_protocol::state::sink::SinkCollector;
    pub use moirai_protocol::state::sink::SinkEffect;
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
    pub refs: petgraph::graph::DiGraph<__package::Instance, __package::Ref>,
}
#[derive(Debug, Clone, Default)]
pub struct ClassHierarchyLog {
    package_log: crate::classifiers::PackageLog,
    reference_manager_log: __package::VecLog<__package::ReferenceManager<__package::FairPolicy>>,
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
    type Command = ClassHierarchy;
    type Op = ClassHierarchy;
    type Rejection = ClassHierarchyRejection;
    fn prepare(&self, command: Self::Command) -> Self::Op {
        command
    }
    fn is_enabled(&self, op: &Self::Op) -> Result<(), Self::Rejection> {
        match op {
            ClassHierarchy::Package(o) => self
                .package_log
                .is_enabled(o)
                .map_err(ClassHierarchyRejection::Package),
            ClassHierarchy::AddReference(o) => self
                .reference_manager_log
                .is_enabled(&__package::ReferenceManager::AddArc(o.clone()))
                .map_err(ClassHierarchyRejection::AddReference),
            ClassHierarchy::RemoveReference(o) => self
                .reference_manager_log
                .is_enabled(&__package::ReferenceManager::RemoveArc(o.clone()))
                .map_err(ClassHierarchyRejection::RemoveReference),
        }
    }
    fn effect(
        &mut self,
        event: __package::ProtocolEvent<Self::Op>,
        _ctx: &mut __package::EffectContext<'_>,
    ) {
        let mut sink = __package::SinkCollector::new();
        {
            let mut ctx = __package::EffectContext::root("class_hierarchy", Some(&mut sink));
            match event.op().clone() {
                ClassHierarchy::Package(o) => {
                    let child_event = __package::ProtocolEvent::unfold(event.clone(), o);
                    ctx.with_field("package", |ctx| {
                        self.package_log.effect(child_event, ctx);
                    });
                }
                ClassHierarchy::AddReference(o) => {
                    let mut ctx = __package::EffectContext::silent();
                    self.reference_manager_log.effect(
                        __package::ProtocolEvent::unfold(
                            event.clone(),
                            __package::ReferenceManager::AddArc(o),
                        ),
                        &mut ctx,
                    );
                }
                ClassHierarchy::RemoveReference(o) => {
                    let mut ctx = __package::EffectContext::silent();
                    self.reference_manager_log.effect(
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
                        .and_then(|kind| __package::instance_from_sink_kind(kind, sink.path()))
                        .map(|instance| __package::ReferenceManager::AddVertex { id: instance });
                    if let Some(o) = vertex_ops {
                        reference_effect_disambiguator += 1;
                        let mut ctx = __package::EffectContext::silent();
                        self.reference_manager_log.effect(
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
                    self.reference_manager_log.effect(
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
    fn stabilize(&mut self, frontier: &__package::CausalFrontier) {
        self.package_log.stabilize(frontier);
        self.reference_manager_log.stabilize(frontier);
    }
    fn redundant_by_parent(&mut self, version: &__package::Version, conservative: bool) {
        self.package_log.redundant_by_parent(version, conservative);
        self.reference_manager_log
            .redundant_by_parent(version, conservative);
    }
    fn is_default(&self) -> bool {
        self.reference_manager_log.is_default() && self.package_log.is_default()
    }
}
impl __package::EvalNested<__package::Read<ClassHierarchyValue>> for ClassHierarchyLog {
    fn execute_query(&self, _q: &__package::Read<ClassHierarchyValue>) -> ClassHierarchyValue {
        ClassHierarchyValue {
            package: self
                .package_log
                .execute_query(&__package::Read::<crate::classifiers::PackageValue>::new()),
            refs: self.reference_manager_log.execute_query(&__package::Read::<
                petgraph::graph::DiGraph<__package::Instance, __package::Ref>,
            >::new()),
        }
    }
}
