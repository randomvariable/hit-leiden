# Tasks: HIT-Leiden Reference-Accurate High-Performance Engine

**Input**: Design documents from `/specs/001-hit-leiden-rust/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests & Benchmarks**: Verification tasks are REQUIRED by constitution for impacted correctness and performance paths.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize crate skeleton and developer tooling.

- [x] T001 Initialize crate manifest and workspace metadata in Cargo.toml
- [x] T002 Create source tree skeleton in src/{lib.rs,cli/mod.rs,core/mod.rs,accel/mod.rs,benchmark/mod.rs}
- [x] T003 [P] Configure rustfmt and clippy settings in rustfmt.toml and clippy.toml
- [x] T004 [P] Add test/benchmark harness dependencies in Cargo.toml
- [x] T005 [P] Create base test folders and module files in tests/{unit,integration,property,contract}/mod.rs
- [x] T006 [P] Add developer docs index in docs/README.md

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core shared abstractions and infrastructure required by all stories.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T007 Define core domain types (`GraphDataset`, `RunConfiguration`, `RunOutcome`) in src/core/types.rs
- [x] T008 Define backend/source enums and fallback metadata in src/core/backend.rs
- [x] T009 [P] Define typed error model and diagnostics surface in src/core/error.rs
- [x] T010 [P] Implement configuration loading/validation module in src/core/config.rs
- [x] T011 [P] Implement artifact/report model types in src/core/report.rs
- [x] T012 Define crate API trait surfaces from contract in src/lib.rs
- [x] T013 [P] Implement graph source abstraction (`File`, `Neo4jSnapshot`) in src/core/graph/source.rs
- [x] T014 [P] Implement graph backend abstraction (`InMemory`, `Mmap`) in src/core/graph/backend.rs
- [x] T015 [P] Implement acceleration abstraction (`PureRust`, `Native`, `CUDA`, `ROCm`) in src/accel/target.rs
- [x] T016 Implement fallback orchestration and backend resolution in src/core/runtime/resolver.rs
- [x] T017 [P] Add baseline benchmark metadata schema in src/benchmark/manifest.rs
- [x] T018 [P] Add pinned hardware profile schema in src/benchmark/hardware_profile.rs

**Checkpoint**: Foundation ready—user story phases can start.

---

## Phase 3: User Story 1 - Produce Correct Clustering Results (Priority: P1) 🎯 MVP

**Goal**: Deliver correct, deterministic-by-default HIT-Leiden clustering with strict validation.

**Independent Test**: Run deterministic mode twice on curated graphs and verify exact partition identity + invariant pass.

### Verification for User Story 1

- [x] T019 [P] [US1] Add contract test for run/validate API behavior in tests/contract/test_run_validate.rs
- [x] T020 [P] [US1] Add integration test for deterministic replay identity in tests/integration/test_deterministic_identity.rs
- [x] T021 [P] [US1] Add property tests for partition invariants in tests/property/test_partition_invariants.rs

### Implementation for User Story 1

- [x] T022 [P] [US1] Implement in-memory CSR graph representation in src/core/graph/in_memory.rs
- [x] T023 [P] [US1] Implement partition state data structures in src/core/partition/state.rs
- [x] T024 [US1] Implement deterministic traversal/tie-break rules in src/core/algorithm/deterministic.rs
- [x] T025 [US1] Implement core HIT-Leiden execution pipeline in src/core/algorithm/hit_leiden.rs
- [x] T026 [US1] Implement invariant validation engine in src/core/validation/invariants.rs
- [x] T027 [US1] Implement mode-specific equivalence validator in src/core/validation/equivalence.rs
- [x] T028 [US1] Wire default deterministic mode execution in src/lib.rs
- [x] T029 [US1] Implement CLI run command for deterministic workflow in src/cli/run.rs
- [x] T030 [US1] Emit run/validation artifacts in crate-defined format in src/core/report/writer.rs
- [x] T031 [US1] Add mathematical algorithm specification document in docs/math/hit_leiden_spec.md
- [x] T032 [US1] Add developer-oriented algorithm walkthrough in docs/guide/hit_leiden_explained.md

**Checkpoint**: US1 complete and independently demonstrable.

---

## Phase 4: User Story 2 - Optimize Throughput and Latency (Priority: P2)

**Goal**: Add explicit high-throughput mode with reproducible benchmark framework and baseline comparison.

**Independent Test**: Run benchmark suite on pinned hardware and verify >=2x median throughput versus frozen baseline with correctness retained.

### Verification for User Story 2

- [x] T033 [P] [US2] Add integration test for throughput mode equivalence bounds in tests/integration/test_throughput_equivalence.rs
- [x] T034 [P] [US2] Add benchmark reproducibility test harness in tests/integration/test_benchmark_reproducibility.rs
- [x] T035 [P] [US2] Add contract test for baseline comparison API in tests/contract/test_compare_baseline.rs
- [x] T063 [P] [US2] Add integration test for release-gate ineligibility when using live per-step DB queries in tests/integration/test_release_gate_live_query_ineligible.rs

### Implementation for User Story 2

MUST USE /home/naadir/go/src/github.com/randomvariable/hit-leiden/docs/2601.08554.md

- [x] T036 [P] [US2] Implement explicit throughput mode scheduling in src/core/algorithm/throughput.rs
- [x] T037 [US2] Implement thread-local frontier parallel execution in src/core/algorithm/parallel_frontier.rs
- [x] T038 [US2] Implement benchmark runner against frozen baseline in src/benchmark/runner.rs
- [x] T039 [US2] Implement pinned hardware profile enforcement in src/benchmark/release_gate.rs
- [x] T040 [US2] Implement benchmark comparison/report generation in src/benchmark/compare.rs
- [x] T041 [US2] Implement CLI benchmark commands in src/cli/benchmark.rs
- [x] T042 [US2] Add benchmark suite definitions in benchmarks/criterion/hit_leiden_suite.rs
- [x] T066 [US2] Implement release-gate eligibility reason codes for snapshot vs live-query graph sources in src/benchmark/release_gate.rs

**Checkpoint**: US2 complete and independently demonstrable.

---

## Phase 5: User Story 3 - Use Optional Native Acceleration Safely (Priority: P3)

**Goal**: Add optional acceleration targets (native, CUDA, ROCm), mmap backend, and Neo4j snapshot source with safe fallback.

**Independent Test**: Enable each optional target/source/backend and verify mode-specific correctness parity or safe fallback behavior.

### Verification for User Story 3

- [x] T043 [P] [US3] Add integration tests for mmap backend parity in tests/integration/test_mmap_parity.rs
- [x] T044 [P] [US3] Add integration tests for Neo4j snapshot projection parity in tests/integration/test_neo4j_snapshot_parity.rs
- [x] T064 [P] [US3] Add integration tests for CUDA successful-run parity in deterministic and throughput modes in tests/integration/test_cuda_parity.rs
- [x] T065 [P] [US3] Add integration tests for ROCm successful-run parity in deterministic and throughput modes in tests/integration/test_rocm_parity.rs
- [x] T045 [P] [US3] Add integration tests for CUDA fallback behavior in tests/integration/test_cuda_fallback.rs
- [x] T046 [P] [US3] Add integration tests for ROCm fallback behavior in tests/integration/test_rocm_fallback.rs
- [x] T047 [P] [US3] Add contract test for source/backends resolution metadata in tests/contract/test_backend_resolution.rs

### Implementation for User Story 3

- [x] T048 [P] [US3] Implement mmap graph backend in src/core/graph/mmap.rs
- [x] T049 [US3] Implement mmap capability checks and diagnostics in src/core/graph/mmap_probe.rs
- [x] T050 [US3] Implement Neo4j/Cypher snapshot projection adapter in src/core/graph/neo4j_snapshot.rs
- [x] T051 [US3] Implement batched Neo4j extraction and mapping rules in src/core/graph/neo4j_mapping.rs
- [x] T052 [US3] Implement optional native acceleration backend in src/accel/native.rs
- [x] T053 [US3] Implement optional CUDA backend target in src/accel/cuda.rs
- [x] T054 [US3] Implement optional ROCm backend target in src/accel/rocm.rs
- [x] T055 [US3] Implement acceleration compatibility probes and fallback resolver in src/accel/probe.rs
- [x] T056 [US3] Integrate source/backend/accel fallback orchestration in src/core/runtime/orchestrator.rs
- [x] T057 [US3] Extend CLI source/backend flags for mmap/neo4j/cuda/rocm in src/cli/options.rs

**Checkpoint**: US3 complete and independently demonstrable.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final quality, documentation alignment, and release readiness.

- [x] T058 [P] Add release-gate runbook for pinned hardware and baseline commit process in docs/runbooks/release_gate.md
- [x] T059 [P] Add troubleshooting guide for fallback diagnostics in docs/runbooks/fallback_diagnostics.md
- [x] T060 Validate quickstart end-to-end scenarios in specs/001-hit-leiden-rust/quickstart.md
- [x] T061 Run full test matrix and benchmark smoke suite via CI workflow in .github/workflows/ci.yml
- [x] T062 Final API/contract consistency review against crate contract in specs/001-hit-leiden-rust/contracts/crate_api.md
- [x] T067 Add integration test proving default run succeeds with only required graph-source argument in tests/integration/test_default_config_minimal_args.rs

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: starts immediately.
- **Foundational (Phase 2)**: depends on Phase 1 completion; blocks all user stories.
- **User Stories (Phases 3–5)**: depend on Phase 2 completion.
- **Polish (Phase 6)**: depends on all desired user stories being complete.

### User Story Dependencies

- **US1 (P1)**: starts after Phase 2; no dependencies on other stories.
- **US2 (P2)**: starts after US1 core interfaces are stable (T028, T029).
- **US3 (P3)**: starts after US1 validators are complete (T026, T027) and US2 benchmark framework exists (T038, T039).

### Within Each User Story

- Verification tasks execute before implementation tasks and must fail or demonstrate gap before feature completion.
- Data structures before algorithm integration.
- Backend/source implementations before orchestration and CLI exposure.

### Parallel Opportunities

- [P] tasks in Phase 1 and 2 can run in parallel where file paths do not overlap.
- Verification tasks in each user story can run in parallel.
- Optional backend implementations in US3 (`cuda.rs`, `rocm.rs`, `native.rs`, `mmap.rs`, `neo4j_snapshot.rs`) can run in parallel.

---

## Parallel Example: User Story 3

```bash
# Run optional backend verification tasks in parallel:
Task: "Add integration tests for mmap backend parity in tests/integration/test_mmap_parity.rs"
Task: "Add integration tests for Neo4j snapshot projection parity in tests/integration/test_neo4j_snapshot_parity.rs"
Task: "Add integration tests for CUDA fallback behavior in tests/integration/test_cuda_fallback.rs"
Task: "Add integration tests for ROCm fallback behavior in tests/integration/test_rocm_fallback.rs"

# Implement optional backends in parallel:
Task: "Implement mmap graph backend in src/core/graph/mmap.rs"
Task: "Implement optional CUDA backend target in src/accel/cuda.rs"
Task: "Implement optional ROCm backend target in src/accel/rocm.rs"
Task: "Implement Neo4j/Cypher snapshot projection adapter in src/core/graph/neo4j_snapshot.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phases 1 and 2.
2. Complete Phase 3 (US1).
3. Validate deterministic correctness + invariant suite.
4. Demo crate API + deterministic run path.

### Incremental Delivery

1. Deliver US1 as correctness baseline.
2. Deliver US2 for reproducible performance gains.
3. Deliver US3 optional backends and sources with safe fallbacks.
4. Finish with Phase 6 polish/release-readiness tasks.

### Parallel Team Strategy

1. Team A: core algorithm/validation (US1).
2. Team B: benchmarks and release-gate profiles (US2).
3. Team C: optional backends and source adapters (US3).

---

## Notes

- Every task follows strict checklist format with task ID and file path.
- [P] marker indicates no dependency on incomplete tasks in overlapping files.
- Verification coverage is mandatory due constitution + explicit correctness/performance requirements.
- Suggested MVP scope: **US1 only** for first deliverable.
