/// Auto-generated code by 🅰🆁🅰🅲🅷🅽🅴 - do not edit directly
mod __classifiers {
    pub use moirai_crdt::counter::resettable_counter::Counter;
    pub use moirai_crdt::flag::ew_flag::EWFlag;
    pub use moirai_crdt::list::eg_walker::List;
    pub use moirai_crdt::list::nested_list::NestedListLog;
    pub use moirai_macros::record;
    pub use moirai_macros::union;
    pub use moirai_protocol::state::event_graph::EventGraph;
    pub use moirai_protocol::state::po_log::VecLog;
}
__classifiers::union!(
    ModelElementKind = Classifier(ClassifierKind, ClassifierKindLog)
        | Package(Package, PackageLog)
        | StructuralFeature(StructuralFeatureKind, StructuralFeatureKindLog)
);
__classifiers::record!(
    ModelElement {
        name : __classifiers::EventGraph<__classifiers::List<char>>,
    }
);
__classifiers::union!(ClassifierKind = Class(Class, ClassLog) | DataType(DataType, DataTypeLog));
__classifiers::record!(Classifier {
    model_element_super: ModelElementLog,
});
__classifiers::record!(
    Package {
        model_element_super: ModelElementLog,
        content: __classifiers::NestedListLog<Box<ModelElementKindLog>>,
    }
);
__classifiers::union!(
    StructuralFeatureKind = Attribute(Attribute, AttributeLog) | Reference(Reference, ReferenceLog)
);
__classifiers::record!(
    StructuralFeature {
        model_element_super: ModelElementLog,
        lower: __classifiers::VecLog<__classifiers::Counter<i32>>,
        upper: __classifiers::VecLog<__classifiers::Counter<i32>>,
        is_ordered: __classifiers::VecLog<__classifiers::EWFlag >,
        is_unique: __classifiers::VecLog<__classifiers::EWFlag>,
    }
);
__classifiers::record!(Attribute {
    structural_feature_super: StructuralFeatureLog,
});
__classifiers::record!(
    Reference {
        structural_feature_super: StructuralFeatureLog,
        is_container: __classifiers::VecLog<__classifiers::EWFlag>,
    }
);
__classifiers::record!(
    Class {
        classifier_super: ClassifierLog,
        is_abstract: __classifiers::VecLog <__classifiers::EWFlag>,
        features: __classifiers::NestedListLog<StructuralFeatureKindLog>,
    }
);
__classifiers::record!(DataType {
    classifier_super: ClassifierLog,
});
