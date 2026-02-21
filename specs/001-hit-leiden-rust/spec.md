# Feature Specification: HIT-Leiden Reference-Accurate High-Performance Engine

**Feature Branch**: `001-hit-leiden-rust`
**Created**: 2026-02-19
**Status**: Draft
**Input**: User description: "Implement the feature specification based on the updated constitution. I want to build the highest possible Rust implementation of HIT-Leiden from this research paper https://arxiv.org/abs/2601.08554. Ensure correctness since i am not an expert. Integrating with C libraries for increased performance is ok."

## Clarifications

### Session 2026-02-19

- Q: What equivalence standard should accelerated mode meet versus standard mode? → A: High-throughput mode uses invariant + quality equivalence with quality metric delta ≤ 0.1%.
- Q: What should be the default execution mode? → A: Deterministic mode by default; explicit switch required for high-throughput non-deterministic mode.
- Q: What baseline should be used for performance comparison? → A: First correctness-validated in-repo implementation, frozen as a benchmark baseline commit.
- Q: What hardware policy should govern benchmark acceptance? → A: Release-gate benchmark acceptance requires pinned hardware profiles; other hardware runs are informational only.
- Q: What equivalence strictness should apply per execution mode? → A: Deterministic mode requires exact partition identity; high-throughput mode requires hard invariants plus quality bounds.
- Q: What delivery scope and artifact format should the project use? → A: Deliver as a Rust crate; internal and output data structures are implementation-defined for performance/correctness and are not constrained to JSON.
- Q: Should large-graph execution support optional memory-mapped graph storage? → A: Yes; provide an explicit opt-in mmap backend with deterministic correctness parity and safe fallback behavior.
- Q: Should Neo4j/Cypher data sources be supported directly? → A: Yes, via optional Neo4j/Cypher snapshot projection into crate-native graph structures; per-step live query execution is out of scope for release-gate mode.
- Q: Should GPU acceleration targets be included? → A: Yes; CUDA and ROCm are optional acceleration targets with the same mode-specific correctness policy and safe fallback to non-GPU backends.

### First PR Scope (2026-02-21)

- Deferred for first PR (release-gate evidence PR): pinned-hardware benchmark evidence, >=2x throughput release claims, and full release-gate reproducibility qualification.
- Deferred for first PR (release-gate evidence PR): optional native/GPU acceleration parity and fallback qualification.
- This PR focuses on correctness-first CPU baseline implementation, deterministic/throughput modes, and non-gating benchmark/reporting plumbing.

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.

  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Produce Correct Clustering Results (Priority: P1)

As a practitioner, I can run HIT-Leiden on an input graph and receive stable,
valid community assignments that match the paper-defined behavior on known
reference datasets.

**Why this priority**: Correctness is the minimum viable value. High performance is
only useful if clustering outputs are trustworthy.

**Independent Test**: Can be fully tested by running the engine on curated
reference graphs with expected invariants and verifying output validity,
determinism settings, and quality metrics.

**Acceptance Scenarios**:

1. **Given** a valid weighted or unweighted graph, **When** the user executes
  HIT-Leiden with default settings, **Then** the system returns a complete
  partition where every node belongs to exactly one community.
2. **Given** the same graph and same configuration, **When** the user reruns the
  algorithm in deterministic mode, **Then** the returned partition and reported
  quality score remain identical.
3. **Given** a published or curated benchmark graph with known behavior,
  **When** the user runs the algorithm, **Then** the result satisfies configured
  correctness checks and produces quality not worse than a defined baseline.

---

### User Story 2 - Optimize Throughput and Latency (Priority: P2)

As a performance-focused user, I can run HIT-Leiden at scale and observe faster
runtime and lower resource cost while preserving correctness guarantees.

**Why this priority**: Performance is the primary project objective after
correctness, and must be measurable and reproducible.

**Independent Test**: Can be tested independently by running reproducible
benchmarks against an explicit baseline and verifying speed/resource targets
without correctness regressions.

**Acceptance Scenarios**:

1. **Given** a benchmark suite and fixed measurement protocol, **When** a new
  optimized build is evaluated, **Then** benchmark reports include baseline,
  candidate, hardware/runtime context, and measured improvement.
2. **Given** a performance-sensitive workload, **When** optimization flags or
  execution modes are selected, **Then** the system reports run statistics and
  quality checks together so regressions are detectable.

---

### User Story 3 - Use Optional Native Acceleration Safely (Priority: P3)

As an advanced user, I can enable optional native acceleration paths for
additional performance while preserving
validated output behavior and safety constraints.

