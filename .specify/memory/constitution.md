<!--
Sync Impact Report
- Version change: 1.0.0 → 1.2.0
- Modified principles:
  - I. Performance as a Feature → I. Incremental Updates Are Non-Negotiable
  - N/A → II. External Graph Processing
  - II. Correctness Before Optimization → III. Hierarchical Communities
  - III. Reproducible Benchmarks → IV. Incremental Output
  - IV. Memory Safety and Determinism → V. Modest Hardware and Scale-Out
  - V. Minimal Surface Area → VI. Throughput by Default
  - N/A → VII. Algorithm Agility
  - N/A → VIII. Correctness
  - N/A → IX. Memory Safety
  - N/A → X. Minimal Surface Area
- Modified sections:
  - Technical Standards (added performance evidence requirement)
  - Preamble (added project purpose and link to GOALS.md)
- Removed sections:
  - None (all v1.0.0 content preserved, reorganised to match GOALS.md priorities)
- Templates requiring updates:
  - ⚠ pending: .specify/templates/plan-template.md
  - ⚠ pending: .specify/templates/tasks-template.md
  - ⚠ pending: .specify/templates/spec-template.md
- Follow-up TODOs:
  - Update speckit templates to reference new principle numbering
-->

# hit-leiden Constitution

This project provides fast, incremental community detection for GraphRAG systems
running on modest hardware. See the [project README](../../README.md) for the
full project goals and priorities. This constitution governs how the project is developed.

## Core Principles

### I. Incremental Updates Are Non-Negotiable
When the knowledge graph changes, only affected communities MUST be reprocessed.
No design decision, optimisation, or architectural change may introduce a
requirement for full re-clustering. A change that breaks incrementality is a
breaking change, regardless of any other benefit it provides.

Rationale: the entire value of this project depends on avoiding full re-clustering
on every graph update. Without incrementality, community detection cannot keep pace
with document ingestion on modest hardware.

### II. External Graph Processing
The system MUST operate against graphs that live outside the process, typically in
a graph database. The external database is authoritative. Working copies of
subgraphs or delta windows MAY be held in process memory, but the system SHOULD
minimise in-process graph state. Changes that assume full in-process graph
ownership as the only mode of operation MUST NOT be accepted for core paths.

Rationale: in a RAG deployment, the knowledge graph is managed by a graph database
that receives document-derived updates. Community detection must operate against
this external store as the source of truth.

### III. Hierarchical Communities
Communities MUST be organised hierarchically. Larger communities MUST be composed
of smaller sub-communities so that each level can be summarised within LLM context
window limits. Changes that flatten the hierarchy or prevent recursive
summarisation MUST NOT be accepted without an equivalent alternative.

Rationale: LLM context windows impose hard limits on summarisation input size.
Hierarchical structure enables recursive summarisation from leaves upward.

### IV. Incremental Output
The system MUST support emitting stabilised communities as soon as they are ready,
rather than requiring the entire graph to converge before producing output. Changes
that force batch-only output MUST justify the regression and provide a remediation
path.

Rationale: incremental output enables pipelined workflows where downstream LLM
summarisation begins immediately, keeping end-to-end ingestion latency low.

### V. Modest Hardware and Scale-Out
The system MUST run correctly on consumer-grade machines. The architecture MUST
support distributing work across multiple modest nodes. Changes that introduce
hard dependencies on GPU clusters, high-memory servers, or single-machine-only
execution MUST NOT be accepted for core paths.

Rationale: the project targets RAG deployments on laptops, small servers, and
commodity cloud instances, with the ability to scale out by adding nodes.

### VI. Throughput by Default
High-throughput parallel execution MUST be the default mode. Deterministic mode
is available as an option for debugging and testing. Changes that make deterministic
mode the default or degrade throughput-mode performance MUST include justification
and measurable evidence.

Rationale: the project serves document ingestion pipelines where speed of update
matters more than bit-exact reproducibility.

### VII. Algorithm Agility
The architecture MUST NOT be permanently coupled to any single algorithm. The
current implementation uses HIT-Leiden, but alternative or supplementary approaches
that deliver correct, incremental, hierarchical community updates MUST be
evaluable without rewriting core infrastructure.

Rationale: better algorithms may emerge. The project's value is in solving the
incremental GraphRAG community detection problem, not in any particular algorithm.

### VIII. Correctness
Community assignments MUST be trustworthy. Changes to clustering logic, numeric
routines, or convergence behaviour MUST include tests that validate expected
outcomes and edge cases. Correctness MUST be established before optimisation is
accepted.

Rationale: summaries built on incorrect community structure propagate errors
through the entire RAG system.

### IX. Memory Safety
Unsafe Rust MUST NOT be introduced unless a measurable requirement cannot be met
with safe Rust, and any accepted unsafe block MUST include explicit invariants
and tests.

Rationale: safety reduces operational risk and debugging cost.

### X. Minimal Surface Area
Public APIs, configuration knobs, and dependencies MUST remain minimal and
purpose-built. New abstractions MUST demonstrate clear reuse or complexity
reduction.

Rationale: constrained interfaces improve maintainability and long-term
performance work.

## Technical Standards

- Implementation language MUST be stable Rust.
- Dependencies MUST be justified by measurable capability or maintenance value.
- Core paths MUST avoid unnecessary allocation, copying, and dynamic dispatch
  where these affect measured performance goals.
- Numerical assumptions, convergence criteria, and data format expectations MUST
  be documented in feature specs and developer-facing docs.
- Pull requests touching hot paths, data layouts, parallelism, or allocation
  behaviour MUST include measurable performance evidence.

## Development Workflow

- Feature specs MUST define measurable success criteria for correctness and
  performance.
- Implementation plans MUST pass Constitution Check gates before design execution.
- Task lists MUST include verification tasks for tests and benchmarks when
  relevant.
- Code review MUST include constitution compliance verification before merge.

## Governance

This constitution overrides conflicting local conventions for this repository.

Amendments require:
1. A documented proposal describing the rule change and rationale.
2. Explicit updates to affected templates and workflow guidance in the same change.
3. Reviewer approval from at least one maintainer.

Versioning policy:
- MAJOR: incompatible governance or principle removals/redefinitions.
- MINOR: new principle/section or materially expanded guidance.
- PATCH: clarifications, wording improvements, and non-semantic refinements.

Compliance review expectations:
- Every pull request MUST state constitution compliance or justified exceptions.
- Exceptions MUST include scope, risk, and follow-up remediation.

**Version**: 1.2.0 | **Ratified**: 2026-02-19 | **Last Amended**: 2026-02-21
