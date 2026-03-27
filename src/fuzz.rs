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

fn generate_boxed_model_element(
    log: &Box<ModelElementLog>,
    rng: &mut impl Rng,
) -> Box<ModelElement> {
    Box::new((**log).generate(rng))
}

fn generate_structural_feature(
    log: &StructuralFeatureLog,
    rng: &mut impl Rng,
) -> StructuralFeature {
    log.generate(rng)
}

fn generate_boxed_model_list(
    log: &NestedListLog<Box<ModelElementLog>>,
    rng: &mut impl Rng,
) -> NestedList<Box<ModelElement>> {
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
            let value = generate_boxed_model_element(&Box::<ModelElementLog>::default(), rng);
            NestedList::Insert { pos, value }
        }
        Choice::Update => {
            let pos = rng.random_range(0..positions.len());
            let target_id = &positions[pos];
            let value = log
                .children()
                .get(target_id)
                .map(|child| generate_boxed_model_element(child, rng))
                .unwrap_or_else(|| {
                    generate_boxed_model_element(&Box::<ModelElementLog>::default(), rng)
                });
            NestedList::Update { pos, value }
        }
        Choice::Delete => NestedList::Delete {
            pos: rng.random_range(0..positions.len()),
        },
    };

    assert!(log.is_enabled(&op));
    op
}

fn generate_structural_feature_list(
    log: &NestedListLog<StructuralFeatureLog>,
    rng: &mut impl Rng,
) -> NestedList<StructuralFeature> {
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
            let value = generate_structural_feature(&StructuralFeatureLog::default(), rng);
            NestedList::Insert { pos, value }
        }
        Choice::Update => {
            let pos = rng.random_range(0..positions.len());
            let target_id = &positions[pos];
            let value = log
                .children()
                .get(target_id)
                .map(|child| generate_structural_feature(child, rng))
                .unwrap_or_else(|| {
                    generate_structural_feature(&StructuralFeatureLog::default(), rng)
                });
            NestedList::Update { pos, value }
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
            ModelElementFeat,
            Content,
        }

        let choice = if self
            .content()
            .positions()
            .execute_query(Read::new())
            .is_empty()
        {
            [Choice::ModelElementFeat, Choice::Content]
                [WeightedIndex::new([1, 4]).unwrap().sample(rng)]
        } else {
            [Choice::ModelElementFeat, Choice::Content]
                [WeightedIndex::new([3, 2]).unwrap().sample(rng)]
        };

        match choice {
            Choice::ModelElementFeat => {
                Package::ModelElementFeat(self.model_element_feat().generate(rng))
            }
            Choice::Content => Package::Content(generate_boxed_model_list(self.content(), rng)),
        }
    }
}

impl OpGeneratorNested for ModelElementLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        match &self.child {
            ModelElementContainer::Unset => {
                match [0_u8, 1, 2][WeightedIndex::new([6, 1, 3]).unwrap().sample(rng)] {
                    0 => ModelElement::Classifier(ClassifierLog::default().generate(rng)),
                    1 => ModelElement::Package(PackageLog::default().generate(rng)),
                    _ => ModelElement::StructuralFeature(
                        StructuralFeatureLog::default().generate(rng),
                    ),
                }
            }
            ModelElementContainer::Value(child) => match child.as_ref() {
                ModelElementChild::Classifier(log) => ModelElement::Classifier(log.generate(rng)),
                ModelElementChild::Package(log) => ModelElement::Package(log.generate(rng)),
                ModelElementChild::StructuralFeature(log) => {
                    ModelElement::StructuralFeature(log.generate(rng))
                }
            },
            ModelElementContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    ModelElementChild::Classifier(log) => {
                        ModelElement::Classifier(log.generate(rng))
                    }
                    ModelElementChild::Package(log) => ModelElement::Package(log.generate(rng)),
                    ModelElementChild::StructuralFeature(log) => {
                        ModelElement::StructuralFeature(log.generate(rng))
                    }
                })
                .unwrap(),
        }
    }
}

impl OpGeneratorNested for ModelElementFeatLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        ModelElementFeat::Name(self.name().generate(rng))
    }
}

