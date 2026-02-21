use crate::core::algorithm::parallel_frontier::{execute_shard, ShardResult, SharedBitVec};
use crate::core::graph::in_memory::InMemoryGraph;
use bitvec::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;

/// Thread-safe wrapper around UnsafeCell for per-thread buffer access.
/// SAFETY: Each rayon worker thread accesses a unique index via current_thread_index(),
/// ensuring no concurrent access to the same UnsafeCell. This is safe because:
/// 1. Rayon maintains a persistent thread pool (no thread ID reuse during parallel work)
/// 2. Each thread accesses only buffers[current_thread_index()]
/// 3. No two threads share the same thread index value
struct SyncUnsafeCell<T>(std::cell::UnsafeCell<T>);

unsafe impl<T> Sync for SyncUnsafeCell<T> {}

impl<T> SyncUnsafeCell<T> {
    fn new(value: T) -> Self {
        SyncUnsafeCell(std::cell::UnsafeCell::new(value))
    }

    fn get(&self) -> *mut T {
        self.0.get()
    }
}

/// Reusable per-thread buffer pool for inc_movement_parallel.
/// Allocates once and reuses across multiple calls to avoid repeated allocations.
/// Safe to reuse because each thread accesses its own slot via current_thread_index().
pub struct BufferPool {
    buffers: Vec<SyncUnsafeCell<(Vec<f64>, Vec<usize>)>>,
}

impl BufferPool {
    /// Create a new buffer pool for `num_threads` workers processing graphs with `node_count` nodes.
    /// Pre-allocates full-size neighbor weight buffers to avoid growth reallocations.
    pub fn new(node_count: usize, num_threads: usize) -> Self {
        let buffers: Vec<_> = (0..num_threads)
            .map(|_| SyncUnsafeCell::new((vec![0.0; node_count], Vec::with_capacity(4096))))
            .collect();
        BufferPool { buffers }
    }

    /// Reset buffers for reuse (clears Vec contents but keeps allocations).
    /// Only clears dirty_communities - neighbor_buf doesn't need zeroing since
    /// execute_shard only reads from indices it previously wrote to.
    pub fn reset(&self) {
        for i in 0..self.buffers.len() {
            unsafe {
                let buf_pair = &mut *(*self.buffers)[i].get();
                buf_pair.1.clear(); // Only clear dirty_communities, not neighbor_buf
            }
        }
    }

    /// Get internal buffer references (internal use only via inc_movement_parallel).
    fn as_ref(&self) -> &[SyncUnsafeCell<(Vec<f64>, Vec<usize>)>] {
        &self.buffers
    }
}

pub fn inc_movement_parallel(
    graph: &InMemoryGraph,
    active_nodes: &BitVec,
    node_to_community: &mut Vec<usize>,
    node_to_subcommunity: &[usize],
    community_degrees: &mut Vec<f64>,
    node_degrees: &[f64],
    twice_total_weight: f64,
    resolution_parameter: f64,
    buffer_pool: &BufferPool,
) -> (BitVec, BitVec, BitVec) {
    let active_nodes_vec: Vec<usize> = active_nodes.iter_ones().collect();

    let n = graph.node_count;

    // Shared atomic bitvecs — rayon worker threads write directly via fetch_or.
    // Rayon maintains a persistent thread pool so there is no spawn/join churn.
    let shared_changed = SharedBitVec::new(n);
    let shared_affected = SharedBitVec::new(n);
    let shared_next_active = SharedBitVec::new(n);

    // Reset buffer pool for reuse (keeps allocations, clears data)
    buffer_pool.reset();
    let buffers = buffer_pool.as_ref();
    let num_threads = buffers.len(); // Get thread count from buffer pool

    // Create immutable views for parallel access
    let node_to_community_view: &[usize] = node_to_community;
    let community_degrees_view: &[f64] = community_degrees;

    // Pre-chunk work by thread count for load balancing
    let chunk_size = (active_nodes_vec.len() + num_threads - 1) / num_threads;
    let chunks: Vec<&[usize]> = active_nodes_vec.chunks(chunk_size).collect();

    // Per-chunk result storage: SyncUnsafeCell to eliminate Mutex syscalls (20% overhead).
    // Safe: each chunk is processed by one thread only; no concurrent access.
    let results: Vec<SyncUnsafeCell<Option<ShardResult>>> = (0..chunks.len())
        .map(|_| SyncUnsafeCell::new(None))
        .collect();

    // Spawn one task per chunk and let rayon's work-stealing handle load balancing.
    // This avoids the problem where pre-batching causes fast threads to idle waiting
    // for slower threads at the scope barrier. Graph structure is uneven (varying degrees),
    // so work-stealing is essential to keep all threads busy.

    rayon::scope(|s| {
        for (chunk_idx, chunk) in chunks.into_iter().enumerate() {
            // Capture references for borrowing in the move closure
            let buffers = &buffers;
            let shared_changed = &shared_changed;
            let shared_affected = &shared_affected;
            let shared_next_active = &shared_next_active;
            let results = &results;

            s.spawn(move |_| {
                // Each spawn gets a unique thread from rayon's persistent pool.
                // Direct UnsafeCell access - zero syscall overhead!
                let thread_idx = rayon::current_thread_index().unwrap_or(0);
                let buf_pair = unsafe { &mut *(*buffers)[thread_idx % buffers.len()].get() };
                let (neighbor_buf, dirty_buf) = buf_pair;

                let result = execute_shard(
                    graph,
                    chunk,
                    node_to_community_view,
                    node_to_subcommunity,
                    community_degrees_view,
                    node_degrees,
                    twice_total_weight,
                    resolution_parameter,
                    shared_changed,
                    shared_affected,
                    shared_next_active,
                    neighbor_buf,
                    dirty_buf,
                );

                // Store result in per-chunk slot (no lock needed, no concurrent access)
                unsafe { *(*results)[chunk_idx].get() = Some(result) };
            });
        }
    });

    // Extract results in order - no sorting needed, indices match chunk order
    let results: Vec<_> = results
        .into_iter()
        .map(|cell| unsafe { (*cell.get()).take().unwrap() })
        .collect();

    // Apply sequential updates (nodes are disjoint across shards — no conflicts)
    for result in results {
        for (node, new_comm) in result.node_to_community_updates {
            node_to_community[node] = new_comm;
        }
        for (comm, delta) in result.community_degree_updates {
            community_degrees[comm] += delta;
        }
    }

    (
        shared_changed.into_bitvec(),
        shared_affected.into_bitvec(),
        shared_next_active.into_bitvec(),
    )
}

pub fn inc_refinement_parallel(
    graph: &InMemoryGraph,
    refined_nodes_sorted: &[usize],
    node_to_community: &[usize],
    node_to_subcommunity: &mut Vec<usize>,
    subcommunity_degrees: &mut HashMap<usize, f64>,
    subcommunity_sizes: &[usize],
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
                // O(1) singleton check via pre-computed sizes
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
