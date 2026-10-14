use deepsize::{Context, DeepSizeOf};
use moirai_crdt::list::nested_list::{NestedList, NestedListLog};
use moirai_fuzz::{
    generator::command_generator::CommandGenerator,
    observers::footprint::{FootprintNode, LogFootprint},
};
use moirai_protocol::{
    crdt::{eval::EvalNested, query::Read},
    state::log::{BoxedLog, IsLog},
};
use rand::{
    Rng, RngExt,
    distr::{Distribution, weighted::WeightedIndex},
    seq::{IndexedRandom, IteratorRandom},
};

use crate::{
    classifiers::*,
    package::{ClassHierarchy, ClassHierarchyLog},
    references::compute_arc_constraints,
};

impl DeepSizeOf for ClassHierarchy {
    fn deep_size_of_children(&self, context: &mut Context) -> usize {
        match self {
            Self::Package(op) => op.deep_size_of_children(context),
            Self::AddReference(op) | Self::RemoveReference(op) => op.deep_size_of_children(context),
        }
    }
}

impl DeepSizeOf for ClassHierarchyLog {
    fn deep_size_of_children(&self, context: &mut Context) -> usize {
        self.package_log().deep_size_of_children(context)
            + self.reference_manager_log().deep_size_of_children(context)
    }
}

impl LogFootprint for ClassHierarchyLog {
    fn footprint(&self) -> FootprintNode {
        FootprintNode::parent(
            self,
            vec![
                FootprintNode::leaf(self.package_log()),
                self.reference_manager_log().footprint(),
            ],
        )
    }
}

fn generate_boxed_model_element_kind(
    log: &BoxedLog<ModelElementKindLog>,
    rng: &mut impl Rng,
) -> Box<ModelElementKind> {
    log.generate_command(rng)
}

fn generate_structural_feature_kind(
    log: &StructuralFeatureKindLog,
    rng: &mut impl Rng,
) -> StructuralFeatureKind {
    log.generate_command(rng)
}

fn generate_boxed_model_list(
    log: &NestedListLog<BoxedLog<ModelElementKindLog>>,
    rng: &mut impl Rng,
) -> NestedList<Box<ModelElementKind>> {
    #[derive(Clone, Copy)]
    enum Choice {
        Insert,
        Update,
        Delete,
    }

    let positions = log.positions().execute_query(&Read::new());
    let choice = if positions.is_empty() {
        Choice::Insert
    } else {
        [Choice::Insert, Choice::Update, Choice::Delete]
            [WeightedIndex::new([1, 6, 2]).unwrap().sample(rng)]
    };

    let op = match choice {
        Choice::Insert => {
            let pos = rng.random_range(0..=positions.len());
            let op =
                generate_boxed_model_element_kind(&BoxedLog::<ModelElementKindLog>::default(), rng);
            NestedList::Insert { pos, op }
        }
        Choice::Update => {
            let pos = rng.random_range(0..positions.len());
            let target_id = &positions[pos];
            let op = log
                .children()
                .get_child(target_id)
                .map(|child| generate_boxed_model_element_kind(child, rng))
                .unwrap_or_else(|| {
                    generate_boxed_model_element_kind(
                        &BoxedLog::<ModelElementKindLog>::default(),
                        rng,
                    )
                });
            NestedList::Update { pos, op }
        }
        Choice::Delete => NestedList::Delete {
            pos: rng.random_range(0..positions.len()),
        },
    };

    assert!(log.is_enabled(&op).is_ok());
    op
}

fn generate_structural_feature_list(
    log: &NestedListLog<StructuralFeatureKindLog>,
    rng: &mut impl Rng,
) -> NestedList<StructuralFeatureKind> {
    #[derive(Clone, Copy)]
    enum Choice {
        Insert,
        Update,
        Delete,
    }

    let positions = log.positions().execute_query(&Read::new());
    let choice = if positions.is_empty() {
        Choice::Insert
    } else {
        [Choice::Insert, Choice::Update, Choice::Delete]
            [WeightedIndex::new([1, 6, 2]).unwrap().sample(rng)]
    };

    let op = match choice {
        Choice::Insert => {
            let pos = rng.random_range(0..=positions.len());
            let op = generate_structural_feature_kind(&StructuralFeatureKindLog::default(), rng);
            NestedList::Insert { pos, op }
        }
        Choice::Update => {
            let pos = rng.random_range(0..positions.len());
            let target_id = &positions[pos];
            let op = log
                .children()
                .get_child(target_id)
                .map(|child| generate_structural_feature_kind(child, rng))
                .unwrap_or_else(|| {
                    generate_structural_feature_kind(&StructuralFeatureKindLog::default(), rng)
                });
            NestedList::Update { pos, op }
        }
        Choice::Delete => NestedList::Delete {
            pos: rng.random_range(0..positions.len()),
        },
    };

    assert!(log.is_enabled(&op).is_ok());
    op
}