**Why this priority**: Optional native acceleration can unlock additional speed,
but only after baseline correctness and reproducible performance are established.

**Independent Test**: Can be tested independently by toggling acceleration on/off
for the same workloads and verifying equivalent clustering outcomes within defined
tolerance and explicit safety checks.

**Acceptance Scenarios**:

1. **Given** an environment with optional native acceleration available,
  **When** the user enables acceleration, **Then** the run completes with
  documented compatibility checks and reports equivalent correctness status to
  the non-accelerated run.
2. **Given** acceleration is unavailable or fails validation, **When** execution
  starts, **Then** the system cleanly falls back to the standard path and
  records the fallback reason.

---

### Edge Cases
- Empty graph input (0 nodes) and singleton graph input.
- Graphs with disconnected components and highly imbalanced component sizes.
- Graphs containing self-loops, duplicate edges, or inconsistent weights.
- Very dense graphs where memory pressure becomes significant.
- Non-deterministic execution mode differences versus deterministic mode.
- Native acceleration library present but incompatible at runtime.
- Native acceleration enabled but produces results outside accepted tolerance.
- Interrupted or timed-out runs where partial results could be misleading.

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: System MUST accept graph input in documented formats and reject
  malformed input with actionable error messages. The minimum supported input
  format MUST include weighted and unweighted edge-list text input
  (`<src> <dst> [weight]`), and the project MAY add additional crate-defined
  binary formats.
- **FR-002**: System MUST produce a full community assignment where each node has
  exactly one assigned community identifier.
- **FR-003**: System MUST provide a deterministic execution mode that yields
  repeatable outputs for identical inputs and configurations, and this mode MUST
  be the default.
- **FR-004**: System MUST provide a standard execution mode optimized for raw
  throughput while still emitting correctness validation outputs, and this mode
  MUST require explicit user selection.
- **FR-005**: System MUST compute and report clustering quality metrics used for
  acceptance and regression tracking.
- **FR-006**: System MUST include an automated correctness validation suite using
  curated datasets and invariant checks.
- **FR-007**: System MUST include a reproducible benchmark workflow that records
  dataset identity, run configuration, and execution context with results in a
  crate-defined format suitable for reproducibility.
- **FR-014**: System MUST define pinned hardware profiles for release-gate
  benchmark acceptance; runs on non-pinned hardware MUST be marked as
  informational and MUST NOT be used for release performance claims. (Deferred
  for first PR: release-gate evidence PR)
- **FR-008**: System MUST allow performance comparisons between baseline and
  candidate implementations under equivalent benchmark conditions, where baseline
  is the first correctness-validated in-repo implementation frozen as a
  benchmark baseline commit.
- **FR-009**: System MUST support optional native acceleration paths behind
  explicit configuration controls.
- **FR-010**: System MUST validate accelerated-path output equivalence against the
  standard path with mode-specific rules: deterministic mode MUST match exact
  partition identity, while high-throughput mode MUST pass all hard invariants
  and keep quality metric delta ≤ 0.1% versus the standard path for the same
  input and configuration.
- **FR-011**: System MUST fall back to the standard path when acceleration is
  unavailable, unsafe, or invalid for the workload.
- **FR-012**: System MUST expose run artifacts sufficient for non-expert review,
  including configuration summary, quality outputs, and pass/fail validation,
  using crate-defined structures and optional serialization formats.
- **FR-013**: System MUST bound configuration complexity by documenting defaults
  and allowing successful execution without advanced tuning. A default run MUST
  succeed with only the required graph source argument and no advanced
  performance or backend flags.
- **FR-015**: System MUST provide an optional memory-mapped graph backend for
  large datasets; default execution remains in-memory unless mmap is explicitly
  requested.
- **FR-016**: System MUST enforce correctness parity between in-memory and mmap
  backends under the same mode-specific equivalence policy.
- **FR-017**: System MUST fail safely with actionable diagnostics when mmap
  backend initialization or access checks fail, and allow fallback to in-memory
  execution when feasible.
- **FR-018**: System MUST provide an optional Neo4j/Cypher graph-source adapter
  that projects a consistent snapshot into crate-native graph structures before
  algorithm execution.
- **FR-019**: System MUST support bounded-memory, batched snapshot extraction from
  Neo4j/Cypher sources with explicit mapping rules for node IDs, relationships,
  and optional weights.
- **FR-020**: System MUST mark release-gate benchmark runs as ineligible when
  graph data is consumed via live per-step database queries instead of projected
  snapshot backends, and this eligibility decision MUST be emitted in benchmark
  artifacts with an explicit reason code. (Deferred for first PR:
  release-gate evidence PR)
