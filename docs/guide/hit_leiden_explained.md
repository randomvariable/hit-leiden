# HIT-Leiden Explained for Software Developers

HIT-Leiden is an algorithm for efficiently maintaining Leiden communities in large dynamic graphs.

## Core Pipeline

1. **Initialization**: Load the graph into a crate-native representation (e.g., `InMemoryGraph` using CSR).
2. **Mode Selection**: Choose between deterministic (default) or throughput mode.
3. **Hierarchical Updates**: For each level $p$ from 1 to $P$:
   - Apply graph updates $\Delta G_p$ to the supergraph $G_p$.
   - **Incremental Movement**: Reassign vertices to communities to maximize modularity, restricting the search space to affected vertices.
   - **Incremental Refinement**: Refine communities into sub-communities to preserve connectivity.
   - **Incremental Aggregation**: Merge sub-communities into supervertices to form the supergraph for the next level.
4. **Deferred Updates**: Update the community and sub-community mappings across all levels.
5. **Validation**: Validate invariants and mode-specific equivalence.
6. **Reporting**: Persist run/benchmark artifacts for reproducibility.
