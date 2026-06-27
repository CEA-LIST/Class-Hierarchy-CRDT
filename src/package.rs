/// Auto-generated code by 🅰🆁🅰🅲🅷🅽🅴 - do not edit directly
mod __package {
    pub use moirai_protocol::crdt::query::Read;
    pub use moirai_protocol::crdt::eval::EvalNested;
    pub use moirai_protocol::state::log::IsLog;
    pub use moirai_protocol::clock::version_vector::Version;
    pub use moirai_protocol::event::Event;
    pub use moirai_protocol::crdt::query::QueryOperation;
    pub use moirai_protocol::state::sink::SinkEffect;
    pub use moirai_protocol::state::effect_context::EffectContext;
    pub use moirai_protocol::utils::intern_str::Interner;
    pub use moirai_protocol::utils::intern_str::InternalizeOp;
    pub use moirai_protocol::state::sink::SinkCollector;
    pub use moirai_protocol::state::po_log::POLog;
    pub use crate::classifiers::*;
    pub use moirai_crdt::policy::FairPolicy;
    pub use moirai_protocol::state::po_log::VecLog;
    pub use moirai_protocol::crdt::pure_crdt::PureCRDT;
    pub use crate::references::*;
}
pub type ReferenceManagerLog = __package::POLog<
    __package::ReferenceManager<__package::FairPolicy>,
    __package::ReferenceManagerState<__package::FairPolicy>,
>;
#[derive(Debug, Clone)]
pub enum ClassHierarchy {
    Package(__package::Package),
    AddReference(__package::Refs),
    RemoveReference(__package::Refs),
}
#[derive(Debug)]
pub enum ClassHierarchyRejection {
    Package(<__package::PackageLog as __package::IsLog>::Rejection),
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
    pub package: __package::PackageValue,
    pub refs: <__package::ReferenceManager<
        __package::FairPolicy,
    > as __package::PureCRDT>::Value,
}
#[derive(Debug, Clone, Default)]
pub struct ClassHierarchyLog {
    package_log: __package::PackageLog,
    reference_manager_log: __package::VecLog<
        __package::ReferenceManager<__package::FairPolicy>,
    >,
}
impl ClassHierarchyLog {
    pub fn package_log(&self) -> &__package::PackageLog {
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
        event: __package::Event<Self::Op>,
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
                    let child_event = __package::Event::unfold(event.clone(), o);
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
                            __package::Event::unfold(
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
                            __package::Event::unfold(
                                event.clone(),
                                __package::ReferenceManager::RemoveArc(o),
                            ),
                            &mut ctx,
                        );
                }
            }
        }
        for sink in sink.into_sinks() {
            match sink.effect() {
                __package::SinkEffect::Create | __package::SinkEffect::Update => {
                    let vertex_ops = __package::instance_from_path(sink.path())
                        .map(|instance| __package::ReferenceManager::AddVertex {
                            id: instance,
                        });
                    if let Some(o) = vertex_ops {
                        let mut ctx = __package::EffectContext::silent();
                        self.reference_manager_log
                            .effect(
                                __package::Event::unfold(event.clone(), o),
                                &mut ctx,
                            );
                    }
                }
                __package::SinkEffect::Delete => {
                    let mut ctx = __package::EffectContext::silent();
                    self.reference_manager_log
                        .effect(
                            __package::Event::unfold(
                                event.clone(),
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
        true && self.package_log.is_default()
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
