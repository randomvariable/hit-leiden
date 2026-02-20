use crate::core::config::RunConfig;
use crate::core::error::HitLeidenError;
use crate::core::partition::state::PartitionState;
use crate::core::runtime::orchestrator;
use crate::core::types::{
    BackendType, GraphInput, PartitionResult, RunExecution, RunOutcome, RunStatus,
};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn run(graph: &GraphInput, config: &RunConfig) -> Result<RunOutcome, HitLeidenError> {
    config
        .validate()
        .map_err(|e| HitLeidenError::InvalidInput(e.to_string()))?;

    if graph
        .edges
        .iter()
        .any(|(s, d, _)| *s >= graph.node_count || *d >= graph.node_count)
    {
        return Err(HitLeidenError::InvalidInput(
            "edge endpoint exceeds node_count".to_string(),
        ));
    }

    let started_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut partition_state = PartitionState::identity(graph.node_count);
    let resolution = orchestrator::resolve_with_fallback(config, true);

    hit_leiden(&mut partition_state, graph, 1.0, config.mode);

    let execution = RunExecution {
        run_id: format!("run:{}", graph.dataset_id),
        dataset_id: graph.dataset_id.clone(),
        config_id: "default".to_string(),
        started_at,
        completed_at: Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        ),
        status: RunStatus::Succeeded,
        backend: BackendType::PureRust,
        graph_backend_resolved: match resolution.backend_resolved {
            crate::core::backend::GraphBackend::InMemory => {
                crate::core::types::GraphBackend::InMemory
            }
            crate::core::backend::GraphBackend::Mmap => crate::core::types::GraphBackend::Mmap,
        },
        graph_source_resolved: match resolution.source_resolved {
            crate::core::backend::GraphSource::File => crate::core::types::GraphSourceType::File,
            crate::core::backend::GraphSource::Neo4jSnapshot => {
                crate::core::types::GraphSourceType::Neo4jSnapshot
            }
            crate::core::backend::GraphSource::LiveNeo4j => {
                crate::core::types::GraphSourceType::Neo4jSnapshot
            } // Fallback
        },
        fallback_reason: resolution.fallback_reason,
    };

    let partition = PartitionResult {
        run_id: execution.run_id.clone(),
        node_to_community: partition_state.node_to_comm,
        community_count: graph.node_count,
        quality_score: 1.0,
        iteration_count: 1,
    };

    Ok(RunOutcome {
        execution,
        partition: Some(partition),
        validation: None,
    })
}

// Algorithm 6: HIT-Leiden
pub fn hit_leiden(
    state: &mut PartitionState,
    delta_g: &GraphInput,
    gamma: f64,
    mode: crate::core::config::RunMode,
) {
    use crate::core::graph::in_memory::InMemoryGraph;

    if state.supergraphs.is_empty() {
        state.supergraphs.push(InMemoryGraph::from(delta_g));
    }

    let p_max = state.levels;
    let mut current_delta = delta_g.clone();

    let mut changed_nodes_per_level = vec![std::collections::BTreeSet::new(); p_max];
    let mut refined_nodes_per_level = vec![std::collections::BTreeSet::new(); p_max];

    for p in 0..p_max {
        // G_p <- G_p \oplus \Delta G_p
        // f_p, \Psi, B_p, K <- inc_movement(G_p, \Delta G_p, f_p, s_cur_p, \Psi, \gamma)
        let (b_p, k) = inc_movement(
            &state.supergraphs[p],
            &current_delta,
            &mut state.community_mapping_per_level[p],
            &state.current_subcommunity_mapping_per_level[p],
            &mut state.cc_indices_per_level[p],
            gamma,
            mode,
        );
        changed_nodes_per_level[p] = b_p;

        // s_cur_p, \Psi, R_p <- inc_refinement(G_p, f_p, s_cur_p, \Psi, K, \gamma)
        let r_p = inc_refinement(
            &state.supergraphs[p],
            &state.community_mapping_per_level[p],
            &mut state.current_subcommunity_mapping_per_level[p],
            &mut state.cc_indices_per_level[p],
            &k,
            gamma,
            mode,
        );
        refined_nodes_per_level[p] = r_p.clone();

        if p < p_max - 1 {
            // \Delta G_{p+1}, s_pre_p <- inc_aggregation(G_p, \Delta G_p, s_pre_p, s_cur_p, R_p)
            let (next_delta, next_s_pre) = inc_aggregation(
                &state.supergraphs[p],
                &current_delta,
                &state.previous_subcommunity_mapping_per_level[p],
                &state.current_subcommunity_mapping_per_level[p],
                &r_p,
            );
            current_delta = next_delta;
            state.previous_subcommunity_mapping_per_level[p] = next_s_pre;
        }
    }

    // {f_P} <- def_update({f_P}, {s_cur_P}, {B_P}, P)
    def_update(
        &mut state.community_mapping_per_level,
        &state.current_subcommunity_mapping_per_level,
        &mut changed_nodes_per_level,
        p_max,
    );
    // {g_P} <- def_update({g_P}, {s_cur_P}, {R_P}, P)
    def_update(
        &mut state.refined_community_mapping_per_level,
        &state.current_subcommunity_mapping_per_level,
        &mut refined_nodes_per_level,
        p_max,
    );
    // f <- g_1
    state.node_to_comm = state.community_mapping_per_level[0].clone();
}

