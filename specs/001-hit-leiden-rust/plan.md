# Implementation Plan: HIT-Leiden Reference-Accurate High-Performance Engine

**Branch**: `001-hit-leiden-rust` | **Date**: 2026-02-19 | **Spec**: `/specs/001-hit-leiden-rust/spec.md`
**Input**: Feature specification from `/specs/001-hit-leiden-rust/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Implement a correctness-first, high-performance Rust HIT-Leiden engine with
deterministic mode as default and explicit high-throughput mode, reproducible
benchmarking against a frozen in-repo baseline commit, and optional native
acceleration plus optional mmap graph backend behind strict validation and
fallback rules.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust stable (>= 1.76)
**Primary Dependencies**: `rayon`, `clap`, `thiserror`, `smallvec`, `bitvec`, optional FFI (`cxx` or `bindgen`-based backend), optional GPU backends (CUDA/ROCm bindings), optional Neo4j/Cypher adapter (Bolt-capable Rust driver)
**Storage**: In-memory crate-native structures by default, with optional memory-mapped graph backend, optional Neo4j/Cypher snapshot projection source, and optional file adapters for datasets and benchmark artifacts
**Testing**: `cargo test`, property/invariant tests (`proptest`), benchmark harness (`criterion`) plus integration golden datasets
**Target Platform**: Linux x86_64 CPU (release-gate pinned profiles), optional CUDA-capable GPU targets, optional ROCm-capable GPU targets, portable Rust fallback on other platforms
**Project Type**: single Rust crate
**Performance Goals**: >=2x median throughput over frozen baseline commit on pinned hardware profiles while preserving correctness pass status
**Constraints**: Deterministic mode default; high-throughput mode explicit opt-in; deterministic mode exact partition identity; throughput mode invariants + quality delta <=0.1%; mmap backend explicit opt-in with correctness parity to in-memory backend; Neo4j/Cypher via snapshot projection for release-gate runs; CUDA/ROCm optional with correctness parity and fallback requirements; non-pinned hardware results are informational only
**Scale/Scope**: Initial release targets a qualifying large-dataset tier of 10M–100M edges on pinned release hardware, with bounded memory growth, no out-of-memory failures, peak resident memory <=85% of system RAM, and reproducible benchmark evidence

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Performance Evidence**: Plan defines how performance will be measured for changed
  paths (baseline + candidate, comparable conditions).
- **Correctness Coverage**: Plan defines tests for algorithmic behavior and edge cases
  affected by the feature.
- **Benchmark Reproducibility**: Plan identifies benchmark inputs, parameters, and
  runtime/hardware context to allow reproduction.
- **Safety & Determinism**: Plan states whether `unsafe` is used (with invariants) and
  documents determinism expectations for outputs.
- **Surface Area Control**: Plan justifies any new public API, configuration, or
  dependency.

Initial gate assessment:
- ✅ Performance Evidence: Benchmark methodology, baseline commit policy, and pinned-hardware release gate defined.
- ✅ Correctness Coverage: Deterministic identity checks, invariant suite, and throughput equivalence thresholds defined.
- ✅ Benchmark Reproducibility: Dataset identity, run config, hardware/runtime manifest, and fixed baseline required.
- ✅ Safety & Determinism: Deterministic default; explicit throughput mode; optional FFI behind validation/fallback.
- ✅ Surface Area Control: Single-project architecture, minimal dependencies, optional acceleration as feature-gated backend.

Mmap backend policy:
- Optional and explicit opt-in only.
- Must satisfy identical mode-specific correctness policy as in-memory backend.
- Operational qualification for release evidence is defined on 10M–100M edge graphs on pinned hardware with no out-of-memory failures and peak resident memory <=85% of system RAM.
- Must surface actionable diagnostics and safe fallback behavior when mmap setup fails.

Neo4j/Cypher source policy:
- Optional and explicit opt-in only.
- Release-gate runs must use projected snapshots, not per-step live query execution.
- Snapshot extraction must support batched transfer and deterministic ID/edge mapping.

GPU target policy (CUDA/ROCm):
- Optional and explicit opt-in only.
- CUDA and ROCm accelerated paths must satisfy identical mode-specific correctness policy as CPU paths.
- GPU initialization and compatibility failures must produce actionable diagnostics and fallback behavior.

## Project Structure

### Documentation (this feature)

```text
specs/001-hit-leiden-rust/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
src/
├── cli/
├── core/
│   ├── graph/
│   ├── partition/
│   ├── algorithm/
│   └── validation/
├── accel/
├── benchmark/
└── docs/

tests/
├── unit/
├── integration/
├── property/
└── contract/

benchmarks/
└── criterion/

datasets/
└── curated/
```

**Structure Decision**: Single-project Rust layout selected to minimize
cross-component overhead and keep hot-path algorithm code, validation, and
benchmarking tightly coupled for reproducibility and performance tuning.

## Phase 0 Output: Research

See `research.md` for final decisions on graph/partition data layouts,
parallelization strategy, deterministic vs throughput behavior,
native-acceleration guardrails, validation strategy, benchmark policy, and
documentation approach.

## Phase 1 Output: Design & Contracts

- `data-model.md`: Defines graph/run/partition/validation/benchmark entities,
  field constraints, and lifecycle transitions.
- `contracts/crate_api.md`: Public crate API contract for execution,
  validation, benchmarking, and optional acceleration controls.
- `quickstart.md`: Developer workflow to run correctness checks, reproducible
  benchmarks, and documentation outputs.

## Post-Design Constitution Check

- ✅ Performance as a Feature: Design includes explicit benchmark gate, baseline, and pinned-hardware policy.
- ✅ Correctness Before Optimization: Validation-first lifecycle and mode-specific equivalence constraints included.
- ✅ Reproducible Benchmarks: Contract and quickstart require metadata-complete benchmark records.
- ✅ Memory Safety and Determinism: Safe Rust default, deterministic default mode, guarded optional acceleration.
- ✅ Minimal Surface Area: Single-project scope, optional acceleration backend, focused contracts and artifacts.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Optional native acceleration backend | Needed for targeted hot-kernel speedups while preserving safe Rust default | Hard native dependency would violate portability and safety constraints |
