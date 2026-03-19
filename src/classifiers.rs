/// Auto-generated code by 🅰🆁🅰🅲🅷🅽🅴 - do not edit directly
mod __classifiers {
    pub use moirai_macros::record;
    pub use moirai_macros::union;
    pub use moirai_protocol::state::event_graph::EventGraph;
    pub use moirai_crdt::list::eg_walker::List;
    pub use moirai_crdt::list::nested_list::NestedListLog;
    pub use moirai_protocol::state::po_log::VecLog;
    pub use moirai_crdt::counter::resettable_counter::Counter;
    pub use moirai_crdt::flag::ew_flag::EWFlag;
}
__classifiers::union!(
    ModelElement = Classifier(Classifier, ClassifierLog) | Package(Package, PackageLog) |
    StructuralFeature(StructuralFeature, StructuralFeatureLog)
);
__classifiers::record!(
    ModelElementFeat { name : __classifiers::EventGraph < __classifiers::List < char > >,
    }
);
__classifiers::union!(
    Classifier = Class(Class, ClassLog) | DataType(DataType, DataTypeLog)
);
__classifiers::record!(ClassifierFeat { model_element_feat : ModelElementFeatLog, });
__classifiers::record!(
    Package { model_element_feat : ModelElementFeatLog, content :
    __classifiers::NestedListLog < Box < ModelElementLog > >, }
);
__classifiers::union!(
    StructuralFeature = Attribute(Attribute, AttributeLog) | Reference(Reference,
    ReferenceLog)
);
__classifiers::record!(
    StructuralFeatureFeat { model_element_feat : ModelElementFeatLog, lower :
    __classifiers::VecLog < __classifiers::Counter < i32 > >, upper :
    __classifiers::VecLog < __classifiers::Counter < i32 > >, is_ordered :
    __classifiers::VecLog < __classifiers::EWFlag >, is_unique : __classifiers::VecLog <
    __classifiers::Counter < i32 > >, }
);
__classifiers::record!(
    Attribute { structural_feature_feat : StructuralFeatureFeatLog, }
);
__classifiers::record!(
    Reference { structural_feature_feat : StructuralFeatureFeatLog, is_container :
    __classifiers::VecLog < __classifiers::EWFlag >, }
);
__classifiers::record!(
    Class { classifier_feat : ClassifierFeatLog, is_abstract : __classifiers::VecLog <
    __classifiers::EWFlag >, features : __classifiers::NestedListLog <
    StructuralFeatureLog >, }
);
__classifiers::record!(DataType { classifier_feat : ClassifierFeatLog, });