fn inc_movement(
    graph: &crate::core::graph::in_memory::InMemoryGraph,
    delta_graph: &GraphInput,
    node_to_community: &mut Vec<usize>,
    node_to_subcommunity: &[usize],
    _edge_updates: &mut Vec<usize>,
    resolution_parameter: f64,
    mode: crate::core::config::RunMode,
) -> (
    std::collections::BTreeSet<usize>,
    std::collections::BTreeSet<usize>,
) {
    use std::collections::BTreeSet;

    let mut active_nodes = BTreeSet::new();
    let mut changed_nodes = BTreeSet::new();
    let mut affected_nodes_for_refinement = BTreeSet::new();

    // 2 for (v_i, v_j, \alpha) \in \Delta G do
    for &(u, v, w) in &delta_graph.edges {
        let alpha = w.unwrap_or(1.0);
        if alpha > 0.0 && node_to_community[u] != node_to_community[v] {
            active_nodes.insert(u);
            active_nodes.insert(v);
        }
        if alpha < 0.0 && node_to_community[u] == node_to_community[v] {
            active_nodes.insert(u);
            active_nodes.insert(v);
        }
        if node_to_subcommunity[u] == node_to_subcommunity[v] {
            // update_edge(G_\Psi, (v_i, v_j, \alpha))
            // K.add(v_i); K.add(v_j);
            affected_nodes_for_refinement.insert(u);
            affected_nodes_for_refinement.insert(v);
        }
    }

    // If delta_graph is empty (initial run), we need to add all nodes to active_nodes
    if delta_graph.edges.is_empty() {
        for i in 0..graph.node_count {
            active_nodes.insert(i);
        }
    }

    let twice_total_weight = graph.total_weight() * 2.0;
    let mut community_degrees = vec![0.0; graph.node_count];
    let mut node_degrees = vec![0.0; graph.node_count];
    for i in 0..graph.node_count {
        let d_i: f64 = graph.neighbors(i).map(|(_, w)| w).sum();
        node_degrees[i] = d_i;
        community_degrees[node_to_community[i]] += d_i;
    }

    if mode == crate::core::config::RunMode::Throughput {
        let mut current_active_nodes = active_nodes;
        while !current_active_nodes.is_empty() {
            let (new_changed, new_affected, next_active) =
                crate::core::algorithm::throughput::inc_movement_parallel(
                    graph,
                    &current_active_nodes,
                    node_to_community,
                    node_to_subcommunity,
                    &mut community_degrees,
                    &node_degrees,
                    twice_total_weight,
                    resolution_parameter,
                );
            changed_nodes.extend(new_changed);
            affected_nodes_for_refinement.extend(new_affected);
            current_active_nodes = next_active;
        }
        return (changed_nodes, affected_nodes_for_refinement);
    }

    // 9 for A \neq \emptyset do
    while !active_nodes.is_empty() {
        let current_node = *active_nodes.iter().next().unwrap();
        active_nodes.remove(&current_node);

        let mut best_community = node_to_community[current_node];
        let mut best_modularity_gain = 0.0;

        // Calculate \Delta Q for moving current_node to C
        // \Delta Q(v \to C', \gamma) = \frac{w(v, C') - w(v, C)}{2m} + \frac{\gamma \cdot d(v) \cdot (d(C) - d(v) - d(C'))}{(2m)^2}

        // First, find neighboring communities
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
            node_to_community[current_node] = best_community;
            changed_nodes.insert(current_node);
            community_degrees[old_community] -= current_node_degree;
            community_degrees[best_community] += current_node_degree;

            for (neighbor_node, _w) in graph.neighbors(current_node) {
                if node_to_community[neighbor_node] != best_community {
                    active_nodes.insert(neighbor_node);
                }
                if node_to_subcommunity[current_node] == node_to_subcommunity[neighbor_node] {
                    // update_edge(G_\Psi, (v_i, v_j, -w(v_i, v_j)))
                    affected_nodes_for_refinement.insert(current_node);
                    affected_nodes_for_refinement.insert(neighbor_node);
                }
            }
        }
    }

    (changed_nodes, affected_nodes_for_refinement)
}

