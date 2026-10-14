#[cfg(feature = "fuzz")]
#[test]
#[ignore]
fn fuzz() {
    use std::num::NonZeroUsize;

    use class_hierarchy::package::{ClassHierarchyLog, ClassHierarchyValue};
    use moirai_fuzz::{
        config::{ChurnConfig, FuzzerConfig, MetricsSampling, Predicate, RunConfig, SaveMode},
        fuzzer::fuzzer,
    };
    use moirai_protocol::crdt::query::Read;

    let predicate = Predicate::new(Read::<ClassHierarchyValue>::new(), |a, b| {
        let package = a.package == b.package;
        if !package {
            return false;
        }

        if a.refs.node_count() == 0 && b.refs.node_count() == 0 {
            // If both graphs are empty, skip the isomorphism
            return true;
        } else {
            use class_hierarchy::utils::graph_view::Vf2GraphView;

            vf2::isomorphisms(&Vf2GraphView(&a.refs), &Vf2GraphView(&b.refs))
                .default_eq()
                .first()
                .is_some()
        }
    });

    let run = RunConfig::new(ChurnConfig::new(0.4, 0.6), 8, 1_000, None, None).with_response_time(
        MetricsSampling::every_replica(NonZeroUsize::new(1).unwrap()),
    );

    let config = FuzzerConfig::<ClassHierarchyLog, Read<ClassHierarchyValue>>::new(
        "json-model",
        vec![run.clone(); 100],
        predicate,
    )
    .save(SaveMode::Summary);

    fuzzer::<ClassHierarchyLog, Read<ClassHierarchyValue>>(config);
}
