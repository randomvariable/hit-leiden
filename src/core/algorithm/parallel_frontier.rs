use crate::core::graph::in_memory::InMemoryGraph;
use bitvec::prelude::*;
use smallvec::SmallVec;
use std::sync::atomic::{AtomicU64, Ordering};

/// Cache-aligned atomic word to prevent false sharing.
/// Each word occupies its own cache line (64 bytes).
#[repr(align(64))]
struct CacheAligned<T>(T);

/// Lock-free bit vector backed by cache-aligned atomic 64-bit words.
///
/// Each word sits on its own cache line to prevent false sharing when
/// multiple threads write to adjacent bit ranges.
///
/// Supports concurrent `set` operations from multiple threads via `fetch_or`.
/// Iterate with `iter_ones()` or `any()` without conversion overhead.
pub struct SharedBitVec {
    words: Vec<CacheAligned<AtomicU64>>,
    len: usize,
}

impl SharedBitVec {
    pub fn new(len: usize) -> Self {
        let num_words = (len + 63) / 64;
        let words = (0..num_words)
            .map(|_| CacheAligned(AtomicU64::new(0)))
            .collect();
        Self { words, len }
    }

    /// Atomically set a single bit. Safe to call from multiple threads.
    #[inline(always)]
    pub fn set(&self, index: usize) {
        debug_assert!(index < self.len);
        let word_idx = index / 64;
        let bit_idx = index % 64;
        self.words[word_idx]
            .0
            .fetch_or(1u64 << bit_idx, Ordering::Relaxed);
    }

    /// Check if any bit is set (non-zero). Used for loop termination.
    #[inline]
    pub fn any(&self) -> bool {
        self.words.iter().any(|w| w.0.load(Ordering::Relaxed) != 0)
    }

    /// Iterate over set bit indices. Reads atomic words directly (no BitVec allocation).
    pub fn iter_ones(&self) -> impl Iterator<Item = usize> + '_ {
        self.words
            .iter()
            .enumerate()
            .flat_map(|(word_idx, aligned)| {
                let mut word = aligned.0.load(Ordering::Relaxed);
                let mut indices = SmallVec::<[usize; 8]>::new();
                while word != 0 {
                    let bit = word.trailing_zeros() as usize;
                    let global_idx = word_idx * 64 + bit;
                    if global_idx < self.len {
                        indices.push(global_idx);
                    }
                    word &= word.wrapping_sub(1); // clear lowest set bit
                }
                indices.into_iter()
            })
    }

    /// Consume into a `BitVec`, extracting all atomically-set bits.
    /// Call only after all writing threads have joined.
    /// Prefer `iter_ones()` to avoid allocation.
    pub fn into_bitvec(self) -> BitVec {
        let mut bv = bitvec![0; self.len];
        for (word_idx, aligned) in self.words.into_iter().enumerate() {
            let mut word = aligned.0.into_inner();
            while word != 0 {
                let bit = word.trailing_zeros() as usize;
                let global_idx = word_idx * 64 + bit;
                if global_idx < self.len {
                    bv.set(global_idx, true);
                }
                word &= word.wrapping_sub(1);
            }
        }
        bv
    }
}

/// Per-shard results that must be applied sequentially after all threads join.
pub struct ShardResult {
    pub node_to_community_updates: Vec<(usize, usize)>,
    pub community_degree_updates: Vec<(usize, f64)>,
}

/// Execute one shard of the incremental movement step.
///
/// Bits for `changed_nodes`, `affected_nodes`, and `next_active_nodes` are
/// written directly into shared atomic bitvecs (zero-copy merge).
/// Only the community-assignment and degree-delta updates are returned for
/// sequential application.
///
/// `neighbor_weight_buf` and `dirty_communities` are thread-local scratch
/// buffers reused across all nodes in the shard to avoid per-node allocation.
pub fn execute_shard(
    graph: &InMemoryGraph,
    shard: &[usize],
    node_to_community: &[usize],
    node_to_subcommunity: &[usize],
    community_degrees: &[f64],
    node_degrees: &[f64],
    twice_total_weight: f64,
    resolution_parameter: f64,
    changed_nodes: &SharedBitVec,
    affected_nodes: &SharedBitVec,
    next_active_nodes: &SharedBitVec,
    neighbor_weight_buf: &mut [f64],
    dirty_communities: &mut Vec<usize>,
) -> ShardResult {
    // Pre-allocate result Vecs to avoid growth reallocations during execution.
    // Estimate ~15% of nodes will change community (conservative for modularity optimization).
    // community_degree_updates needs 2x capacity (old and new community per change).
    let estimated_changes = (shard.len() * 15) / 100;
    let mut result = ShardResult {
        node_to_community_updates: Vec::with_capacity(estimated_changes),
        community_degree_updates: Vec::with_capacity(estimated_changes * 2),
    };

    for &current_node in shard {
        let current_community = node_to_community[current_node];
        let current_node_degree = node_degrees[current_node];
        let mut best_community = current_community;
        let mut best_modularity_gain = 0.0;
        let mut weight_to_current_community = 0.0;

        // Accumulate neighbor weights by community in flat buffer (O(degree))
        for (neighbor_node, w) in graph.neighbors(current_node) {
            let c = node_to_community[neighbor_node];
            if neighbor_weight_buf[c] == 0.0 {
                dirty_communities.push(c);
            }
            neighbor_weight_buf[c] += w;
            if c == current_community {
                weight_to_current_community += w;
            }
        }

        // Evaluate each neighbor community
        for &candidate_community in dirty_communities.iter() {
            if candidate_community == current_community {
                continue;
            }

            let weight_to_candidate = neighbor_weight_buf[candidate_community];
            let current_community_degree = community_degrees[current_community];
            let candidate_community_degree = community_degrees[candidate_community];

            let modularity_gain = (weight_to_candidate - weight_to_current_community)
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

        // Reset dirty entries (O(degree), not O(n))
        for &c in dirty_communities.iter() {
            neighbor_weight_buf[c] = 0.0;
        }
        dirty_communities.clear();

        // Record move if beneficial
        if best_modularity_gain > 0.0 {
            result
                .node_to_community_updates
                .push((current_node, best_community));
            result
                .community_degree_updates
                .push((current_community, -current_node_degree));
            result
                .community_degree_updates
                .push((best_community, current_node_degree));

            // Write directly to shared bitvecs (atomic OR, no per-shard alloc)
            changed_nodes.set(current_node);

            for (neighbor_node, _w) in graph.neighbors(current_node) {
                if node_to_community[neighbor_node] != best_community {
                    next_active_nodes.set(neighbor_node);
                }
                if node_to_subcommunity[current_node] == node_to_subcommunity[neighbor_node] {
                    affected_nodes.set(current_node);
                    affected_nodes.set(neighbor_node);
                }
            }
        }
    }

    result
}

