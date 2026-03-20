/// Auto-generated code by 🅰🆁🅰🅲🅷🅽🅴 - do not edit directly
mod __references {
    pub use moirai_macros::typed_graph;
    pub use moirai_protocol::state::sink::ObjectPath;
    pub use moirai_protocol::state::sink::PathSegment::{Field, ListElement, MapEntry, Variant};
}
pub fn instance_from_path(path: &__references::ObjectPath) -> Option<Instance> {
    let segs = path.segments();
    match segs {
        [.., __references::Variant("class")] => Some(Instance::ClassId(ClassId(path.clone()))),
        [.., __references::Variant("attribute")] => {
            Some(Instance::AttributeId(AttributeId(path.clone())))
        }
        [.., __references::Variant("reference")] => {
            Some(Instance::ReferenceId(ReferenceId(path.clone())))
        }
        [.., __references::Variant("data_type")] => {
            Some(Instance::DataTypeId(DataTypeId(path.clone())))
        }
        _ => None,
    }
}
pub fn instance_path(instance: &Instance) -> &__references::ObjectPath {
    match instance {
        Instance::AttributeId(id) => &id.0,
        Instance::ReferenceId(id) => &id.0,
        Instance::ClassId(id) => &id.0,
        Instance::DataTypeId(id) => &id.0,
    }
}
pub fn deleted_prefix_instances(path: &__references::ObjectPath) -> Vec<Instance> {
    let mut instances = Vec::new();
    if let Some(instance) = instance_from_path(path) {
        instances.push(instance);
    }
    match path.segments() {
        [
            ..,
            __references::Field("content"),
            __references::ListElement(_),
        ] => {
            instances.push(Instance::ClassId(ClassId(
                path.clone().variant("classifier").variant("class"),
            )));
            instances.push(Instance::DataTypeId(DataTypeId(
                path.clone().variant("classifier").variant("data_type"),
            )));
            instances.push(Instance::AttributeId(AttributeId(
                path.clone()
                    .variant("structuralfeature")
                    .variant("attribute"),
            )));
            instances.push(Instance::ReferenceId(ReferenceId(
                path.clone()
                    .variant("structuralfeature")
                    .variant("reference"),
            )));
        }
        [
            ..,
            __references::Field("features"),
            __references::ListElement(_),
        ] => {
            instances.push(Instance::AttributeId(AttributeId(
                path.clone().variant("attribute"),
            )));
            instances.push(Instance::ReferenceId(ReferenceId(
                path.clone().variant("reference"),
            )));
        }
        _ => {}
    }
    instances.dedup();
    instances
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttributeTypEdge;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceOppositeEdge;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceTypEdge;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClassSupertypesEdge;
__references::typed_graph! {
    graph : ReferenceManager, vertex : Instance, edge : Ref, arcs_type : Refs, vertices {
    AttributeId, ReferenceId, ClassId, DataTypeId }, connections { AttributeToClass :
    AttributeId -> ClassId(AttributeTypEdge) [1, 1], AttributeToDataType : AttributeId ->
    DataTypeId(AttributeTypEdge) [1, 1], ReferenceToReference : ReferenceId ->
    ReferenceId(ReferenceOppositeEdge) [0, 1], ReferenceToClass : ReferenceId ->
    ClassId(ReferenceTypEdge) [1, 1], ReferenceToDataType : ReferenceId ->
    DataTypeId(ReferenceTypEdge) [1, 1], ClassToClass : ClassId ->
    ClassId(ClassSupertypesEdge) [0, *] }
}
