use crate::core::types::GraphInput;

#[derive(Clone, Debug, PartialEq)]
pub struct InMemoryGraph {
    pub node_count: usize,
    pub offsets: Vec<usize>,
    pub degrees: Vec<usize>, // Precomputed for O(1) lookup
    pub neighbors: Vec<usize>,
    pub weights: Vec<f64>,
    cached_total_weight: f64,
}

impl From<&GraphInput> for InMemoryGraph {
    fn from(value: &GraphInput) -> Self {
        let mut degrees = vec![0; value.node_count];
        for &(u, v, _) in &value.edges {
            degrees[u] += 1;
            degrees[v] += 1;
        }

        let mut offsets = vec![0; value.node_count + 1];
        for i in 0..value.node_count {
            offsets[i + 1] = offsets[i] + degrees[i];
        }

        let total_entries = offsets[value.node_count];
        let mut neighbors = vec![0; total_entries];
        let mut weights = vec![0.0; total_entries];
        let mut current_offsets = offsets.clone();

        for &(u, v, w) in &value.edges {
            let weight = w.unwrap_or(1.0);

            let u_offset = current_offsets[u];
            neighbors[u_offset] = v;
            weights[u_offset] = weight;
            current_offsets[u] += 1;

            let v_offset = current_offsets[v];
            neighbors[v_offset] = u;
            weights[v_offset] = weight;
            current_offsets[v] += 1;
        }

        let cached_total_weight = weights.iter().sum::<f64>() / 2.0;

        // Precompute degrees for O(1) neighbor iteration
        let degrees: Vec<usize> = (0..value.node_count)
            .map(|i| offsets[i + 1] - offsets[i])
            .collect();

        Self {
            node_count: value.node_count,
            offsets,
            degrees,
            neighbors,
            weights,
            cached_total_weight,
        }
    }
}

impl InMemoryGraph {
    /// Iterate over (neighbor, weight) pairs for a node.
    /// Uses precomputed degree for single-load bound calculation.
    #[inline]
    pub fn neighbors(&self, node: usize) -> impl Iterator<Item = (usize, f64)> + '_ {
        let start = self.offsets[node];
        let count = self.degrees[node]; // Single load instead of offsets[node+1]
        self.neighbors[start..start + count]
            .iter()
            .copied()
            .zip(self.weights[start..start + count].iter().copied())
    }

    /// Get node degree in O(1) time.
    #[inline]
    pub fn degree(&self, node: usize) -> usize {
        self.degrees[node] // Direct lookup, no subtraction
    }

    pub fn total_weight(&self) -> f64 {
        self.cached_total_weight
    }
}
