# Data Model: HIT-Leiden Engine

## Entity: GraphDataset
- Fields:
  - `dataset_id` (string, required, unique)
  - `source_uri` (string, required)
  - `is_weighted` (boolean, required)
  - `node_count` (integer, required, >= 0)
  - `edge_count` (integer, required, >= 0)
  - `checksum` (string, required)
  - `format` (enum: edge_list, csr_binary, required)
  - `mmap_compatible` (boolean, required)
  - `mmap_path` (string, optional)
  - `source_type` (enum: file, neo4j_snapshot, required)
  - `source_snapshot_id` (string, optional)
- Relationships:
  - One `GraphDataset` can have many `RunExecution` records.
- Validation:
  - `checksum` must match imported graph payload.
  - `edge_count == 0` allowed only for empty/singleton edge cases.

## Entity: RunConfiguration
- Fields:
  - `config_id` (string, required, unique)
  - `mode` (enum: deterministic, throughput, required)
  - `acceleration_enabled` (boolean, required)
  - `seed` (integer, optional; required for deterministic replay)
  - `max_iterations` (integer, required, > 0)
  - `quality_tolerance` (float, required, default 0.001)
  - `pinned_profile_id` (string, optional; required for release-gate benchmarks)
  - `graph_backend` (enum: in_memory, mmap, required, default in_memory)
  - `graph_source` (enum: file, neo4j_snapshot, required, default file)
- Relationships:
  - One `RunConfiguration` can be reused across many `RunExecution` records.
- Validation:
  - `mode=deterministic` implies fixed traversal and stable reductions.
  - `mode=throughput` requires explicit opt-in (not default).
  - `graph_backend=mmap` requires mmap-compatible dataset metadata.
  - `graph_source=neo4j_snapshot` requires snapshot extraction metadata.

## Entity: RunExecution
- Fields:
  - `run_id` (string, required, unique)
  - `dataset_id` (fk -> GraphDataset, required)
  - `config_id` (fk -> RunConfiguration, required)
  - `started_at` (timestamp, required)
  - `completed_at` (timestamp, optional)
  - `status` (enum: running, succeeded, failed, required)
  - `backend` (enum: pure_rust, native_accel, cuda_accel, rocm_accel, required)
  - `graph_backend_resolved` (enum: in_memory, mmap, required)
  - `graph_source_resolved` (enum: file, neo4j_snapshot, required)
  - `fallback_reason` (string, optional)
- Relationships:
  - One `RunExecution` has one `PartitionResult` and one `ValidationReport`.
- State transitions:
  - `running -> succeeded|failed`
  - On acceleration failure: backend switches to `pure_rust` with `fallback_reason` recorded.
  - On mmap init/access failure: graph backend may switch to `in_memory` when feasible with `fallback_reason` recorded.
  - On CUDA/ROCm init or compatibility failure: backend may switch to `pure_rust` or non-GPU acceleration when feasible with `fallback_reason` recorded.

## Entity: SourceProjectionReport
- Fields:
  - `projection_id` (string, required, unique)
  - `source_type` (enum: neo4j_snapshot, required)
  - `snapshot_id` (string, required)
  - `node_rows_read` (integer, required, >= 0)
  - `edge_rows_read` (integer, required, >= 0)
  - `batched` (boolean, required)
  - `mapping_rules_version` (string, required)
- Validation:
  - Projection must be completed before algorithm run starts.

## Entity: PartitionResult
- Fields:
  - `run_id` (fk -> RunExecution, required, unique)
  - `node_to_community` (array<int>, required)
  - `community_count` (integer, required, > 0 unless empty graph)
  - `quality_score` (float, required)
  - `iteration_count` (integer, required, >= 1)
- Validation:
  - Each node maps to exactly one community.
  - Deterministic mode replay must produce exact same partition mapping.

## Entity: ValidationReport
- Fields:
  - `run_id` (fk -> RunExecution, required, unique)
  - `hard_invariants_passed` (boolean, required)
  - `deterministic_identity_passed` (boolean, optional)
  - `quality_delta_vs_reference` (float, optional)
  - `equivalence_passed` (boolean, required)
  - `notes` (string, optional)
- Validation rules:
  - Deterministic mode: `deterministic_identity_passed=true` required.
  - Throughput mode: `hard_invariants_passed=true` and `quality_delta_vs_reference <= 0.001` required.

## Entity: BenchmarkRecord
- Fields:
  - `benchmark_id` (string, required, unique)
  - `baseline_commit` (string, required)
  - `candidate_commit` (string, required)
  - `run_ids` (array<string>, required)
  - `hardware_profile_id` (string, required for release-gate)
  - `median_throughput_gain` (float, required)
  - `reproducible` (boolean, required)
  - `release_gate_eligible` (boolean, required)
- Validation:
  - `release_gate_eligible=true` only when `hardware_profile_id` is pinned and all run manifests are complete.

## Representation Notes
- This project is a Rust crate, so internal data structures are implementation-defined
  and chosen for performance/correctness (not constrained to JSON-oriented schemas).
- Serialization format is optional and adapter-driven; core algorithm behavior and
  validation do not depend on any specific interchange format.
