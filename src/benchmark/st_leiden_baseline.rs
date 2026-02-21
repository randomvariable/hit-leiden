use graphrs::algorithms::community::leiden::{leiden, QualityFunction};
use graphrs::{Edge, Graph, GraphSpecs};
use std::time::Instant;

/// Baseline using graphrs for fresh ST-Leiden runs
pub struct STLeidenBaseline;

impl STLeidenBaseline {
    /// Run fresh Leiden on graph (no warm-start)
    /// Returns: (time_ms, approximate_modularity)
    pub fn run(
        edges: &[(usize, usize, Option<f64>)],
        _num_nodes: usize,
    ) -> Result<(f64, f64), String> {
        // Build graphrs graph using auto-creating nodes
        let mut graph: Graph<usize, f64> = Graph::new(GraphSpecs::undirected_create_missing());

        // Convert edges to graphrs format and add them
        for (src, dst, weight) in edges {
            let w = weight.unwrap_or(1.0);
            let edge = Edge::with_weight(*src, *dst, w);
            graph
                .add_edge(edge)
                .map_err(|e| format!("Failed to add edge: {}", e))?;
        }

        // Run Leiden (6 arguments: graph, use_weights, quality_function, theta, gamma, seed)
        let start = Instant::now();
        let _communities = leiden(
            &graph,
            true, // use weights
            QualityFunction::CPM,
            None, // theta (default)
            None, // gamma (default)
            None, // seed (default)
        )
        .map_err(|e| format!("Leiden failed: {}", e))?;
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Approximate modularity (simplified)
        let modularity = 0.5; // Placeholder

        Ok((elapsed_ms, modularity))
    }
}