impl CommandGenerator for ClassHierarchyLog {
    fn generate_command(&self, rng: &mut impl Rng) -> Self::Command {
        #[derive(Clone, Copy)]
        enum Choice {
            Package,
            AddReference,
            RemoveReference,
        }

        let refs = self.reference_manager_log().execute_query(&Read::new());
        let constraints = compute_arc_constraints(&refs);

        let choice = if refs.node_count() < 2
            || (constraints.addable.is_empty() && constraints.removable.is_empty())
        {
            Choice::Package
        } else if constraints.removable.is_empty() {
            [Choice::Package, Choice::AddReference][WeightedIndex::new([8, 3]).unwrap().sample(rng)]
        } else if constraints.addable.is_empty() {
            [Choice::Package, Choice::RemoveReference]
                [WeightedIndex::new([8, 2]).unwrap().sample(rng)]
        } else {
            [
                Choice::Package,
                Choice::AddReference,
                Choice::RemoveReference,
            ][WeightedIndex::new([8, 3, 2]).unwrap().sample(rng)]
        };

        match choice {
            Choice::Package => ClassHierarchy::Package(self.package_log().generate_command(rng)),
            Choice::AddReference => ClassHierarchy::AddReference(
                constraints
                    .addable
                    .choose(rng)
                    .expect("addable references should not be empty")
                    .clone(),
            ),
            Choice::RemoveReference => ClassHierarchy::RemoveReference(
                constraints
                    .removable
                    .choose(rng)
                    .expect("removable references should not be empty")
                    .clone(),
            ),
        }
    }
}

impl CommandGenerator for PackageLog {
    fn generate_command(&self, rng: &mut impl Rng) -> Self::Command {
        #[derive(Clone, Copy)]
        enum Choice {
            ModelElement,
            Content,
        }

        let choice = if self
            .content()
            .positions()
            .execute_query(&Read::new())
            .is_empty()
        {
            [Choice::ModelElement, Choice::Content][WeightedIndex::new([1, 4]).unwrap().sample(rng)]
        } else {
            [Choice::ModelElement, Choice::Content][WeightedIndex::new([3, 2]).unwrap().sample(rng)]
        };

        match choice {
            Choice::ModelElement => {
                Package::ModelElementSuper(self.model_element_super().generate_command(rng))
            }
            Choice::Content => Package::Content(generate_boxed_model_list(self.content(), rng)),
        }
    }
}

impl CommandGenerator for ModelElementKindLog {
    fn generate_command(&self, rng: &mut impl Rng) -> Self::Command {
        match &self.child {
            ModelElementKindContainer::Unset => {
                match [0_u8, 1, 2][WeightedIndex::new([6, 1, 3]).unwrap().sample(rng)] {
                    0 => ModelElementKind::Classifier(
                        ClassifierKindLog::default().generate_command(rng),
                    ),
                    1 => ModelElementKind::Package(PackageLog::default().generate_command(rng)),
                    _ => ModelElementKind::StructuralFeature(
                        StructuralFeatureKindLog::default().generate_command(rng),
                    ),
                }
            }
            ModelElementKindContainer::Value(child) => match child.as_ref() {
                ModelElementKindChild::Classifier(log) => {
                    ModelElementKind::Classifier(log.generate_command(rng))
                }
                ModelElementKindChild::Package(log) => {
                    ModelElementKind::Package(log.generate_command(rng))
                }
                ModelElementKindChild::StructuralFeature(log) => {
                    ModelElementKind::StructuralFeature(log.generate_command(rng))
                }
            },
            ModelElementKindContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    ModelElementKindChild::Classifier(log) => {
                        ModelElementKind::Classifier(log.generate_command(rng))
                    }
                    ModelElementKindChild::Package(log) => {
                        ModelElementKind::Package(log.generate_command(rng))
                    }
                    ModelElementKindChild::StructuralFeature(log) => {
                        ModelElementKind::StructuralFeature(log.generate_command(rng))
                    }
                })
                .unwrap(),
        }
    }
}

impl CommandGenerator for ModelElementLog {
    fn generate_command(&self, rng: &mut impl Rng) -> Self::Command {
        ModelElement::Name(self.name().generate_command(rng))
    }
}

impl CommandGenerator for ClassifierKindLog {
    fn generate_command(&self, rng: &mut impl Rng) -> Self::Command {
        match &self.child {
            ClassifierKindContainer::Unset => {
                if rng.random_bool(0.65) {
                    ClassifierKind::Class(ClassLog::default().generate_command(rng))
                } else {
                    ClassifierKind::DataType(DataTypeLog::default().generate_command(rng))
                }
            }
            ClassifierKindContainer::Value(child) => match child.as_ref() {
                ClassifierKindChild::Class(log) => ClassifierKind::Class(log.generate_command(rng)),
                ClassifierKindChild::DataType(log) => {
                    ClassifierKind::DataType(log.generate_command(rng))
                }
            },
            ClassifierKindContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    ClassifierKindChild::Class(log) => {
                        ClassifierKind::Class(log.generate_command(rng))
                    }
                    ClassifierKindChild::DataType(log) => {
                        ClassifierKind::DataType(log.generate_command(rng))
                    }
                })
                .unwrap(),
        }
    }
}

