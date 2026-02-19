<!--
Sync Impact Report
- Version change: N/A → 1.0.0
- Modified principles:
  - N/A → I. Performance as a Feature
  - N/A → II. Correctness Before Optimization
  - N/A → III. Reproducible Benchmarks
  - N/A → IV. Memory Safety and Determinism
  - N/A → V. Minimal Surface Area
- Added sections:
  - Technical Standards
  - Development Workflow
- Removed sections:
  - None
- Templates requiring updates:
  - ✅ updated: .specify/templates/plan-template.md
  - ✅ updated: .specify/templates/tasks-template.md
  - ✅ updated: .specify/templates/spec-template.md
  - ⚠ pending: .specify/templates/commands/*.md (directory not present)
- Follow-up TODOs:
  - None
-->

# hit-leiden Constitution

## Core Principles

### I. Performance as a Feature
All production changes MUST preserve or improve end-to-end throughput and latency for
target workloads derived from the Leiden-paper implementation scope. Pull requests
MUST include measurable performance evidence when touching hot paths, data layouts,
parallelism, or allocation behavior.

Rationale: this project exists to deliver a high-performance Rust implementation, so
performance regressions are functional regressions.

### II. Correctness Before Optimization
Behavioral correctness MUST be established before optimization is accepted. Any change
to clustering logic, numeric routines, or convergence behavior MUST include tests that
validate expected outcomes and edge cases.

Rationale: invalid fast results are unacceptable for scientific and production use.

### III. Reproducible Benchmarks
Benchmark inputs, execution parameters, and hardware context MUST be documented so
results are reproducible by another contributor. Benchmark comparisons MUST report
baseline and candidate measurements under equivalent conditions.

Rationale: reproducibility prevents misleading claims and enables reliable tuning.

### IV. Memory Safety and Determinism
Unsafe Rust MUST NOT be introduced unless a measurable requirement cannot be met with
safe Rust, and any accepted unsafe block MUST include explicit invariants and tests.
Given identical input and configuration, the implementation SHOULD produce deterministic
outputs; deviations MUST be documented and justified.

Rationale: safety and deterministic behavior reduce operational risk and debugging cost.

### V. Minimal Surface Area
Public APIs, configuration knobs, and dependencies MUST remain minimal and purpose-built
for the core Leiden implementation goals. New abstractions MUST demonstrate clear reuse
or complexity reduction.

Rationale: constrained interfaces improve maintainability and long-term performance work.

## Technical Standards

- Implementation language MUST be stable Rust.
- Dependencies MUST be justified by measurable capability or maintenance value.
- Core paths MUST avoid unnecessary allocation, copying, and dynamic dispatch where
  these affect measured performance goals.
- Numerical assumptions, convergence criteria, and data format expectations MUST be
  documented in feature specs and developer-facing docs.

## Development Workflow

- Feature specs MUST define measurable success criteria for correctness and performance.
- Implementation plans MUST pass Constitution Check gates before design execution.
- Task lists MUST include verification tasks for tests and benchmarks when relevant.
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

**Version**: 1.0.0 | **Ratified**: 2026-02-19 | **Last Amended**: 2026-02-19
