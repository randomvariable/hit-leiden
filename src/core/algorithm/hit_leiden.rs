use crate::core::config::RunConfig;
use crate::core::error::HitLeidenError;
use crate::core::partition::state::PartitionState;
use crate::core::runtime::orchestrator;
use crate::core::types::{
    BackendType, GraphInput, PartitionResult, RunExecution, RunOutcome, RunStatus,
};
use bitvec::prelude::*;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet, VecDeque};
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
    // Use Cow to avoid cloning delta_g at level 0; only own when aggregation produces a new delta
    let mut current_delta: Cow<GraphInput> = Cow::Borrowed(delta_g);

    let mut changed_nodes_per_level: Vec<BitVec> = vec![bitvec![0; delta_g.node_count]; p_max];
    let mut refined_nodes_per_level: Vec<BitVec> = vec![bitvec![0; delta_g.node_count]; p_max];

    for p in 0..p_max {
        let (b_p, k) = inc_movement(
            &state.supergraphs[p],
            &current_delta,
            &mut state.community_mapping_per_level[p],
            &state.current_subcommunity_mapping_per_level[p],
            gamma,
            mode,
        );
        changed_nodes_per_level[p] = b_p;

        let r_p = inc_refinement(
            &state.supergraphs[p],
            &state.community_mapping_per_level[p],
            &mut state.current_subcommunity_mapping_per_level[p],
            &k,
            gamma,
            mode,
        );
        refined_nodes_per_level[p] = r_p.clone();

        if p < p_max - 1 {
            let (next_delta, next_s_pre) = inc_aggregation(
                &state.supergraphs[p],
                &current_delta,
                &state.previous_subcommunity_mapping_per_level[p],
                &state.current_subcommunity_mapping_per_level[p],
                &r_p,
            );
            current_delta = Cow::Owned(next_delta);
            state.previous_subcommunity_mapping_per_level[p] = next_s_pre;
        }
    }

    def_update(
        &mut state.community_mapping_per_level,
        &state.current_subcommunity_mapping_per_level,
        &mut changed_nodes_per_level,
        p_max,
    );
    def_update(
        &mut state.refined_community_mapping_per_level,
        &state.current_subcommunity_mapping_per_level,
        &mut refined_nodes_per_level,
        p_max,
    );
    state.node_to_comm = state.community_mapping_per_level[0].clone();
}