- **FR-021**: System MUST support optional GPU acceleration targets for both CUDA
  and ROCm environments behind explicit configuration controls. (Deferred for
  first PR: follow-up acceleration PR)
- **FR-022**: System MUST enforce correctness parity for CUDA/ROCm accelerated
  runs using the same mode-specific equivalence policy as CPU backends, with
  explicit verification for both successful accelerated runs and fallback runs.
  (Deferred for first PR: follow-up acceleration PR)
- **FR-023**: System MUST fail safely with actionable diagnostics when CUDA/ROCm
  acceleration is unavailable or incompatible, and allow fallback to a supported
  non-GPU backend when feasible. (Deferred for first PR: follow-up acceleration
  PR)

### Assumptions

- The paper defines the target behavior and evaluation framing for HIT-Leiden.
- Curated benchmark datasets and expected validation rules are available or can be
  assembled from openly shareable graph data.
- Users may not be clustering experts, so default settings prioritize safe,
  correct outcomes over aggressive tuning.
- Native acceleration is optional and must never be required for functional use.
- Mmap backend is optional and must never be required for functional use.
- Neo4j/Cypher integration is optional and should default to snapshot projection
  rather than live per-step query execution.
- CUDA/ROCm acceleration is optional and must never be required for functional
  correctness.

### Key Entities *(include if feature involves data)*

- **Graph Dataset**: Input graph and metadata used for one or more executions;
  includes node/edge counts, optional weights, source identity, and integrity
  markers, optional mmap-compatible binary layout metadata, and optional source
  provenance for Neo4j/Cypher snapshot extraction.
- **Run Configuration**: User-selected or default algorithm settings controlling
  deterministic behavior, performance mode, and optional acceleration toggles.
- **Partition Result**: Community assignment output for all nodes plus summary
  statistics and quality metrics.
- **Validation Report**: Correctness checks and outcomes tied to a specific run,
  including invariant checks, equivalence checks, and pass/fail status.
- **Benchmark Record**: Reproducible performance result entry containing dataset,
  configuration, runtime context, baseline/candidate metrics, and comparison
  outcome.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: 100% of curated correctness-suite runs complete with all mandatory
  validity checks passing before any release candidate is accepted.
- **SC-002**: In deterministic mode, repeated runs on the same dataset and
  configuration produce identical partition outputs in 100 out of 100 trial
  pairs, and all default runs use deterministic mode unless explicitly
  overridden.
- **SC-003**: Candidate releases achieve at least 2x median throughput
  improvement over the initial in-repo frozen baseline commit on the primary
  benchmark suite while preserving correctness pass status. (Deferred for first
  PR: release-gate evidence PR)
- **SC-004**: Benchmark reports are reproducible by a second run operator in at
  least 95% of benchmark cases using only documented inputs and run metadata on
  pinned hardware profiles. (Deferred for first PR: release-gate evidence PR)
- **SC-005**: When optional native acceleration is enabled, at least 95% of
  supported benchmark cases meet mode-specific equivalence checks versus the
  standard path: deterministic runs with exact partition identity, and
  high-throughput runs with all hard invariants passing and quality metric delta
  ≤ 0.1%. (Deferred for first PR: follow-up acceleration PR)
- **SC-006**: A new user can execute a default end-to-end run and interpret the
  pass/fail validation outcome in under 15 minutes using project documentation.
- **SC-007**: Project documentation includes a complete mathematical description
  of HIT-Leiden (objective, update steps, termination criteria, and all symbols)
  reviewed as complete by at least one maintainer.
- **SC-008**: Project documentation includes a complete end-to-end explanation of
  the algorithm for a general software developer, and at least 90% of pilot
  readers can correctly explain the full workflow after reading it once.
- **SC-009**: On qualifying large datasets (defined for this release as graphs
  between 10M and 100M edges on pinned hardware profiles), mmap backend runs
  satisfy the same mode-specific correctness policy as in-memory runs and
  complete with no out-of-memory failure and with peak resident memory not
  exceeding 85% of available system RAM. (Deferred for first PR:
  release-gate evidence PR)
- **SC-010**: Neo4j/Cypher snapshot-projected runs satisfy the same mode-specific
  correctness policy as equivalent in-memory dataset runs on at least 95% of
  qualification cases. (Deferred for first PR: release-gate evidence PR)
- **SC-011**: On qualified hardware, CUDA and ROCm accelerated runs each satisfy
  mode-specific correctness policy on at least 95% of qualification cases and
  produce benchmark artifacts comparable to CPU baselines. (Deferred for first
  PR: follow-up acceleration PR)