impl OpGeneratorNested for ClassifierLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        match &self.child {
            ClassifierContainer::Unset => {
                if rng.random_bool(0.65) {
                    Classifier::Class(ClassLog::default().generate(rng))
                } else {
                    Classifier::DataType(DataTypeLog::default().generate(rng))
                }
            }
            ClassifierContainer::Value(child) => match child.as_ref() {
                ClassifierChild::Class(log) => Classifier::Class(log.generate(rng)),
                ClassifierChild::DataType(log) => Classifier::DataType(log.generate(rng)),
            },
            ClassifierContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    ClassifierChild::Class(log) => Classifier::Class(log.generate(rng)),
                    ClassifierChild::DataType(log) => Classifier::DataType(log.generate(rng)),
                })
                .unwrap(),
        }
    }
}

impl OpGeneratorNested for ClassifierFeatLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        ClassifierFeat::ModelElementFeat(self.model_element_feat().generate(rng))
    }
}

impl OpGeneratorNested for StructuralFeatureLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        match &self.child {
            StructuralFeatureContainer::Unset => {
                if rng.random_bool(0.65) {
                    StructuralFeature::Attribute(AttributeLog::default().generate(rng))
                } else {
                    StructuralFeature::Reference(ReferenceLog::default().generate(rng))
                }
            }
            StructuralFeatureContainer::Value(child) => match child.as_ref() {
                StructuralFeatureChild::Attribute(log) => {
                    StructuralFeature::Attribute(log.generate(rng))
                }
                StructuralFeatureChild::Reference(log) => {
                    StructuralFeature::Reference(log.generate(rng))
                }
            },
            StructuralFeatureContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    StructuralFeatureChild::Attribute(log) => {
                        StructuralFeature::Attribute(log.generate(rng))
                    }
                    StructuralFeatureChild::Reference(log) => {
                        StructuralFeature::Reference(log.generate(rng))
                    }
                })
                .unwrap(),
        }
    }
}

impl OpGeneratorNested for StructuralFeatureFeatLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            ModelElementFeat,
            Lower,
            Upper,
            IsOrdered,
            IsUnique,
        }

        match [
            Choice::ModelElementFeat,
            Choice::Lower,
            Choice::Upper,
            Choice::IsOrdered,
            Choice::IsUnique,
        ]
        .into_iter()
        .choose(rng)
        .unwrap()
        {
            Choice::ModelElementFeat => {
                StructuralFeatureFeat::ModelElementFeat(self.model_element_feat().generate(rng))
            }
            Choice::Lower => StructuralFeatureFeat::Lower(self.lower().generate(rng)),
            Choice::Upper => StructuralFeatureFeat::Upper(self.upper().generate(rng)),
            Choice::IsOrdered => StructuralFeatureFeat::IsOrdered(self.is_ordered().generate(rng)),
            Choice::IsUnique => StructuralFeatureFeat::IsUnique(self.is_unique().generate(rng)),
        }
    }
}

impl OpGeneratorNested for AttributeLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        Attribute::StructuralFeatureFeat(self.structural_feature_feat().generate(rng))
    }
}

impl OpGeneratorNested for ReferenceLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            StructuralFeatureFeat,
            IsContainer,
        }

        match [Choice::StructuralFeatureFeat, Choice::IsContainer]
            .into_iter()
            .choose(rng)
            .unwrap()
        {
            Choice::StructuralFeatureFeat => {
                Reference::StructuralFeatureFeat(self.structural_feature_feat().generate(rng))
            }
            Choice::IsContainer => Reference::IsContainer(self.is_container().generate(rng)),
        }
    }
}

impl OpGeneratorNested for ClassLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            ClassifierFeat,
            IsAbstract,
            Features,
        }

        match [Choice::ClassifierFeat, Choice::IsAbstract, Choice::Features]
            [WeightedIndex::new([3, 3, 1]).unwrap().sample(rng)]
        {
            Choice::ClassifierFeat => Class::ClassifierFeat(self.classifier_feat().generate(rng)),
            Choice::IsAbstract => Class::IsAbstract(self.is_abstract().generate(rng)),
            Choice::Features => {
                Class::Features(generate_structural_feature_list(self.features(), rng))
            }
        }
    }
}

impl OpGeneratorNested for DataTypeLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        DataType::ClassifierFeat(self.classifier_feat().generate(rng))
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
