use crate::core::types::GraphInput;
use rand::seq::SliceRandom;
use rand::SeedableRng;

#[derive(Clone, Debug)]
pub struct IncrementalSplit {
    pub initial_graph: GraphInput,
    pub update_batches: Vec<GraphInput>,
    pub batch_size: usize,
    pub rounds: usize,
}

/// Builder for creating dynamically updated graphs from batches
pub struct DynamicGraphBuilder {
    all_edges: Vec<(usize, usize)>,
    node_count: usize,
}

impl DynamicGraphBuilder {
    /// Create from full graph
    pub fn new(graph: &GraphInput) -> Self {
        let edges: Vec<(usize, usize)> = graph.edges.iter().map(|(s, d, _)| (*s, *d)).collect();

        Self {
            all_edges: edges,
            node_count: graph.node_count,
        }
    }

    /// Shuffle edges deterministically (like it-2004 in paper)
    pub fn shuffle(&mut self, seed: u64) {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        self.all_edges.shuffle(&mut rng);
    }

    /// Split into cumulative batches
    /// Each batch includes all edges from batch 0..i
    pub fn batches(&self, batch_size: usize) -> Vec<GraphInput> {
        let mut batches = Vec::new();
        let mut cumulative_edges = Vec::new();

        for (idx, chunk) in self.all_edges.chunks(batch_size).enumerate() {
            cumulative_edges.extend_from_slice(chunk);

            batches.push(GraphInput {
                dataset_id: format!("batch_{}", idx),
                node_count: self.node_count,
                edges: cumulative_edges
                    .iter()
                    .map(|(s, d)| (*s, *d, None::<f64>))
                    .collect(),
            });
        }

        batches
    }

    /// Build batches that follow the paper setup:
    /// - Initial static graph built from first `initial_ratio` of shuffled edges
    /// - Then `rounds` update batches of size `batch_size`
    /// - Returns cumulative graph states after each update batch
    pub fn paper_split(
        &self,
        initial_ratio: f64,
        batch_size: usize,
        rounds: usize,
        seed: u64,
    ) -> IncrementalSplit {
        let mut shuffled = self.all_edges.clone();
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        shuffled.shuffle(&mut rng);

        let clamped_ratio = initial_ratio.clamp(0.0, 1.0);
        let initial_count = ((shuffled.len() as f64) * clamped_ratio).floor() as usize;

        let initial_edges = shuffled[..initial_count].to_vec();
        let mut cumulative_edges = initial_edges.clone();
        let mut update_batches = Vec::new();

        let available_updates = shuffled.len().saturating_sub(initial_count);
        let effective_rounds = if batch_size == 0 {
            0
        } else {
            rounds.min(available_updates / batch_size)
        };

        for round in 0..effective_rounds {
            let start = initial_count + (round * batch_size);
            let end = start + batch_size;
            cumulative_edges.extend_from_slice(&shuffled[start..end]);

            update_batches.push(GraphInput {
                dataset_id: format!("paper_batch_{}", round),
                node_count: self.node_count,
                edges: cumulative_edges
                    .iter()
                    .map(|(s, d)| (*s, *d, None::<f64>))
                    .collect(),
            });
        }

        IncrementalSplit {
            initial_graph: GraphInput {
                dataset_id: "paper_initial".to_string(),
                node_count: self.node_count,
                edges: initial_edges
                    .iter()
                    .map(|(s, d)| (*s, *d, None::<f64>))
                    .collect(),
            },
            update_batches,
            batch_size,
            rounds: effective_rounds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paper_split_uses_initial_ratio_and_fixed_rounds() {
        let graph = GraphInput {
            dataset_id: "test".to_string(),
            node_count: 100,
            edges: (0..100)
                .map(|i| (i, (i + 1) % 100, None::<f64>))
                .collect(),
        };

        let builder = DynamicGraphBuilder::new(&graph);
        let split = builder.paper_split(0.8, 5, 4, 42);

        assert_eq!(split.initial_graph.edges.len(), 80);
        assert_eq!(split.update_batches.len(), 4);
        assert_eq!(split.update_batches[0].edges.len(), 85);
        assert_eq!(split.update_batches[3].edges.len(), 100);
        assert_eq!(split.batch_size, 5);
        assert_eq!(split.rounds, 4);
    }
}
