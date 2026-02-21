# Crate API Contract: `hit_leiden`

## Scope
This contract defines the public crate-facing interfaces for running HIT-Leiden,
validating correctness, and producing benchmark-comparison artifacts.

## Public Concepts
- `GraphInput`: Graph payload accepted by the crate (format and representation are implementation-defined).
- `GraphSource`: `File` (default) or `Neo4jSnapshot` (explicit opt-in).
- `RunConfig`: Execution configuration.
- `RunMode`: `Deterministic` (default) or `Throughput` (explicit opt-in).
- `GraphBackend`: `InMemory` (default) or `Mmap` (explicit opt-in).
- `RunOutcome`: Partition result and execution metadata.
- `ValidationOutcome`: Invariant and equivalence checks.
- `BenchmarkOutcome`: Baseline-vs-candidate comparison summary.

## Behavioral Contract
1. Deterministic mode is default.
2. Throughput mode requires explicit opt-in.
3. Deterministic mode equivalence requires exact partition identity.
4. Throughput mode equivalence requires hard invariants and quality delta <= 0.1%.
5. `Mmap` backend is optional and explicit; it must satisfy the same mode-specific
  correctness policy as `InMemory`.
6. `Neo4jSnapshot` source is optional and explicit; release-gate runs must consume
  projected snapshots, not live per-step query execution.
7. Optional acceleration backends (including CUDA and ROCm targets) must fallback
  to pure Rust or another supported backend on incompatibility or failed
  validation.
8. Release-gate benchmark claims are valid only on pinned hardware profiles.

## Interface-Level Contract (logical)
- `run(graph, config) -> RunOutcome`
  - Must return complete partition assignment (or explicit failure).
  - Must report resolved graph backend and any backend fallback reason.
  - Must report resolved acceleration target (CPU/native/CUDA/ROCm) and any
    compatibility fallback reason.
- `project_from_neo4j(source_config, projection_config) -> GraphInput`
  - Must perform consistent snapshot projection with batched extraction.
  - Must emit projection metadata for reproducibility.
- `validate(reference, candidate, mode) -> ValidationOutcome`
  - Must enforce mode-specific equivalence policy.
- `compare_baseline(baseline_commit, candidate_commit, benchmark_suite, profile) -> BenchmarkOutcome`
  - Must report throughput gain and reproducibility metadata.

## Error Contract
- Errors are explicit, typed, and non-panicking at public boundaries.
- Fallback events (acceleration -> pure Rust, mmap -> in-memory when feasible)
 - Fallback events (acceleration -> pure Rust, mmap -> in-memory when feasible)
  must be surfaced in outcome metadata.

## Artifact Contract
- Output artifacts are crate-defined structures.
- Optional export/import adapters may support text or binary formats but are not required by core behavior.
