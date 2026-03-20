use moirai_protocol::{
    replica::ReplicaIdx, utils::intern_str::Interner, utils::translate_ids::TranslateIds,
};

use crate::package::ClassHierarchy;

impl TranslateIds for ClassHierarchy {
    fn translate_ids(&self, from: ReplicaIdx, interner: &Interner) -> Self {
        match self {
            ClassHierarchy::Package(op) => ClassHierarchy::Package(op.clone()),
            ClassHierarchy::AddReference(op) => {
                ClassHierarchy::AddReference(op.translate_ids(from, interner))
            }
            ClassHierarchy::RemoveReference(op) => {
                ClassHierarchy::RemoveReference(op.translate_ids(from, interner))
            }
        }
    }
}