fn inc_refinement(
    graph: &crate::core::graph::in_memory::InMemoryGraph,
    node_to_community: &[usize],
    node_to_subcommunity: &mut Vec<usize>,
    _edge_updates: &mut Vec<usize>,
    affected_nodes: &std::collections::BTreeSet<usize>,
    resolution_parameter: f64,
    mode: crate::core::config::RunMode,
) -> std::collections::BTreeSet<usize> {
    use std::collections::{BTreeMap, BTreeSet, VecDeque};
    let mut refined_nodes = BTreeSet::new();

    // Find affected sub-communities
    let mut affected_subcommunities = BTreeSet::new();
    for &v in affected_nodes {
        affected_subcommunities.insert(node_to_subcommunity[v]);
    }

    let mut next_subcommunity_id = node_to_subcommunity.iter().max().copied().unwrap_or(0) + 1;

    // 2 for v_i \in K do
    for &subcommunity in &affected_subcommunities {
        let mut vertices = Vec::new();
        for i in 0..graph.node_count {
            if node_to_subcommunity[i] == subcommunity {
                vertices.push(i);
            }
        }

        if vertices.is_empty() {
            continue;
        }

        let mut visited = vec![false; vertices.len()];
        let mut components = Vec::new();

        for i in 0..vertices.len() {
            if !visited[i] {
                let mut comp = Vec::new();
                let mut queue = VecDeque::new();
                queue.push_back(i);
                visited[i] = true;

                while let Some(curr_idx) = queue.pop_front() {
                    let current_node = vertices[curr_idx];
                    comp.push(current_node);

                    for (neighbor_node, _w) in graph.neighbors(current_node) {
                        if node_to_subcommunity[neighbor_node] == subcommunity {
                            if let Some(neighbor_index) =
                                vertices.iter().position(|&x| x == neighbor_node)
                            {
                                if !visited[neighbor_index] {
                                    visited[neighbor_index] = true;
                                    queue.push_back(neighbor_index);
                                }
                            }
                        }
                    }
                }
                components.push(comp);
            }
        }

        if components.len() > 1 {
            let mut largest_idx = 0;
            let mut largest_size = components[0].len();
            for (i, comp) in components.iter().enumerate().skip(1) {
                if comp.len() > largest_size {
                    largest_size = comp.len();
                    largest_idx = i;
                }
            }

            for (i, comp) in components.iter().enumerate() {
                if i != largest_idx {
                    let new_subcommunity = next_subcommunity_id;
                    next_subcommunity_id += 1;
                    for &v in comp {
                        node_to_subcommunity[v] = new_subcommunity;
                        refined_nodes.insert(v);
                    }
                }
            }
        }
    }

    let is_initial = node_to_subcommunity
        .iter()
        .enumerate()
        .all(|(i, &c)| i == c);
    if is_initial {
        for i in 0..graph.node_count {
            refined_nodes.insert(i);
        }
    }

    let twice_total_weight = graph.total_weight() * 2.0;
    let mut subcommunity_degrees = BTreeMap::new();
    let mut node_degrees = vec![0.0; graph.node_count];
    for i in 0..graph.node_count {
        let d_i: f64 = graph.neighbors(i).map(|(_, w)| w).sum();
        node_degrees[i] = d_i;
        *subcommunity_degrees
            .entry(node_to_subcommunity[i])
            .or_insert(0.0) += d_i;
    }

    let mut refined_nodes_sorted: Vec<usize> = refined_nodes.iter().copied().collect();
    refined_nodes_sorted.sort_by(|&a, &b| node_degrees[a].partial_cmp(&node_degrees[b]).unwrap());

    if mode == crate::core::config::RunMode::Throughput {
        crate::core::algorithm::throughput::inc_refinement_parallel(
            graph,
            &refined_nodes_sorted,
            node_to_community,
            node_to_subcommunity,
            &mut subcommunity_degrees,
            &node_degrees,
            twice_total_weight,
            resolution_parameter,
        );
        return refined_nodes;
    }

    // 5 for v_i \in R do
    for &current_node in &refined_nodes_sorted {
        let mut is_singleton = true;
        for i in 0..graph.node_count {
            if i != current_node && node_to_subcommunity[i] == node_to_subcommunity[current_node] {
                is_singleton = false;
                break;
            }
        }

        if is_singleton {
            let mut neighbor_subcommunities = BTreeMap::new();
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
                let old_subcommunity = node_to_subcommunity[current_node];
                node_to_subcommunity[current_node] = best_subcommunity;
                *subcommunity_degrees.entry(old_subcommunity).or_insert(0.0) -= current_node_degree;
                *subcommunity_degrees.entry(best_subcommunity).or_insert(0.0) +=
                    current_node_degree;
            }
        }
    }

    refined_nodes
}

