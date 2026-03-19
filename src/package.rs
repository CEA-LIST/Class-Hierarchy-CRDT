use moirai_protocol::state::sink::SinkEffect;

use crate::references::{Instance, instance_from_path, instance_path};

/// Auto-generated code by 🅰🆁🅰🅲🅷🅽🅴 - do not edit directly
mod __package {
    pub use crate::classifiers::*;
    pub use crate::references::*;
    pub use moirai_crdt::policy::LwwPolicy;
    pub use moirai_protocol::clock::version_vector::Version;
    pub use moirai_protocol::crdt::eval::EvalNested;
    pub use moirai_protocol::crdt::pure_crdt::PureCRDT;
    pub use moirai_protocol::crdt::query::QueryOperation;
    pub use moirai_protocol::crdt::query::Read;
    pub use moirai_protocol::event::Event;
    pub use moirai_protocol::state::log::IsLog;
    pub use moirai_protocol::state::po_log::VecLog;
    pub use moirai_protocol::state::sink::IsLogSink;
    pub use moirai_protocol::state::sink::ObjectPath;
    pub use moirai_protocol::state::sink::SinkCollector;
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
    reference_manager_log: __package::VecLog<__package::ReferenceManager<__package::LwwPolicy>>,
}
impl ClassHierarchyLog {
    pub fn package_log(&self) -> &__package::PackageLog {
        &self.package_log
    }
    pub fn reference_manager_log(
        &self,
    ) -> &__package::VecLog<__package::ReferenceManager<__package::LwwPolicy>> {
        &self.reference_manager_log
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
            ClassHierarchy::AddReference(o) => self.reference_manager_log.effect(
                __package::Event::unfold(event.clone(), __package::ReferenceManager::AddArc(o)),
            ),
            ClassHierarchy::RemoveReference(o) => self.reference_manager_log.effect(
                __package::Event::unfold(event.clone(), __package::ReferenceManager::RemoveArc(o)),
            ),
        }
        println!("Applied event: {:?}", event.op());
        for sink in sink.into_sinks() {
            match sink.effect() {
                SinkEffect::Create | SinkEffect::Update => {
                    let vertex_ops = instance_from_path(&sink.path())
                        .map(|instance| __package::ReferenceManager::AddVertex { id: instance });
                    if let Some(o) = vertex_ops {
                        println!("Adding vertex: {} ({:?})", sink.path(), sink.effect());
                        self.reference_manager_log
                            .effect(__package::Event::unfold(event.clone(), o));
                    }
                }
                SinkEffect::Delete => {
                    let graph = self.reference_manager_log.eval(__package::Read::new());
                    let removals = graph
                        .node_weights()
                        .filter(|n| sink.path().is_prefix_of(instance_path(n)))
                        .collect::<Vec<_>>();
                    println!("Removing vertex:");
                    for removal in removals {
                        match removal {
                            Instance::ClassifierId(id) => {
                                println!("      {} (ClassifierId)", id.0)
                            }
                            Instance::StructuralFeatureId(id) => {
                                println!("      {} (StructuralFeatureId)", id.0)
                            }
                            Instance::ReferenceId(id) => println!("      {} (ReferenceId)", id.0),
                            Instance::ClassId(id) => println!("      {} (ClassId)", id.0),
                        }
                        let removal_event = __package::Event::unfold(
                            event.clone(),
                            __package::ReferenceManager::RemoveVertex {
                                id: removal.clone(),
                            },
                        );
                        self.reference_manager_log.effect(removal_event);
                    }
                }
            }
        }
        println!("------------");
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
