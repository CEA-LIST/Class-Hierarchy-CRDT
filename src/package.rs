/// Auto-generated code by 🅰🆁🅰🅲🅷🅽🅴 - do not edit directly
mod __package {
    pub use crate::classifiers::*;
    pub use crate::reference_manager_log::ReferenceManagerLog;
    pub use crate::references::*;
    pub use moirai_crdt::policy::LwwPolicy;
    pub use moirai_protocol::clock::version_vector::Version;
    pub use moirai_protocol::crdt::eval::EvalNested;
    pub use moirai_protocol::crdt::pure_crdt::PureCRDT;
    pub use moirai_protocol::crdt::query::QueryOperation;
    pub use moirai_protocol::crdt::query::Read;
    pub use moirai_protocol::event::Event;
    pub use moirai_protocol::event::id::EventId;
    pub use moirai_protocol::state::log::IsLog;
    pub use moirai_protocol::state::sink::IsLogSink;
    pub use moirai_protocol::state::sink::ObjectPath;
    pub use moirai_protocol::state::sink::PathSegment;
    pub use moirai_protocol::state::sink::SinkCollector;
    pub use moirai_protocol::state::sink::SinkEffect;
    pub use moirai_protocol::utils::intern_str::Resolver;
}
#[derive(Debug, Clone)]
pub enum ClassHierarchy {
    Package(__package::Package),
    AddReference(__package::Refs),
    RemoveReference(__package::Refs),
}
#[derive(Debug, Clone, Default)]
pub struct ClassHierarchyValue {
    pub package: __package::PackageValue,
    pub refs: <__package::ReferenceManager<__package::LwwPolicy> as __package::PureCRDT>::Value,
}
#[derive(Debug, Clone, Default)]
pub struct ClassHierarchyLog {
    package_log: __package::PackageLog,
    reference_manager_log: __package::ReferenceManagerLog,
}
impl ClassHierarchyLog {
    pub fn package_log(&self) -> &__package::PackageLog {
        &self.package_log
    }
    pub fn reference_manager_log(&self) -> &__package::ReferenceManagerLog {
        &self.reference_manager_log
    }
}

fn path_uses_resolver(path: &__package::ObjectPath, resolver: &__package::Resolver) -> bool {
    path.segments().iter().all(|segment| match segment {
        __package::PathSegment::ListElement(id) => id.resolver() == resolver,
        _ => true,
    })
}

fn refs_use_resolver(refs: &__package::Refs, resolver: &__package::Resolver) -> bool {
    match refs {
        __package::Refs::AttributeToClass(arc) => {
            path_uses_resolver(&arc.source.0, resolver)
                && path_uses_resolver(&arc.target.0, resolver)
        }
        __package::Refs::AttributeToDataType(arc) => {
            path_uses_resolver(&arc.source.0, resolver)
                && path_uses_resolver(&arc.target.0, resolver)
        }
        __package::Refs::ReferenceToReference(arc) => {
            path_uses_resolver(&arc.source.0, resolver)
                && path_uses_resolver(&arc.target.0, resolver)
        }
        __package::Refs::ReferenceToClass(arc) => {
            path_uses_resolver(&arc.source.0, resolver)
                && path_uses_resolver(&arc.target.0, resolver)
        }
        __package::Refs::ReferenceToDataType(arc) => {
            path_uses_resolver(&arc.source.0, resolver)
                && path_uses_resolver(&arc.target.0, resolver)
        }
        __package::Refs::ClassToClass(arc) => {
            path_uses_resolver(&arc.source.0, resolver)
                && path_uses_resolver(&arc.target.0, resolver)
        }
    }
}

impl __package::IsLog for ClassHierarchyLog {
    type Value = ClassHierarchyValue;
    type Op = ClassHierarchy;
    fn is_enabled(&self, op: &Self::Op) -> bool {
        match op {
            ClassHierarchy::Package(o) => self.package_log.is_enabled(o),
            ClassHierarchy::AddReference(o) => self
                .reference_manager_log
                .is_enabled(&__package::ReferenceManager::AddArc(o.clone())),
            ClassHierarchy::RemoveReference(o) => self
                .reference_manager_log
                .is_enabled(&__package::ReferenceManager::RemoveArc(o.clone())),
        }
    }
    fn effect(&mut self, event: __package::Event<Self::Op>) {
        let mut sink = __package::SinkCollector::new();
        match event.op().clone() {
            ClassHierarchy::Package(o) => __package::IsLogSink::effect_with_sink(
                &mut self.package_log,
                __package::Event::unfold(event.clone(), o),
                __package::ObjectPath::new("class_hierarchy").field("package"),
                &mut sink,
            ),
            ClassHierarchy::AddReference(o) => {
                debug_assert!(
                    refs_use_resolver(&o, event.id().resolver()),
                    "AddReference payload contains EventId values that were not internalized to the local resolver"
                );
                self.reference_manager_log.effect(__package::Event::unfold(
                    event.clone(),
                    __package::ReferenceManager::AddArc(o),
                ))
            }
            ClassHierarchy::RemoveReference(o) => {
                debug_assert!(
                    refs_use_resolver(&o, event.id().resolver()),
                    "RemoveReference payload contains EventId values that were not internalized to the local resolver"
                );
                self.reference_manager_log.effect(__package::Event::unfold(
                    event.clone(),
                    __package::ReferenceManager::RemoveArc(o),
                ))
            }
        }
        for sink in sink.into_sinks() {
            match sink.effect() {
                __package::SinkEffect::Create | __package::SinkEffect::Update => {
                    let vertex_ops = __package::instance_from_path(&sink.path())
                        .map(|instance| __package::ReferenceManager::AddVertex { id: instance });
                    if let Some(o) = vertex_ops {
                        self.reference_manager_log
                            .effect(__package::Event::unfold(event.clone(), o));
                    }
                }
                __package::SinkEffect::Delete => {
                    self.reference_manager_log.effect(__package::Event::unfold(
                        event.clone(),
                        __package::ReferenceManager::DeleteSubtree {
                            prefix: sink.path().clone(),
                        },
                    ));
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
        self.reference_manager_log
            .redundant_by_parent(version, conservative);
    }
    fn is_default(&self) -> bool {
        true && self.package_log.is_default()
    }
}
impl __package::EvalNested<__package::Read<<Self as __package::IsLog>::Value>>
    for ClassHierarchyLog
{
    fn execute_query(
        &self,
        _q: __package::Read<<Self as __package::IsLog>::Value>,
    ) -> <__package::Read<<Self as __package::IsLog>::Value> as __package::QueryOperation>::Response
    {
        ClassHierarchyValue {
            package: self.package_log.execute_query(__package::Read::new()),
            refs: self
                .reference_manager_log
                .execute_query(__package::Read::new()),
        }
    }
}
