use hit_leiden::benchmark::dynamic_graph::DynamicGraphBuilder;
use hit_leiden::benchmark::hit_leiden_incremental::run_incremental;
/// Profiling harness for incremental batch updates
/// Loads dataset, shuffles edges, processes as incremental batches
/// Compares HIT-Leiden against ST-Leiden baseline
use hit_leiden::GraphInput;
use lender::prelude::*;
use std::fs;
use std::path::Path;
use webgraph::prelude::*;

const OUTPUT_DIR: &str = "artifacts/incremental";
const OUTPUT_CSV: &str = "artifacts/incremental/uk_2007_05_100000_incremental.csv";
const OUTPUT_JSON: &str = "artifacts/incremental/uk_2007_05_100000_incremental.json";

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

fn write_exports(
    outcome: &hit_leiden::core::types::IncrementalOutcome,
    initial_edges: usize,
    batch_size: usize,
    rounds: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(OUTPUT_DIR)?;

    let mut csv = String::from(
        "batch_idx,edges_added,total_edges,nodes_in_graph,hit_leiden_time_ms,st_leiden_time_ms,speedup,hit_leiden_iterations,modularity\n",
    );
    for batch in &outcome.batches {
        csv.push_str(&format!(
            "{},{},{},{},{:.6},{:.6},{:.6},{},{}\n",
            batch.batch_idx,
            batch.edges_added,
            batch.total_edges,
            batch.nodes_in_graph,
            batch.hit_leiden_time_ms,
            batch.st_leiden_time_ms,
            batch.speedup,
            batch.hit_leiden_iterations,
            batch.modularity
        ));
    }
    fs::write(OUTPUT_CSV, csv)?;

    let mut batches_json = String::new();
    for (i, batch) in outcome.batches.iter().enumerate() {
        if i > 0 {
            batches_json.push(',');
        }
        batches_json.push_str(&format!(
            "{{\"batch_idx\":{},\"edges_added\":{},\"total_edges\":{},\"nodes_in_graph\":{},\"hit_leiden_time_ms\":{:.6},\"st_leiden_time_ms\":{:.6},\"speedup\":{:.6},\"hit_leiden_iterations\":{},\"modularity\":{}}}",
            batch.batch_idx,
            batch.edges_added,
            batch.total_edges,
            batch.nodes_in_graph,
            batch.hit_leiden_time_ms,
            batch.st_leiden_time_ms,
            batch.speedup,
            batch.hit_leiden_iterations,
            batch.modularity
        ));
    }

    let json = format!(
        "{{\"dataset_id\":\"uk-2007-05@100000\",\"paper_setup\":{{\"initial_ratio\":0.8,\"initial_edges\":{},\"batch_size\":{},\"rounds\":{}}},\"summary\":{{\"total_batches\":{},\"total_time_seconds\":{:.6},\"avg_speedup\":{:.6},\"cumulative_speedup\":{:.6}}},\"batches\":[{}]}}",
        initial_edges,
        batch_size,
        rounds,
        outcome.batches.len(),
        outcome.total_time_seconds,
        outcome.avg_speedup,
        outcome.cumulative_speedup,
        batches_json
    );
    fs::write(OUTPUT_JSON, json)?;

    Ok(())
}

fn main() {
    let graph = load_graph();

    // Paper-style setup:
    // - initial static graph from first 80% of shuffled edges
    // - then r=9 update batches with b=1000
    let builder = DynamicGraphBuilder::new(&graph);
    let split = builder.paper_split(0.8, 1000, 9, 42);

    eprintln!(
        "\nInitial graph: {} edges | Processing {} update batches of {} edges each…\n",
        split.initial_graph.edges.len(),
        split.update_batches.len(),
        split.batch_size
    );

    let initial_edges = split.initial_graph.edges.len();
    let batch_size = split.batch_size;
    let rounds = split.rounds;

    match run_incremental(split.update_batches, batch_size, initial_edges) {
        Ok(outcome) => {
            println!("\n{}", "=".repeat(50));
            println!("INCREMENTAL BATCH RESULTS");
            println!("{}", "=".repeat(50));
            println!("Total batches: {}", outcome.batches.len());
            println!("Total time: {:.2}s", outcome.total_time_seconds);
            println!("Avg speedup (per-batch): {:.2}x", outcome.avg_speedup);
            println!(
                "Cumulative speedup (total time): {:.2}x",
                outcome.cumulative_speedup
            );

            println!("\n{:-^50}", "Per-batch breakdown");
            println!(
                "{:<6} {:<12} {:<12} {:<12} {:<10}",
                "Batch", "Edges", "HIT (ms)", "ST (ms)", "Speedup"
            );
            println!("{}", "-".repeat(50));

            for batch in &outcome.batches {
                println!(
                    "{:<6} {:<12.0} {:<12.2} {:<12.2} {:<10.2}x",
                    batch.batch_idx,
                    batch.total_edges,
                    batch.hit_leiden_time_ms,
                    batch.st_leiden_time_ms,
                    batch.speedup
                );
            }

            if let Err(e) = write_exports(&outcome, initial_edges, batch_size, rounds) {
                eprintln!("Failed to write exports: {}", e);
            } else {
                println!("\nExported CSV: {}", OUTPUT_CSV);
                println!("Exported JSON: {}", OUTPUT_JSON);
            }

            println!("{}", "=".repeat(50));
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
