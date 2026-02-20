use crate::core::graph::in_memory::InMemoryGraph;
use std::collections::BTreeSet;

pub struct ThreadLocalState {
    pub node_to_community_updates: Vec<(usize, usize)>,
    pub community_degree_updates: Vec<(usize, f64)>,
    pub changed_nodes: BTreeSet<usize>,
    pub affected_nodes: BTreeSet<usize>,
    pub next_active_nodes: BTreeSet<usize>,
}

pub fn execute_shard(
    graph: &InMemoryGraph,
    shard: &[usize],
    node_to_community: &[usize],
    node_to_subcommunity: &[usize],
    community_degrees: &[f64],
    node_degrees: &[f64],
    twice_total_weight: f64,
    resolution_parameter: f64,
) -> ThreadLocalState {
    let mut state = ThreadLocalState {
        node_to_community_updates: Vec::new(),
        community_degree_updates: Vec::new(),
        changed_nodes: BTreeSet::new(),
        affected_nodes: BTreeSet::new(),
        next_active_nodes: BTreeSet::new(),
    };

    for &current_node in shard {
        let mut best_community = node_to_community[current_node];
        let mut best_modularity_gain = 0.0;

        let mut neighbor_communities = std::collections::BTreeMap::new();
        let mut weight_to_current_community = 0.0;
        let current_node_degree = node_degrees[current_node];

        for (neighbor_node, w) in graph.neighbors(current_node) {
            let c = node_to_community[neighbor_node];
            *neighbor_communities.entry(c).or_insert(0.0) += w;
            if c == node_to_community[current_node] {
                weight_to_current_community += w;
            }
        }

        for (&candidate_community, &weight_to_candidate_community) in &neighbor_communities {
            if candidate_community == node_to_community[current_node] {
                continue;
            }

            let current_community_degree = community_degrees[node_to_community[current_node]];
            let candidate_community_degree = community_degrees[candidate_community];

            let modularity_gain = (weight_to_candidate_community - weight_to_current_community)
                / twice_total_weight
                + resolution_parameter
                    * current_node_degree
                    * (current_community_degree - current_node_degree - candidate_community_degree)
                    / (twice_total_weight * twice_total_weight);

            if modularity_gain > best_modularity_gain {
                best_modularity_gain = modularity_gain;
                best_community = candidate_community;
            }
        }

        if best_modularity_gain > 0.0 {
            let old_community = node_to_community[current_node];
            state
                .node_to_community_updates
                .push((current_node, best_community));
            state.changed_nodes.insert(current_node);
            state
                .community_degree_updates
                .push((old_community, -current_node_degree));
            state
                .community_degree_updates
                .push((best_community, current_node_degree));

            for (neighbor_node, _w) in graph.neighbors(current_node) {
                if node_to_community[neighbor_node] != best_community {
                    state.next_active_nodes.insert(neighbor_node);
                }
                if node_to_subcommunity[current_node] == node_to_subcommunity[neighbor_node] {
                    state.affected_nodes.insert(current_node);
                    state.affected_nodes.insert(neighbor_node);
                }
            }
        }
    }

    state
}
