use class_hierarchy::{package::ClassHierarchyLog, utils::graph_view::Vf2GraphView};

#[test]
fn fuzz() {
    use moirai_fuzz::{
        config::{FuzzerConfig, RunConfig},
        fuzzer::fuzzer,
    };

    let run = RunConfig::new(0.4, 4, 4, None, None, false, false);
    let runs = vec![run.clone(); 10_000];

    let config = FuzzerConfig::<ClassHierarchyLog>::new(
        "class_hierarchy",
        runs,
        true,
        |a, b| {
            let package = a.package == b.package;
            if !package {
                println!("Package mismatch");
                println!("----- Package A -----");
                println!("{:#?}", a.package);
                println!("----- Package B -----");
                println!("{:#?}", b.package);
                return false;
            }

            if a.refs.node_count() == 0 && b.refs.node_count() == 0 {
                // If both graphs are empty, skip the isomorphism
                return true;
            } else {
                let is_isomorph = vf2::isomorphisms(&Vf2GraphView(&a.refs), &Vf2GraphView(&b.refs))
                    .default_eq()
                    .first()
                    .is_some();
                if !is_isomorph {
                    println!(
                        "Graph isomorphism mismatch: nodes {} vs {}, edges {} vs {}",
                        a.refs.node_count(),
                        b.refs.node_count(),
                        a.refs.edge_count(),
                        b.refs.edge_count()
                    );
                    println!("----- Graph A -----");
                    println!("{:#?}", a.refs);
                    println!("----- Graph B -----");
                    println!("{:#?}", b.refs);
                }
                is_isomorph
            }
        },
        false,
    );

    fuzzer::<ClassHierarchyLog>(config);
}