fn inc_aggregation(
    graph: &crate::core::graph::in_memory::InMemoryGraph,
    delta_graph: &GraphInput,
    previous_node_to_subcommunity: &[usize],
    current_node_to_subcommunity: &[usize],
    refined_nodes: &std::collections::BTreeSet<usize>,
) -> (GraphInput, Vec<usize>) {
    let mut delta_supergraph = Vec::new();
    let mut next_previous_node_to_subcommunity = previous_node_to_subcommunity.to_vec();

    // 2 for (v_i, v_j, \alpha) \in \Delta G do
    for &(u, v, w) in &delta_graph.edges {
        let alpha = w.unwrap_or(1.0);
        let subcommunity_u = previous_node_to_subcommunity[u];
        let subcommunity_v = previous_node_to_subcommunity[v];
        delta_supergraph.push((subcommunity_u, subcommunity_v, Some(alpha)));
    }

    // 5 for v_i \in R do
    for &current_node in refined_nodes {
        for (neighbor_node, w) in graph.neighbors(current_node) {
            if current_node_to_subcommunity[neighbor_node]
                == previous_node_to_subcommunity[neighbor_node]
                || current_node < neighbor_node
            {
                delta_supergraph.push((
                    previous_node_to_subcommunity[current_node],
                    previous_node_to_subcommunity[neighbor_node],
                    Some(-w),
                ));
                delta_supergraph.push((
                    current_node_to_subcommunity[current_node],
                    current_node_to_subcommunity[neighbor_node],
                    Some(w),
                ));
            }
        }
        // Self loops
        // delta_supergraph.push((previous_node_to_subcommunity[current_node], previous_node_to_subcommunity[current_node], Some(-w(current_node, current_node))));
        // delta_supergraph.push((current_node_to_subcommunity[current_node], current_node_to_subcommunity[current_node], Some(w(current_node, current_node))));
    }

    // 12 for v_i \in R do
    for &current_node in refined_nodes {
        next_previous_node_to_subcommunity[current_node] =
            current_node_to_subcommunity[current_node];
    }

    // 14 Compress(\Delta H)
    let mut compressed_supergraph = std::collections::BTreeMap::new();
    for (u, v, w) in delta_supergraph {
        let weight = w.unwrap_or(1.0);
        let (min_u, max_v) = if u <= v { (u, v) } else { (v, u) };
        *compressed_supergraph.entry((min_u, max_v)).or_insert(0.0) += weight;
    }

    let mut final_delta_supergraph = Vec::new();
    for ((u, v), w) in compressed_supergraph {
        if w.abs() > 1e-9 {
            final_delta_supergraph.push((u, v, Some(w)));
        }
    }

    let max_subcommunity = current_node_to_subcommunity
        .iter()
        .chain(previous_node_to_subcommunity.iter())
        .copied()
        .max()
        .unwrap_or(0);
    let next_node_count = max_subcommunity + 1;

    let next_delta_graph = GraphInput {
        dataset_id: delta_graph.dataset_id.clone(),
        node_count: next_node_count,
        edges: final_delta_supergraph,
    };

    (next_delta_graph, next_previous_node_to_subcommunity)
}

fn def_update(
    node_to_community_per_level: &mut [Vec<usize>],
    node_to_subcommunity_per_level: &[Vec<usize>],
    changed_nodes_per_level: &mut [std::collections::BTreeSet<usize>],
    max_levels: usize,
) {
    // 1 for p from P to 1 do
    for p in (0..max_levels).rev() {
        // 2 if p \neq P then
        if p < max_levels - 1 {
            // 3 for v_i^p \in B_p do
            for &current_node in &changed_nodes_per_level[p] {
                // 4 f_p(v_i^p) = f_{p+1}(s_p(v_i^p))
                node_to_community_per_level[p][current_node] = node_to_community_per_level[p + 1]
                    [node_to_subcommunity_per_level[p][current_node]];
            }
        }

        // 5 if p \neq 1 then
        if p > 0 {
            // 6 for v_i^p \in B_p do
            let changed_nodes_clone = changed_nodes_per_level[p].clone();
            for &current_node in &changed_nodes_clone {
                // 7 B_{p-1}.add(s_p^{-1}(v_i^p))
                // We need to find all vertices in level p-1 that map to current_node in level p
                for (previous_level_node, &subcommunity_value) in
                    node_to_subcommunity_per_level[p - 1].iter().enumerate()
                {
                    if subcommunity_value == current_node {
                        changed_nodes_per_level[p - 1].insert(previous_level_node);
                    }
                }
            }
        }
    }
}
