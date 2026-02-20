use crate::core::types::GraphInput;

#[derive(Clone, Debug, PartialEq)]
pub struct InMemoryGraph {
    pub node_count: usize,
    pub edges: Vec<(usize, usize, Option<f64>)>,
    pub offsets: Vec<usize>,
    pub neighbors: Vec<usize>,
    pub weights: Vec<f64>,
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

        let mut neighbors = vec![0; offsets[value.node_count]];
        let mut weights = vec![0.0; offsets[value.node_count]];
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

        Self {
            node_count: value.node_count,
            edges: value.edges.clone(),
            offsets,
            neighbors,
            weights,
        }
    }
}

impl InMemoryGraph {
    pub fn neighbors(&self, node: usize) -> impl Iterator<Item = (usize, f64)> + '_ {
        let start = self.offsets[node];
        let end = self.offsets[node + 1];
        self.neighbors[start..end]
            .iter()
            .copied()
            .zip(self.weights[start..end].iter().copied())
    }

    pub fn degree(&self, node: usize) -> usize {
        self.offsets[node + 1] - self.offsets[node]
    }

    pub fn total_weight(&self) -> f64 {
        self.weights.iter().sum::<f64>() / 2.0
    }
}
