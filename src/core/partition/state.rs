use crate::core::graph::in_memory::InMemoryGraph;

#[derive(Clone, Debug, PartialEq)]
pub struct PartitionState {
    pub node_to_comm: Vec<usize>,
    pub comm_weights: Vec<f64>,
    pub node_weights: Vec<f64>,

    // Hierarchical state for HIT-Leiden
    pub levels: usize,
    pub supergraphs: Vec<InMemoryGraph>,
    pub community_mapping_per_level: Vec<Vec<usize>>, // Community mappings per level
    pub refined_community_mapping_per_level: Vec<Vec<usize>>, // Sub-community mappings per level
    pub previous_subcommunity_mapping_per_level: Vec<Vec<usize>>, // Previous sub-community mappings
    pub current_subcommunity_mapping_per_level: Vec<Vec<usize>>, // Current sub-community mappings
    pub cc_indices_per_level: Vec<Vec<usize>>,        // CC-indices
}

impl PartitionState {
    pub fn identity(node_count: usize) -> Self {
        Self {
            node_to_comm: (0..node_count).collect(),
            comm_weights: vec![0.0; node_count],
            node_weights: vec![0.0; node_count],
            levels: 1,
            supergraphs: Vec::new(),
            community_mapping_per_level: vec![(0..node_count).collect()],
            refined_community_mapping_per_level: vec![(0..node_count).collect()],
            previous_subcommunity_mapping_per_level: vec![(0..node_count).collect()],
            current_subcommunity_mapping_per_level: vec![(0..node_count).collect()],
            cc_indices_per_level: vec![(0..node_count).collect()],
        }
    }

    pub fn with_weights(node_count: usize, node_weights: Vec<f64>) -> Self {
        Self {
            node_to_comm: (0..node_count).collect(),
            comm_weights: node_weights.clone(),
            node_weights,
            levels: 1,
            supergraphs: Vec::new(),
            community_mapping_per_level: vec![(0..node_count).collect()],
            refined_community_mapping_per_level: vec![(0..node_count).collect()],
            previous_subcommunity_mapping_per_level: vec![(0..node_count).collect()],
            current_subcommunity_mapping_per_level: vec![(0..node_count).collect()],
            cc_indices_per_level: vec![(0..node_count).collect()],
        }
    }
}
