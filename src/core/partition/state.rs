use crate::core::graph::in_memory::InMemoryGraph;

#[derive(Clone, Debug, PartialEq)]
pub struct PartitionState {
    pub node_to_comm: Vec<usize>,
    pub comm_weights: Vec<f64>,
    pub node_weights: Vec<f64>,

    // Hierarchical state for HIT-Leiden
    pub levels: usize,
    pub supergraphs: Vec<InMemoryGraph>,
    pub community_mapping_per_level: Vec<Vec<usize>>,
    pub refined_community_mapping_per_level: Vec<Vec<usize>>,
    pub previous_subcommunity_mapping_per_level: Vec<Vec<usize>>,
    pub current_subcommunity_mapping_per_level: Vec<Vec<usize>>,
}

impl PartitionState {
    pub fn identity(node_count: usize) -> Self {
        // Share one identity allocation, clone for those that will be mutated independently
        let identity: Vec<usize> = (0..node_count).collect();
        Self {
            node_to_comm: identity.clone(),
            comm_weights: vec![0.0; node_count],
            node_weights: vec![0.0; node_count],
            levels: 1,
            supergraphs: Vec::new(),
            community_mapping_per_level: vec![identity.clone()],
            refined_community_mapping_per_level: vec![identity.clone()],
            previous_subcommunity_mapping_per_level: vec![identity.clone()],
            current_subcommunity_mapping_per_level: vec![identity],
        }
    }

    pub fn with_weights(node_count: usize, node_weights: Vec<f64>) -> Self {
        let identity: Vec<usize> = (0..node_count).collect();
        Self {
            node_to_comm: identity.clone(),
            comm_weights: node_weights.clone(),
            node_weights,
            levels: 1,
            supergraphs: Vec::new(),
            community_mapping_per_level: vec![identity.clone()],
            refined_community_mapping_per_level: vec![identity.clone()],
            previous_subcommunity_mapping_per_level: vec![identity.clone()],
            current_subcommunity_mapping_per_level: vec![identity],
        }
    }
}
