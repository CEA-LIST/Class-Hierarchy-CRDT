use moirai_crdt::list::nested_list::{NestedList, NestedListLog};
use moirai_fuzz::{
    metrics::{FuzzMetrics, StructureMetrics},
    op_generator::OpGeneratorNested,
};
use moirai_protocol::{
    crdt::{eval::EvalNested, query::Read},
    state::log::IsLog,
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

fn generate_boxed_model_element_kind(
    log: &ModelElementKindLog,
    rng: &mut impl Rng,
) -> Box<ModelElementKind> {
    Box::new((*log).generate(rng))
}

fn generate_structural_feature_kind(
    log: &StructuralFeatureKindLog,
    rng: &mut impl Rng,
) -> StructuralFeatureKind {
    log.generate(rng)
}

fn generate_boxed_model_list(
    log: &NestedListLog<Box<ModelElementKindLog>>,
    rng: &mut impl Rng,
) -> NestedList<Box<ModelElementKind>> {
    #[derive(Clone, Copy)]
    enum Choice {
        Insert,
        Update,
        Delete,
    }

    let positions = log.positions().execute_query(Read::new());
    let choice = if positions.is_empty() {
        Choice::Insert
    } else {
        [Choice::Insert, Choice::Update, Choice::Delete]
            [WeightedIndex::new([1, 6, 2]).unwrap().sample(rng)]
    };

    let op = match choice {
        Choice::Insert => {
            let pos = rng.random_range(0..=positions.len());
            let op = generate_boxed_model_element_kind(&Box::<ModelElementKindLog>::default(), rng);
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
                    generate_boxed_model_element_kind(&Box::<ModelElementKindLog>::default(), rng)
                });
            NestedList::Update { pos, op }
        }
        Choice::Delete => NestedList::Delete {
            pos: rng.random_range(0..positions.len()),
        },
    };

    assert!(log.is_enabled(&op));
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

    let positions = log.positions().execute_query(Read::new());
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

    assert!(log.is_enabled(&op));
    op
}

impl OpGeneratorNested for ClassHierarchyLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            Package,
            AddReference,
            RemoveReference,
        }

        let refs = self.reference_manager_log().execute_query(Read::new());
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
            Choice::Package => ClassHierarchy::Package(self.package_log().generate(rng)),
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

impl OpGeneratorNested for PackageLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            ModelElement,
            Content,
        }

        let choice = if self
            .content()
            .positions()
            .execute_query(Read::new())
            .is_empty()
        {
            [Choice::ModelElement, Choice::Content][WeightedIndex::new([1, 4]).unwrap().sample(rng)]
        } else {
            [Choice::ModelElement, Choice::Content][WeightedIndex::new([3, 2]).unwrap().sample(rng)]
        };

        match choice {
            Choice::ModelElement => {
                Package::ModelElementSuper(self.model_element_super().generate(rng))
            }
            Choice::Content => Package::Content(generate_boxed_model_list(self.content(), rng)),
        }
    }
}

impl OpGeneratorNested for ModelElementKindLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        match &self.child {
            ModelElementKindContainer::Unset => {
                match [0_u8, 1, 2][WeightedIndex::new([6, 1, 3]).unwrap().sample(rng)] {
                    0 => ModelElementKind::Classifier(ClassifierKindLog::default().generate(rng)),
                    1 => ModelElementKind::Package(PackageLog::default().generate(rng)),
                    _ => ModelElementKind::StructuralFeature(
                        StructuralFeatureKindLog::default().generate(rng),
                    ),
                }
            }
            ModelElementKindContainer::Value(child) => match child.as_ref() {
                ModelElementKindChild::Classifier(log) => {
                    ModelElementKind::Classifier(log.generate(rng))
                }
                ModelElementKindChild::Package(log) => ModelElementKind::Package(log.generate(rng)),
                ModelElementKindChild::StructuralFeature(log) => {
                    ModelElementKind::StructuralFeature(log.generate(rng))
                }
            },
            ModelElementKindContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    ModelElementKindChild::Classifier(log) => {
                        ModelElementKind::Classifier(log.generate(rng))
                    }
                    ModelElementKindChild::Package(log) => {
                        ModelElementKind::Package(log.generate(rng))
                    }
                    ModelElementKindChild::StructuralFeature(log) => {
                        ModelElementKind::StructuralFeature(log.generate(rng))
                    }
                })
                .unwrap(),
        }
    }
}