fn inc_movement(
    graph: &crate::core::graph::in_memory::InMemoryGraph,
    delta_graph: &GraphInput,
    node_to_community: &mut Vec<usize>,
    node_to_subcommunity: &[usize],
    resolution_parameter: f64,
    mode: crate::core::config::RunMode,
) -> (BitVec, BitVec) {
    let n = graph.node_count;
    let mut active_nodes = bitvec![0; n];
    let mut changed_nodes = bitvec![0; n];
    let mut affected_nodes_for_refinement = bitvec![0; n];

    // 2 for (v_i, v_j, \alpha) \in \Delta G do
    for &(u, v, w) in &delta_graph.edges {
        let alpha = w.unwrap_or(1.0);
        if alpha > 0.0 && node_to_community[u] != node_to_community[v] {
            active_nodes.set(u, true);
            active_nodes.set(v, true);
        }
        if alpha < 0.0 && node_to_community[u] == node_to_community[v] {
            active_nodes.set(u, true);
            active_nodes.set(v, true);
        }
        if node_to_subcommunity[u] == node_to_subcommunity[v] {
            affected_nodes_for_refinement.set(u, true);
            affected_nodes_for_refinement.set(v, true);
        }
    }

    // If delta_graph is empty (initial run), activate all nodes
    if delta_graph.edges.is_empty() {
        active_nodes.fill(true);
    }

    let twice_total_weight = graph.total_weight() * 2.0;
    let mut community_degrees = vec![0.0; n];
    let mut node_degrees = vec![0.0; n];
    for i in 0..n {
        let d_i: f64 = graph.neighbors(i).map(|(_, w)| w).sum();
        node_degrees[i] = d_i;
        community_degrees[node_to_community[i]] += d_i;
    }

    if mode == crate::core::config::RunMode::Throughput {
        let mut current_active_nodes = active_nodes;
        // Create buffer pool once for reuse across multiple inc_movement_parallel calls
        let buffer_pool =
            crate::core::algorithm::throughput::BufferPool::new(n, rayon::current_num_threads());
        while current_active_nodes.any() {
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
                    &buffer_pool,
                );
            changed_nodes |= new_changed;
            affected_nodes_for_refinement |= new_affected;
            current_active_nodes = next_active;
        }
        return (changed_nodes, affected_nodes_for_refinement);
    }

    // 9 for A \neq \emptyset do (deterministic mode)
    while active_nodes.any() {
        let current_node = active_nodes.iter_ones().next().unwrap();
        active_nodes.set(current_node, false);

        let mut best_community = node_to_community[current_node];
        let mut best_modularity_gain = 0.0;

        let mut neighbor_communities: HashMap<usize, f64> = HashMap::new();
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
            changed_nodes.set(current_node, true);
            community_degrees[old_community] -= current_node_degree;
            community_degrees[best_community] += current_node_degree;

            for (neighbor_node, _w) in graph.neighbors(current_node) {
                if node_to_community[neighbor_node] != best_community {
                    active_nodes.set(neighbor_node, true);
                }
                if node_to_subcommunity[current_node] == node_to_subcommunity[neighbor_node] {
                    affected_nodes_for_refinement.set(current_node, true);
                    affected_nodes_for_refinement.set(neighbor_node, true);
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
    affected_nodes: &BitVec,
    resolution_parameter: f64,
    mode: crate::core::config::RunMode,
) -> BitVec {
    let n = graph.node_count;
    let mut refined_nodes = bitvec![0; n];

    // Build inverted index: subcommunity -> nodes (only for affected subcommunities)
    let mut affected_subcommunities: HashSet<usize> = HashSet::new();
    for v in affected_nodes.iter_ones() {
        affected_subcommunities.insert(node_to_subcommunity[v]);
    }

    // Build node lists per affected subcommunity in a single O(n) pass
    let mut subcomm_nodes: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n {
        let sc = node_to_subcommunity[i];
        if affected_subcommunities.contains(&sc) {
            subcomm_nodes.entry(sc).or_default().push(i);
        }
    }

    let mut next_subcommunity_id = node_to_subcommunity.iter().max().copied().unwrap_or(0) + 1;

    // Reusable visited bitvec across subcommunities
    let mut visited = bitvec![0; n];

    // 2 for v_i \in K do — connected component splitting
    for (_subcommunity, vertices) in &subcomm_nodes {
        if vertices.is_empty() {
            continue;
        }

        let mut components: Vec<Vec<usize>> = Vec::new();

        for &start_node in vertices {
            if visited[start_node] {
                continue;
            }
            let mut comp = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(start_node);
            visited.set(start_node, true);

            while let Some(current_node) = queue.pop_front() {
                comp.push(current_node);
                let current_sc = node_to_subcommunity[current_node];
                for (neighbor_node, _w) in graph.neighbors(current_node) {
                    if node_to_subcommunity[neighbor_node] == current_sc && !visited[neighbor_node]
                    {
                        visited.set(neighbor_node, true);
                        queue.push_back(neighbor_node);
                    }
                }
            }
            components.push(comp);
        }

        // Clear visited bits for nodes we touched
        for &v in vertices {
            visited.set(v, false);
        }

        if components.len() > 1 {
            let largest_idx = components
                .iter()
                .enumerate()
                .max_by_key(|(_, c)| c.len())
                .map(|(i, _)| i)
                .unwrap();

            for (i, comp) in components.iter().enumerate() {
                if i != largest_idx {
                    let new_subcommunity = next_subcommunity_id;
                    next_subcommunity_id += 1;
                    for &v in comp {
                        node_to_subcommunity[v] = new_subcommunity;
                        refined_nodes.set(v, true);
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
        refined_nodes.fill(true);
    }

    // Pre-compute subcommunity sizes for O(1) singleton check
    let max_subcomm = node_to_subcommunity.iter().max().copied().unwrap_or(0);
    let mut subcommunity_sizes = vec![0usize; max_subcomm + 1];
    for &sc in node_to_subcommunity.iter() {
        subcommunity_sizes[sc] += 1;
    }

    let twice_total_weight = graph.total_weight() * 2.0;
    let mut subcommunity_degrees: HashMap<usize, f64> = HashMap::new();
    let mut node_degrees = vec![0.0; n];
    for i in 0..n {
        let d_i: f64 = graph.neighbors(i).map(|(_, w)| w).sum();
        node_degrees[i] = d_i;
        *subcommunity_degrees
            .entry(node_to_subcommunity[i])
            .or_insert(0.0) += d_i;
    }

    let mut refined_nodes_sorted: Vec<usize> = refined_nodes.iter_ones().collect();
    refined_nodes_sorted.sort_by(|&a, &b| node_degrees[a].partial_cmp(&node_degrees[b]).unwrap());

    if mode == crate::core::config::RunMode::Throughput {
        crate::core::algorithm::throughput::inc_refinement_parallel(
            graph,
            &refined_nodes_sorted,
            node_to_community,
            node_to_subcommunity,
            &mut subcommunity_degrees,
            &subcommunity_sizes,
            &node_degrees,
            twice_total_weight,
            resolution_parameter,
        );
        return refined_nodes;
    }

    // 5 for v_i \in R do (deterministic refinement merging)
    for &current_node in &refined_nodes_sorted {
        // O(1) singleton check
        let is_singleton = subcommunity_sizes[node_to_subcommunity[current_node]] == 1;

        if is_singleton {
            let mut neighbor_subcommunities: HashMap<usize, f64> = HashMap::new();
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
                subcommunity_sizes[old_subcommunity] -= 1;
                subcommunity_sizes[best_subcommunity] += 1;
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
    refined_nodes: &BitVec,
) -> (GraphInput, Vec<usize>) {
    let mut delta_supergraph = Vec::new();
    // Mutate in-place instead of to_vec() — start from previous, update refined nodes
    let mut next_previous_node_to_subcommunity = previous_node_to_subcommunity.to_vec();

    // 2 for (v_i, v_j, \alpha) \in \Delta G do
    for &(u, v, w) in &delta_graph.edges {
        let alpha = w.unwrap_or(1.0);
        let subcommunity_u = previous_node_to_subcommunity[u];
        let subcommunity_v = previous_node_to_subcommunity[v];
        delta_supergraph.push((subcommunity_u, subcommunity_v, Some(alpha)));
    }

    // 5 for v_i \in R do
    for current_node in refined_nodes.iter_ones() {
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
    }

    // 12 for v_i \in R do
    for current_node in refined_nodes.iter_ones() {
        next_previous_node_to_subcommunity[current_node] =
            current_node_to_subcommunity[current_node];
    }

    // 14 Compress(\Delta H) — use HashMap instead of BTreeMap
    let mut compressed_supergraph: HashMap<(usize, usize), f64> = HashMap::new();
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
    changed_nodes_per_level: &mut [BitVec],
    max_levels: usize,
) {
    // 1 for p from P to 1 do
    for p in (0..max_levels).rev() {
        // 2 if p \neq P then
        if p < max_levels - 1 {
            // 3 for v_i^p \in B_p do
            for current_node in changed_nodes_per_level[p].iter_ones() {
                // 4 f_p(v_i^p) = f_{p+1}(s_p(v_i^p))
                node_to_community_per_level[p][current_node] = node_to_community_per_level[p + 1]
                    [node_to_subcommunity_per_level[p][current_node]];
            }
        }

        // 5 if p \neq 1 then
        if p > 0 {
            // 6 for v_i^p \in B_p do
            let changed_nodes_at_p: Vec<usize> = changed_nodes_per_level[p].iter_ones().collect();
            for current_node in changed_nodes_at_p {
                // 7 B_{p-1}.add(s_p^{-1}(v_i^p))
                for (previous_level_node, &subcommunity_value) in
                    node_to_subcommunity_per_level[p - 1].iter().enumerate()
                {
                    if subcommunity_value == current_node {
                        changed_nodes_per_level[p - 1].set(previous_level_node, true);
                    }
                }
            }
        }
    }
}
