use crate::core::algorithm::parallel_frontier::{execute_shard, ThreadLocalState};
use crate::core::graph::in_memory::InMemoryGraph;
use rayon::prelude::*;
use std::collections::BTreeSet;

pub fn inc_movement_parallel(
    graph: &InMemoryGraph,
    active_nodes: &BTreeSet<usize>,
    node_to_community: &mut Vec<usize>,
    node_to_subcommunity: &[usize],
    community_degrees: &mut Vec<f64>,
    node_degrees: &[f64],
    twice_total_weight: f64,
    resolution_parameter: f64,
) -> (BTreeSet<usize>, BTreeSet<usize>, BTreeSet<usize>) {
    let active_nodes_vec: Vec<usize> = active_nodes.iter().copied().collect();
    let chunk_size = (active_nodes_vec.len() / rayon::current_num_threads()).max(1);

    let states: Vec<ThreadLocalState> = active_nodes_vec
        .par_chunks(chunk_size)
        .map(|shard| {
            execute_shard(
                graph,
                shard,
                node_to_community,
                node_to_subcommunity,
                community_degrees,
                node_degrees,
                twice_total_weight,
                resolution_parameter,
            )
        })
        .collect();

    let mut changed_nodes = BTreeSet::new();
    let mut affected_nodes = BTreeSet::new();
    let mut next_active_nodes = BTreeSet::new();

    for state in states {
        for (node, new_comm) in state.node_to_community_updates {
            node_to_community[node] = new_comm;
        }
        for (comm, delta) in state.community_degree_updates {
            community_degrees[comm] += delta;
        }
        changed_nodes.extend(state.changed_nodes);
        affected_nodes.extend(state.affected_nodes);
        next_active_nodes.extend(state.next_active_nodes);
    }

    (changed_nodes, affected_nodes, next_active_nodes)
}

pub fn inc_refinement_parallel(
    graph: &InMemoryGraph,
    refined_nodes_sorted: &[usize],
    node_to_community: &[usize],
    node_to_subcommunity: &mut Vec<usize>,
    subcommunity_degrees: &mut std::collections::BTreeMap<usize, f64>,
    node_degrees: &[f64],
    twice_total_weight: f64,
    resolution_parameter: f64,
) {
    let chunk_size = (refined_nodes_sorted.len() / rayon::current_num_threads()).max(1);

    let states: Vec<Vec<(usize, usize, usize, f64)>> = refined_nodes_sorted
        .par_chunks(chunk_size)
        .map(|shard| {
            let mut local_updates = Vec::new();
            for &current_node in shard {
                let mut is_singleton = true;
                for i in 0..graph.node_count {
                    if i != current_node
                        && node_to_subcommunity[i] == node_to_subcommunity[current_node]
                    {
                        is_singleton = false;
                        break;
                    }
                }

                if is_singleton {
                    let mut neighbor_subcommunities = std::collections::BTreeMap::new();
                    let mut weight_to_current_subcommunity = 0.0;
                    let current_node_degree = node_degrees[current_node];

                    for (neighbor_node, w) in graph.neighbors(current_node) {
                        if node_to_community[neighbor_node] == node_to_community[current_node] {
                            let neighbor_subcommunity = node_to_subcommunity[neighbor_node];
                            *neighbor_subcommunities
                                .entry(neighbor_subcommunity)
                                .or_insert(0.0) += w;
                            if neighbor_subcommunity == node_to_subcommunity[current_node] {
                                weight_to_current_subcommunity += w;
                            }
                        }
                    }

                    let mut best_subcommunity = node_to_subcommunity[current_node];
                    let mut best_modularity_gain = 0.0;

                    for (&candidate_subcommunity, &weight_to_candidate_subcommunity) in
                        &neighbor_subcommunities
                    {
                        if candidate_subcommunity == node_to_subcommunity[current_node] {
                            continue;
                        }

                        let current_subcommunity_degree = *subcommunity_degrees
                            .get(&node_to_subcommunity[current_node])
                            .unwrap_or(&0.0);
                        let candidate_subcommunity_degree = *subcommunity_degrees
                            .get(&candidate_subcommunity)
                            .unwrap_or(&0.0);

                        let modularity_gain = (weight_to_candidate_subcommunity
                            - weight_to_current_subcommunity)
                            / twice_total_weight
                            + resolution_parameter
                                * current_node_degree
                                * (current_subcommunity_degree
                                    - current_node_degree
                                    - candidate_subcommunity_degree)
                                / (twice_total_weight * twice_total_weight);

                        if modularity_gain > best_modularity_gain {
                            best_modularity_gain = modularity_gain;
                            best_subcommunity = candidate_subcommunity;
                        }
                    }

                    if best_modularity_gain > 0.0 {
                        local_updates.push((
                            current_node,
                            node_to_subcommunity[current_node],
                            best_subcommunity,
                            current_node_degree,
                        ));
                    }
                }
            }
            local_updates
        })
        .collect();

    for state in states {
        for (node, old_subcomm, new_subcomm, degree) in state {
            node_to_subcommunity[node] = new_subcomm;
            *subcommunity_degrees.entry(old_subcomm).or_insert(0.0) -= degree;
            *subcommunity_degrees.entry(new_subcomm).or_insert(0.0) += degree;
        }
    }
}
