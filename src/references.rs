/// Auto-generated code by 🅰🆁🅰🅲🅷🅽🅴 - do not edit directly
mod __references {
    pub use moirai_macros::typed_graph;
    pub use moirai_protocol::state::sink::ObjectPath;
    pub use moirai_protocol::state::sink::PathSegment::{Field, ListElement, MapEntry, Variant};
}
pub fn instance_from_path(path: &__references::ObjectPath) -> Option<Instance> {
    let segs = path.segments();
    match segs {
        [.., __references::Field("structural_feature_feat")] => Some(
            Instance::StructuralFeatureId(StructuralFeatureId(path.clone())),
        ),
        [.., __references::Field("classifier_feat")] => {
            Some(Instance::ClassifierId(ClassifierId(path.clone())))
        }
        [
            ..,
            __references::Field("content"),
            __references::ListElement(_),
            __references::Variant("classifier"),
            __references::Variant("class"),
        ] => Some(Instance::ClassId(ClassId(path.clone()))),
        [.., __references::Variant("reference")] => {
            Some(Instance::ReferenceId(ReferenceId(path.clone())))
        }
        _ => None,
    }
}

pub fn instance_path(instance: &Instance) -> &__references::ObjectPath {
    match instance {
        Instance::ClassifierId(id) => &id.0,
        Instance::StructuralFeatureId(id) => &id.0,
        Instance::ReferenceId(id) => &id.0,
        Instance::ClassId(id) => &id.0,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClassifierId(pub __references::ObjectPath);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructuralFeatureId(pub __references::ObjectPath);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceId(pub __references::ObjectPath);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClassId(pub __references::ObjectPath);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructuralFeatureTypEdge;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceOppositeEdge;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClassSupertypesEdge;
__references::typed_graph! {
    graph : ReferenceManager, vertex : Instance, edge : Ref, arcs_type : Refs, vertices {
    ClassifierId, StructuralFeatureId, ReferenceId, ClassId }, connections {
    StructuralFeatureTyp : StructuralFeatureId -> ClassifierId(StructuralFeatureTypEdge)
    [1, 1], ReferenceOpposite : ReferenceId -> ReferenceId(ReferenceOppositeEdge) [0, 1],
    ClassSupertypes : ClassId -> ClassId(ClassSupertypesEdge) [0, *] }
}