impl OpGeneratorNested for ModelElementLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        ModelElement::Name(self.name().generate(rng))
    }
}

impl OpGeneratorNested for ClassifierKindLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        match &self.child {
            ClassifierKindContainer::Unset => {
                if rng.random_bool(0.65) {
                    ClassifierKind::Class(ClassLog::default().generate(rng))
                } else {
                    ClassifierKind::DataType(DataTypeLog::default().generate(rng))
                }
            }
            ClassifierKindContainer::Value(child) => match child.as_ref() {
                ClassifierKindChild::Class(log) => ClassifierKind::Class(log.generate(rng)),
                ClassifierKindChild::DataType(log) => ClassifierKind::DataType(log.generate(rng)),
            },
            ClassifierKindContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    ClassifierKindChild::Class(log) => ClassifierKind::Class(log.generate(rng)),
                    ClassifierKindChild::DataType(log) => {
                        ClassifierKind::DataType(log.generate(rng))
                    }
                })
                .unwrap(),
        }
    }
}

impl OpGeneratorNested for ClassifierLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        Classifier::ModelElementSuper(self.model_element_super().generate(rng))
    }
}

impl OpGeneratorNested for StructuralFeatureKindLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        match &self.child {
            StructuralFeatureKindContainer::Unset => {
                if rng.random_bool(0.65) {
                    StructuralFeatureKind::Attribute(AttributeLog::default().generate(rng))
                } else {
                    StructuralFeatureKind::Reference(ReferenceLog::default().generate(rng))
                }
            }
            StructuralFeatureKindContainer::Value(child) => match child.as_ref() {
                StructuralFeatureKindChild::Attribute(log) => {
                    StructuralFeatureKind::Attribute(log.generate(rng))
                }
                StructuralFeatureKindChild::Reference(log) => {
                    StructuralFeatureKind::Reference(log.generate(rng))
                }
            },
            StructuralFeatureKindContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    StructuralFeatureKindChild::Attribute(log) => {
                        StructuralFeatureKind::Attribute(log.generate(rng))
                    }
                    StructuralFeatureKindChild::Reference(log) => {
                        StructuralFeatureKind::Reference(log.generate(rng))
                    }
                })
                .unwrap(),
        }
    }
}

impl OpGeneratorNested for StructuralFeatureLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
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
            Choice::ModelElement => {
                StructuralFeature::ModelElementSuper(self.model_element_super().generate(rng))
            }
            Choice::Lower => StructuralFeature::Lower(self.lower().generate(rng)),
            Choice::Upper => StructuralFeature::Upper(self.upper().generate(rng)),
            Choice::IsOrdered => StructuralFeature::IsOrdered(self.is_ordered().generate(rng)),
            Choice::IsUnique => StructuralFeature::IsUnique(self.is_unique().generate(rng)),
        }
    }
}

impl OpGeneratorNested for AttributeLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        Attribute::StructuralFeatureSuper(self.structural_feature_super().generate(rng))
    }
}

impl OpGeneratorNested for ReferenceLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
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
            Choice::StructuralFeature => {
                Reference::StructuralFeatureSuper(self.structural_feature_super().generate(rng))
            }
            Choice::IsContainer => Reference::IsContainer(self.is_container().generate(rng)),
        }
    }
}

impl OpGeneratorNested for ClassLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            ClassifierKind,
            IsAbstract,
            Features,
        }

        match [Choice::ClassifierKind, Choice::IsAbstract, Choice::Features]
            [WeightedIndex::new([3, 3, 1]).unwrap().sample(rng)]
        {
            Choice::ClassifierKind => Class::ClassifierSuper(self.classifier_super().generate(rng)),
            Choice::IsAbstract => Class::IsAbstract(self.is_abstract().generate(rng)),
            Choice::Features => {
                Class::Features(generate_structural_feature_list(self.features(), rng))
            }
        }
    }
}

impl OpGeneratorNested for DataTypeLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        DataType::ClassifierSuper(self.classifier_super().generate(rng))
    }
}

impl FuzzMetrics for ClassHierarchyLog {
    fn structure_metrics(&self) -> StructureMetrics {
        StructureMetrics::object([
            self.package_log().structure_metrics(),
            self.reference_manager_log().structure_metrics(),
        ])
    }
}