impl CommandGenerator for ClassifierLog {
    fn generate_command(&self, rng: &mut impl Rng) -> Self::Command {
        Classifier::ModelElementSuper(self.model_element_super().generate_command(rng))
    }
}

impl CommandGenerator for StructuralFeatureKindLog {
    fn generate_command(&self, rng: &mut impl Rng) -> Self::Command {
        match &self.child {
            StructuralFeatureKindContainer::Unset => {
                if rng.random_bool(0.65) {
                    StructuralFeatureKind::Attribute(AttributeLog::default().generate_command(rng))
                } else {
                    StructuralFeatureKind::Reference(ReferenceLog::default().generate_command(rng))
                }
            }
            StructuralFeatureKindContainer::Value(child) => match child.as_ref() {
                StructuralFeatureKindChild::Attribute(log) => {
                    StructuralFeatureKind::Attribute(log.generate_command(rng))
                }
                StructuralFeatureKindChild::Reference(log) => {
                    StructuralFeatureKind::Reference(log.generate_command(rng))
                }
            },
            StructuralFeatureKindContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    StructuralFeatureKindChild::Attribute(log) => {
                        StructuralFeatureKind::Attribute(log.generate_command(rng))
                    }
                    StructuralFeatureKindChild::Reference(log) => {
                        StructuralFeatureKind::Reference(log.generate_command(rng))
                    }
                })
                .unwrap(),
        }
    }
}

impl CommandGenerator for StructuralFeatureLog {
    fn generate_command(&self, rng: &mut impl Rng) -> Self::Command {
        #[derive(Clone, Copy)]
        enum Choice {
            ModelElement,
            Lower,
            Upper,
            IsOrdered,
            IsUnique,
        }

        match [
            Choice::ModelElement,
            Choice::Lower,
            Choice::Upper,
            Choice::IsOrdered,
            Choice::IsUnique,
        ]
        .into_iter()
        .choose(rng)
        .unwrap()
        {
            Choice::ModelElement => StructuralFeature::ModelElementSuper(
                self.model_element_super().generate_command(rng),
            ),
            Choice::Lower => StructuralFeature::Lower(self.lower().generate_command(rng)),
            Choice::Upper => StructuralFeature::Upper(self.upper().generate_command(rng)),
            Choice::IsOrdered => {
                StructuralFeature::IsOrdered(self.is_ordered().generate_command(rng))
            }
            Choice::IsUnique => StructuralFeature::IsUnique(self.is_unique().generate_command(rng)),
        }
    }
}

impl CommandGenerator for AttributeLog {
    fn generate_command(&self, rng: &mut impl Rng) -> Self::Command {
        Attribute::StructuralFeatureSuper(self.structural_feature_super().generate_command(rng))
    }
}

impl CommandGenerator for ReferenceLog {
    fn generate_command(&self, rng: &mut impl Rng) -> Self::Command {
        #[derive(Clone, Copy)]
        enum Choice {
            StructuralFeature,
            IsContainer,
        }

        match [Choice::StructuralFeature, Choice::IsContainer]
            .into_iter()
            .choose(rng)
            .unwrap()
        {
            Choice::StructuralFeature => Reference::StructuralFeatureSuper(
                self.structural_feature_super().generate_command(rng),
            ),
            Choice::IsContainer => {
                Reference::IsContainer(self.is_container().generate_command(rng))
            }
        }
    }
}

impl CommandGenerator for ClassLog {
    fn generate_command(&self, rng: &mut impl Rng) -> Self::Command {
        #[derive(Clone, Copy)]
        enum Choice {
            ClassifierKind,
            IsAbstract,
            Features,
        }

        match [Choice::ClassifierKind, Choice::IsAbstract, Choice::Features]
            [WeightedIndex::new([3, 3, 1]).unwrap().sample(rng)]
        {
            Choice::ClassifierKind => {
                Class::ClassifierSuper(self.classifier_super().generate_command(rng))
            }
            Choice::IsAbstract => Class::IsAbstract(self.is_abstract().generate_command(rng)),
            Choice::Features => {
                Class::Features(generate_structural_feature_list(self.features(), rng))
            }
        }
    }
}

impl CommandGenerator for DataTypeLog {
    fn generate_command(&self, rng: &mut impl Rng) -> Self::Command {
        DataType::ClassifierSuper(self.classifier_super().generate_command(rng))
    }
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::SmallRng};

    use super::*;

    #[test]
    fn generates_boxed_package_content_operation() {
        let package = PackageLog::default();
        let mut rng = SmallRng::seed_from_u64(0);

        let op = generate_boxed_model_list(package.content(), &mut rng);

        assert!(matches!(op, NestedList::Insert { .. }));
        assert!(package.content().is_enabled(&op).is_ok());
    }
}
