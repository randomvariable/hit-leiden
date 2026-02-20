# Research: HIT-Leiden Reference-Accurate High-Performance Engine

## 1) Graph and Partition Representation
- Decision: Use immutable contiguous graph storage (CSR-oriented adjacency arrays) with mutable frontier-focused buffers for updates; partition state uses structure-of-arrays (`node_to_comm`, `comm_volume`, `comm_internal_weight`, hierarchy links).
- Rationale: Maximizes cache locality, minimizes allocation churn, and enables high-throughput scans and deterministic replay.
- Alternatives considered: Adjacency-list object graphs (simpler but poor locality), hash-map-driven state (flexible but slower), full copy-on-write snapshots (clean but memory-heavy).

## 2) Deterministic vs High-Throughput Modes
- Decision: Deterministic mode is default with fixed traversal/tie-break/reduction rules; high-throughput mode is explicit opt-in with relaxed scheduling.
- Rationale: Supports non-expert correctness-first usage while allowing performance-oriented users to trade strict identity for bounded equivalence.
- Alternatives considered: Deterministic-only mode (safe but leaves performance on table), throughput-only mode (fast but weak reproducibility), auto-switching modes (hard to reason about and validate).

## 3) Parallel Execution Strategy
- Decision: Parallelize by frontier shards and phase boundaries with thread-local scratch state; merge deterministically in deterministic mode and contention-minimized merge in throughput mode.
- Rationale: Matches algorithm locality and reduces global lock contention.
- Alternatives considered: Global coarse-grained locks (contention-heavy), async runtime model (not ideal for CPU-bound kernels), actor-per-community model (coordination overhead).

## 4) Optional Native Acceleration
- Decision: Introduce feature-gated acceleration backend behind a stable backend trait with runtime capability checks and mandatory fallback to pure Rust on mismatch or error.
- Rationale: Preserves portability and safety while enabling targeted speedups where justified.
- Alternatives considered: Mandatory native dependency (fragile deployment), ad-hoc FFI in core path (unsafe sprawl), full external native rewrite (maintenance complexity).

## 5) Correctness and Equivalence Validation
- Decision: Layered validation with hard invariants for all modes; deterministic mode requires exact partition identity; throughput mode requires hard invariants + quality delta <=0.1%.
- Rationale: Ensures strict correctness where deterministic guarantees are claimed and practical correctness bounds where non-determinism is allowed.
- Alternatives considered: Exact equality in all modes (too strict for throughput), quality-only checks (insufficient), spot/manual checks (non-scalable).

## 6) Reproducible Benchmark Policy
- Decision: Freeze first correctness-validated implementation as baseline commit; release-gate benchmarks only on pinned hardware profiles; require complete run manifests.
- Rationale: Makes benchmark claims comparable and auditable across optimization cycles.
- Alternatives considered: Moving baseline (progress drift), mixed hardware acceptance (non-comparable), ad-hoc local benchmarking (non-reproducible).

## 7) Documentation Strategy
- Decision: Provide two first-class docs: (a) complete mathematical specification and (b) complete developer-oriented algorithm walkthrough sharing one symbol glossary.
- Rationale: Separates formal correctness communication from onboarding and implementation comprehension.
- Alternatives considered: Single blended doc (too dense for one audience, too shallow for the other), code comments only (insufficient), paper-link-only guidance (not operational).
