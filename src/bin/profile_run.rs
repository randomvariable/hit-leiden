/// Profiling harness: loads uk-2007-05@100000, runs the algorithm in a tight
/// loop for 30 seconds, then exits. Use with samply or perf instead of Criterion
/// to avoid gnuplot/reporting noise in the profile.
///
///   samply record ./target/release/profile_run
use hit_leiden::{run, GraphInput, RunConfig, RunMode};
use lender::prelude::*;
use std::path::Path;
use std::time::{Duration, Instant};
use webgraph::prelude::*;

fn load_graph() -> GraphInput {
    let path = Path::new("data/uk-2007-05@100000/uk-2007-05@100000");
    assert!(
        path.with_extension("graph").exists(),
        "Dataset not found at {:?}. Run `cargo make data` first.",
        path
    );

    eprintln!("Loading graph…");
    let graph = webgraph::graphs::bvgraph::sequential::BvGraphSeq::with_basename(path)
        .load()
        .expect("Failed to load webgraph");
    let num_nodes = graph.num_nodes();

    let mut edges = Vec::with_capacity(graph.num_arcs_hint().unwrap_or(0) as usize);
    for_![(src, succ) in graph {
        for dst in succ {
            if src <= dst {
                edges.push((src, dst, None::<f64>));
            }
        }
    }];

    eprintln!(
        "Loaded {} nodes, {} undirected edges",
        num_nodes,
        edges.len()
    );
    GraphInput {
        dataset_id: "uk-2007-05@100000".to_string(),
        node_count: num_nodes,
        edges,
    }
}

fn main() {
    let graph = load_graph();
    let config = RunConfig {
        mode: RunMode::Throughput,
        max_iterations: 1,
        quality_tolerance: 1.0,
        ..RunConfig::default()
    };

    let duration = Duration::from_secs(5);
    let deadline = Instant::now() + duration;
    let mut iters = 0u64;

    eprintln!("Running for {:?}… (Ctrl-C to stop early)", duration);
    while Instant::now() < deadline {
        run(&graph, &config).expect("run failed");
        iters += 1;
    }
    eprintln!("Completed {} iterations in {:?}", iters, duration);
}
