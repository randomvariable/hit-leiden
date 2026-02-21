use crate::benchmark::st_leiden_baseline::STLeidenBaseline;
use crate::core::config::{RunConfig, RunMode};
use crate::core::types::{BatchResult, GraphInput, IncrementalOutcome};
use std::time::Instant;

/// Run HIT-Leiden incrementally across batches and compare against ST-Leiden baseline
pub fn run_incremental(
    batches: Vec<GraphInput>,
    batch_size: usize,
    initial_edge_count: usize,
) -> Result<IncrementalOutcome, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    let overall_start = Instant::now();
    let mut prev_total_edges = initial_edge_count;

    for (idx, batch_graph) in batches.iter().enumerate() {
        // Run HIT-Leiden (incremental)
        let config = RunConfig {
            mode: RunMode::Throughput,
            ..Default::default()
        };

        let start = Instant::now();
        let outcome = crate::run(&batch_graph, &config)?;
        let hit_leiden_ms = start.elapsed().as_secs_f64() * 1000.0;

        let hit_leiden_iterations = outcome
            .partition
            .as_ref()
            .map(|p| p.iteration_count)
            .unwrap_or(0);

        let modularity = outcome
            .partition
            .as_ref()
            .map(|p| p.quality_score)
            .unwrap_or(0.0);

        // Run ST-Leiden baseline (fresh)
        let (st_leiden_ms, _st_modularity) =
            STLeidenBaseline::run(&batch_graph.edges, batch_graph.node_count)?;

        let speedup = if hit_leiden_ms > 0.0 {
            st_leiden_ms / hit_leiden_ms
        } else {
            0.0
        };

        let current_total = batch_graph.edges.len();
        let edges_added = current_total.saturating_sub(prev_total_edges);
        prev_total_edges = current_total;

        results.push(BatchResult {
            batch_idx: idx,
            edges_added: if edges_added == 0 {
                batch_size.min(current_total)
            } else {
                edges_added
            },
            total_edges: current_total,
            nodes_in_graph: batch_graph.node_count,
            hit_leiden_time_ms: hit_leiden_ms,
            st_leiden_time_ms: st_leiden_ms,
            speedup,
            hit_leiden_iterations,
            modularity,
        });

        eprintln!(
            "Batch {}: +{} edges | Total: {:.0} | HIT: {:.2}ms | ST: {:.2}ms | Speedup: {:.2}x",
            idx,
            edges_added,
            batch_graph.edges.len(),
            hit_leiden_ms,
            st_leiden_ms,
            speedup
        );
    }

    let total_seconds = overall_start.elapsed().as_secs_f64();
    let hit_total: f64 = results.iter().map(|r| r.hit_leiden_time_ms).sum();
    let st_total: f64 = results.iter().map(|r| r.st_leiden_time_ms).sum();

    let cumulative_speedup = if hit_total > 0.0 {
        st_total / hit_total
    } else {
        0.0
    };

    let avg_speedup = if !results.is_empty() {
        results.iter().map(|r| r.speedup).sum::<f64>() / results.len() as f64
    } else {
        0.0
    };

    Ok(IncrementalOutcome {
        batches: results,
        total_time_seconds: total_seconds,
        avg_speedup,
        cumulative_speedup,
    })
}
