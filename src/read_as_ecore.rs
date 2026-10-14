/// Auto-generated code by 🅰🆁🅰🅲🅷🅽🅴 - do not edit directly
mod __read_as_ecore {
    pub use crate::classifiers::*;
    pub use crate::package::*;
    pub use crate::references::*;
    pub use arachne_xmi::ReferenceIndex as XmiReferenceIndex;
    pub use arachne_xmi::{Attributes as XmiAttributes, Writer as XmiWriter};
    pub use moirai_protocol::crdt::eval::EvalNested;
    pub use moirai_protocol::crdt::query::QueryOperation;
    pub use moirai_protocol::crdt::query::Read;
    pub use moirai_protocol::event::id::EventId;
    pub use moirai_protocol::state::log::IsLog;
    pub use moirai_protocol::state::object_path::ObjectPath;
}
/// Serializes the current replicated model as XMI conforming to the
/// source Ecore package.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReadAsEcore;
impl __read_as_ecore::QueryOperation for ReadAsEcore {
    type Response = Vec<u8>;
}
impl ReadAsEcore {
    pub fn new() -> Self {
        Self
    }
}
fn xmi_read<L, V>(log: &L) -> V
where
    L: __read_as_ecore::IsLog + __read_as_ecore::EvalNested<__read_as_ecore::Read<V>>,
{
    log.execute_query(&__read_as_ecore::Read::<V>::new())
}
type XmiReferences = __read_as_ecore::XmiReferenceIndex<__read_as_ecore::ObjectPath>;
fn build_xmi_references(
    refs: &petgraph::graph::DiGraph<__read_as_ecore::Instance, __read_as_ecore::Ref>,
) -> XmiReferences {
    let mut values = XmiReferences::default();
    use petgraph::visit::EdgeRef as _;
    for edge in refs.edge_references() {
        let source = &refs[edge.source()];
        let target = &refs[edge.target()];
        match (source, target, edge.weight()) {
            (
                __read_as_ecore::Instance::AttributeId(source),
                __read_as_ecore::Instance::ClassId(target),
                __read_as_ecore::Ref::AttributeToClass(_),
            ) => {
                values.insert(source.0.clone(), "structural_feature::typ", &target.0);
            }
            (
                __read_as_ecore::Instance::AttributeId(source),
                __read_as_ecore::Instance::DataTypeId(target),
                __read_as_ecore::Ref::AttributeToDataType(_),
            ) => {
                values.insert(source.0.clone(), "structural_feature::typ", &target.0);
            }
            (
                __read_as_ecore::Instance::ReferenceId(source),
                __read_as_ecore::Instance::ClassId(target),
                __read_as_ecore::Ref::ReferenceToClass(_),
            ) => {
                values.insert(source.0.clone(), "structural_feature::typ", &target.0);
            }
            (
                __read_as_ecore::Instance::ReferenceId(source),
                __read_as_ecore::Instance::DataTypeId(target),
                __read_as_ecore::Ref::ReferenceToDataType(_),
            ) => {
                values.insert(source.0.clone(), "structural_feature::typ", &target.0);
            }
            (
                __read_as_ecore::Instance::ReferenceId(source),
                __read_as_ecore::Instance::ReferenceId(target),
                __read_as_ecore::Ref::ReferenceToReference(_),
            ) => {
                values.insert(source.0.clone(), "reference::opposite", &target.0);
            }
            (
                __read_as_ecore::Instance::ClassId(source),
                __read_as_ecore::Instance::ClassId(target),
                __read_as_ecore::Ref::ClassToClass(_),
            ) => {
                values.insert(source.0.clone(), "class::supertypes", &target.0);
            }
            _ => {}
        }
    }
    values
}
#[allow(dead_code, unused_variables)]
fn visit_package(
    writer: &mut __read_as_ecore::XmiWriter,
    element_name: &str,
    xmi_type: Option<&str>,
    path: __read_as_ecore::ObjectPath,
    log: &__read_as_ecore::PackageLog,
    refs: &XmiReferences,
) {
    let mut attrs = __read_as_ecore::XmiAttributes::for_object(&path);
    if let Some(xmi_type) = xmi_type {
        attrs.push_value("xsi:type", xmi_type);
    }
    {
        let value = xmi_read::<_, std::string::String>(((log).model_element_super()).name());
        attrs.push_values("name", vec![(value).to_string()], true);
    }
    writer.element(element_name, &attrs, |writer| {
        let list_base_path = path.clone().field("content");
        let positions = xmi_read::<_, Vec<__read_as_ecore::EventId>>(((log).content()).positions());
        for event_id in &positions {
            if let Some(child) = ((log).content()).children().get_child(event_id) {
                let child_path = list_base_path.clone().list_element(event_id.clone());
                visit_model_element_kind(
                    writer,
                    Some("content"),
                    child_path,
                    (child).inner(),
                    refs,
                );
            }
        }
    });
}
#[allow(dead_code, unused_variables)]
fn visit_attribute(
    writer: &mut __read_as_ecore::XmiWriter,
    element_name: &str,
    xmi_type: Option<&str>,
    path: __read_as_ecore::ObjectPath,
    log: &__read_as_ecore::AttributeLog,
    refs: &XmiReferences,
) {
    let mut attrs = __read_as_ecore::XmiAttributes::for_object(&path);
    if let Some(xmi_type) = xmi_type {
        attrs.push_value("xsi:type", xmi_type);
    }
    {
        let value = xmi_read::<_, std::string::String>(
            (((log).structural_feature_super()).model_element_super()).name(),
        );
        attrs.push_values("name", vec![(value).to_string()], true);
    }
    {
        let value = xmi_read::<_, i32>(((log).structural_feature_super()).lower());
        attrs.push_values("lower", vec![(value).to_string()], true);
    }
    {
        let value = xmi_read::<_, i32>(((log).structural_feature_super()).upper());
        attrs.push_values("upper", vec![(value).to_string()], true);
    }
    {
        let value = xmi_read::<_, bool>(((log).structural_feature_super()).is_ordered());
        attrs.push_values("isOrdered", vec![(value).to_string()], true);
    }
    {
        let value = xmi_read::<_, bool>(((log).structural_feature_super()).is_unique());
        attrs.push_values("isUnique", vec![(value).to_string()], true);
    }
    attrs.push_values("typ", refs.values(&path, "structural_feature::typ"), false);
    writer.element(element_name, &attrs, |writer| {});
}
#[allow(dead_code, unused_variables)]
fn visit_reference(
    writer: &mut __read_as_ecore::XmiWriter,
    element_name: &str,
    xmi_type: Option<&str>,
    path: __read_as_ecore::ObjectPath,
    log: &__read_as_ecore::ReferenceLog,
    refs: &XmiReferences,
) {
    let mut attrs = __read_as_ecore::XmiAttributes::for_object(&path);
    if let Some(xmi_type) = xmi_type {
        attrs.push_value("xsi:type", xmi_type);
    }
    {
        let value = xmi_read::<_, std::string::String>(
            (((log).structural_feature_super()).model_element_super()).name(),
        );
        attrs.push_values("name", vec![(value).to_string()], true);
    }
    {
        let value = xmi_read::<_, i32>(((log).structural_feature_super()).lower());
        attrs.push_values("lower", vec![(value).to_string()], true);
    }
    {
        let value = xmi_read::<_, i32>(((log).structural_feature_super()).upper());
        attrs.push_values("upper", vec![(value).to_string()], true);
    }
    {
        let value = xmi_read::<_, bool>(((log).structural_feature_super()).is_ordered());
        attrs.push_values("isOrdered", vec![(value).to_string()], true);
    }
    {
        let value = xmi_read::<_, bool>(((log).structural_feature_super()).is_unique());
        attrs.push_values("isUnique", vec![(value).to_string()], true);
    }
    attrs.push_values("typ", refs.values(&path, "structural_feature::typ"), false);
    {
        let value = xmi_read::<_, bool>((log).is_container());
        attrs.push_values("isContainer", vec![(value).to_string()], true);
    }
    attrs.push_values("opposite", refs.values(&path, "reference::opposite"), false);
    writer.element(element_name, &attrs, |writer| {});
}
#[allow(dead_code, unused_variables)]
fn visit_class(
    writer: &mut __read_as_ecore::XmiWriter,
    element_name: &str,
    xmi_type: Option<&str>,
    path: __read_as_ecore::ObjectPath,
    log: &__read_as_ecore::ClassLog,
    refs: &XmiReferences,
) {
    let mut attrs = __read_as_ecore::XmiAttributes::for_object(&path);
    if let Some(xmi_type) = xmi_type {
        attrs.push_value("xsi:type", xmi_type);
    }
    {
        let value = xmi_read::<_, std::string::String>(
            (((log).classifier_super()).model_element_super()).name(),
        );
        attrs.push_values("name", vec![(value).to_string()], true);
    }
    {
        let value = xmi_read::<_, bool>((log).is_abstract());
        attrs.push_values("isAbstract", vec![(value).to_string()], true);
    }
    attrs.push_values("supertypes", refs.values(&path, "class::supertypes"), false);
    writer.element(element_name, &attrs, |writer| {
        let list_base_path = path.clone().field("features");
        let positions =
            xmi_read::<_, Vec<__read_as_ecore::EventId>>(((log).features()).positions());
        for event_id in &positions {
            if let Some(child) = ((log).features()).children().get_child(event_id) {
                let child_path = list_base_path.clone().list_element(event_id.clone());
                visit_structural_feature_kind(writer, Some("features"), child_path, child, refs);
            }
        }
    });
}
#[allow(dead_code, unused_variables)]
fn visit_data_type(
    writer: &mut __read_as_ecore::XmiWriter,
    element_name: &str,
    xmi_type: Option<&str>,
    path: __read_as_ecore::ObjectPath,
    log: &__read_as_ecore::DataTypeLog,
    refs: &XmiReferences,
) {
    let mut attrs = __read_as_ecore::XmiAttributes::for_object(&path);
    if let Some(xmi_type) = xmi_type {
        attrs.push_value("xsi:type", xmi_type);
    }
    {
        let value = xmi_read::<_, std::string::String>(
            (((log).classifier_super()).model_element_super()).name(),
        );
        attrs.push_values("name", vec![(value).to_string()], true);
    }
    writer.element(element_name, &attrs, |writer| {});
}
#[allow(dead_code, unused_variables)]
fn visit_model_element_kind(
    writer: &mut __read_as_ecore::XmiWriter,
    element_name: Option<&str>,
    path: __read_as_ecore::ObjectPath,
    log: &__read_as_ecore::ModelElementKindLog,
    refs: &XmiReferences,
) {
    let visit_child = |child: &__read_as_ecore::ModelElementKindChild,
                       writer: &mut __read_as_ecore::XmiWriter| {
        match child {
            __read_as_ecore::ModelElementKindChild::Classifier(child_log) => {
                visit_classifier_kind(
                    writer,
                    element_name,
                    path.clone().variant("classifier"),
                    child_log,
                    refs,
                );
            }
            __read_as_ecore::ModelElementKindChild::Package(child_log) => {
                let (element_name, xmi_type) = match element_name {
                    Some(name) => (name, Some("class_hierarchy:Package")),
                    None => ("class_hierarchy:Package", None),
                };
                visit_package(
                    writer,
                    element_name,
                    xmi_type,
                    path.clone().variant("package"),
                    child_log,
                    refs,
                );
            }
            __read_as_ecore::ModelElementKindChild::StructuralFeature(child_log) => {
                visit_structural_feature_kind(
                    writer,
                    element_name,
                    path.clone().variant("structuralfeature"),
                    child_log,
                    refs,
                );
            }
        }
    };
    let child_rank = |child: &__read_as_ecore::ModelElementKindChild| match child {
        __read_as_ecore::ModelElementKindChild::Classifier(_) => 0usize,
        __read_as_ecore::ModelElementKindChild::Package(_) => 1usize,
        __read_as_ecore::ModelElementKindChild::StructuralFeature(_) => 2usize,
    };
    match &log.child {
        __read_as_ecore::ModelElementKindContainer::Unset => {}
        __read_as_ecore::ModelElementKindContainer::Value(child) => {
            visit_child(child.as_ref(), writer);
        }
        __read_as_ecore::ModelElementKindContainer::Conflicts(children) => {
            let mut children = children.iter().collect::<Vec<_>>();
            children.sort_by_key(|child| child_rank(child));
            for child in children {
                visit_child(child, writer);
            }
        }
    }
}
#[allow(dead_code, unused_variables)]
fn visit_classifier_kind(
    writer: &mut __read_as_ecore::XmiWriter,
    element_name: Option<&str>,
    path: __read_as_ecore::ObjectPath,
    log: &__read_as_ecore::ClassifierKindLog,
    refs: &XmiReferences,
) {
    let visit_child = |child: &__read_as_ecore::ClassifierKindChild,
                       writer: &mut __read_as_ecore::XmiWriter| {
        match child {
            __read_as_ecore::ClassifierKindChild::Class(child_log) => {
                let (element_name, xmi_type) = match element_name {
                    Some(name) => (name, Some("class_hierarchy:Class")),
                    None => ("class_hierarchy:Class", None),
                };
                visit_class(
                    writer,
                    element_name,
                    xmi_type,
                    path.clone().variant("class"),
                    child_log,
                    refs,
                );
            }
            __read_as_ecore::ClassifierKindChild::DataType(child_log) => {
                let (element_name, xmi_type) = match element_name {
                    Some(name) => (name, Some("class_hierarchy:DataType")),
                    None => ("class_hierarchy:DataType", None),
                };
                visit_data_type(
                    writer,
                    element_name,
                    xmi_type,
                    path.clone().variant("datatype"),
                    child_log,
                    refs,
                );
            }
        }
    };
    let child_rank = |child: &__read_as_ecore::ClassifierKindChild| match child {
        __read_as_ecore::ClassifierKindChild::Class(_) => 0usize,
        __read_as_ecore::ClassifierKindChild::DataType(_) => 1usize,
    };
    match &log.child {
        __read_as_ecore::ClassifierKindContainer::Unset => {}
        __read_as_ecore::ClassifierKindContainer::Value(child) => {
            visit_child(child.as_ref(), writer);
        }
        __read_as_ecore::ClassifierKindContainer::Conflicts(children) => {
            let mut children = children.iter().collect::<Vec<_>>();
            children.sort_by_key(|child| child_rank(child));
            for child in children {
                visit_child(child, writer);
            }
        }
    }
}
#[allow(dead_code, unused_variables)]
fn visit_structural_feature_kind(
    writer: &mut __read_as_ecore::XmiWriter,
    element_name: Option<&str>,
    path: __read_as_ecore::ObjectPath,
    log: &__read_as_ecore::StructuralFeatureKindLog,
    refs: &XmiReferences,
) {
    let visit_child = |child: &__read_as_ecore::StructuralFeatureKindChild,
                       writer: &mut __read_as_ecore::XmiWriter| {
        match child {
            __read_as_ecore::StructuralFeatureKindChild::Attribute(child_log) => {
                let (element_name, xmi_type) = match element_name {
                    Some(name) => (name, Some("class_hierarchy:Attribute")),
                    None => ("class_hierarchy:Attribute", None),
                };
                visit_attribute(
                    writer,
                    element_name,
                    xmi_type,
                    path.clone().variant("attribute"),
                    child_log,
                    refs,
                );
            }
            __read_as_ecore::StructuralFeatureKindChild::Reference(child_log) => {
                let (element_name, xmi_type) = match element_name {
                    Some(name) => (name, Some("class_hierarchy:Reference")),
                    None => ("class_hierarchy:Reference", None),
                };
                visit_reference(
                    writer,
                    element_name,
                    xmi_type,
                    path.clone().variant("reference"),
                    child_log,
                    refs,
                );
            }
        }
    };
    let child_rank = |child: &__read_as_ecore::StructuralFeatureKindChild| match child {
        __read_as_ecore::StructuralFeatureKindChild::Attribute(_) => 0usize,
        __read_as_ecore::StructuralFeatureKindChild::Reference(_) => 1usize,
    };
    match &log.child {
        __read_as_ecore::StructuralFeatureKindContainer::Unset => {}
        __read_as_ecore::StructuralFeatureKindContainer::Value(child) => {
            visit_child(child.as_ref(), writer);
        }
        __read_as_ecore::StructuralFeatureKindContainer::Conflicts(children) => {
            let mut children = children.iter().collect::<Vec<_>>();
            children.sort_by_key(|child| child_rank(child));
            for child in children {
                visit_child(child, writer);
            }
        }
    }
}
impl __read_as_ecore::EvalNested<ReadAsEcore> for __read_as_ecore::ClassHierarchyLog {
    fn execute_query(
        &self,
        _q: &ReadAsEcore,
    ) -> <ReadAsEcore as __read_as_ecore::QueryOperation>::Response {
        let refs = build_xmi_references(&xmi_read(self.reference_manager_log()));
        let mut writer = __read_as_ecore::XmiWriter::new(
            "class_hierarchy",
            "http://www.example.org/class_hierarchy",
        );
        visit_package(
            &mut writer,
            "class_hierarchy:Package",
            None,
            __read_as_ecore::ObjectPath::new("class_hierarchy").field("package"),
            self.package_log(),
            &refs,
        );
        writer.finish()
    }
}
