# hit-leiden

Fast, incremental community detection for GraphRAG systems running on modest
hardware. When a user adds a document to a knowledge graph, communities and their
summaries update without reprocessing the entire graph.

## Background

GraphRAG extends retrieval-augmented generation by organising a knowledge graph
into communities and summarising each one. These community summaries let an LLM
answer broad, thematic questions that span many documents — queries that
traditional vector search handles poorly. The quality of those summaries depends
directly on the quality and timeliness of the underlying community structure.

Standard community detection algorithms like Leiden run from scratch every time
the graph changes. For a RAG system where users regularly submit new documents,
this means re-clustering the entire graph on every ingestion — a process that
becomes prohibitively slow on consumer hardware as the knowledge graph grows.

This project takes an incremental approach: when edges are added or removed, only
the affected parts of the community hierarchy are reprocessed. This makes
continuous community maintenance feasible on modest machines and enables
downstream summarisation to keep pace with document ingestion.

## Quick Start

```sh
# Build
cargo build --release

# Run on an edge list (one "src dst [weight]" per line)
cargo run --release -- run --source file --path graph.txt

# Run benchmarks
cargo bench
```

## Goals

The following goals are listed in priority order.

### 1. Incremental community updates

The non-negotiable core of the project. When the knowledge graph changes, only
affected communities are reprocessed. Every architectural and algorithmic decision
must preserve this property. A design that requires full re-clustering on update
is a failure, regardless of how fast it is.

### 2. External graph processing

The graph of record lives outside the process — typically in a graph database. The
system must operate against this external store, receiving update notifications
and maintaining community structure accordingly. While working copies of subgraphs
or delta windows may be held in process memory, the external database is
authoritative. The system should minimise how much of the graph it needs in memory
at any given time, though a hard zero-copy constraint is not assumed.

### 3. Hierarchical communities

Communities must be organised hierarchically. LLM context windows impose an upper
bound on how much of a community can be sent for summarisation in a single pass.
Larger communities must therefore be composed of smaller sub-communities, enabling
recursive summarisation: summarise the leaves first, then summarise the summaries,
up through the hierarchy. This directly supports multi-resolution understanding of
the knowledge graph.

### 4. Incremental community output

Emit stabilised communities as soon as they are ready, rather than waiting for the
entire graph to converge. In a pipeline, this means downstream LLM summarisation
can begin as soon as a community is established or updated, rather than blocking
on the full clustering pass. This is essential for keeping end-to-end ingestion
latency low.

### 5. Modest hardware viability

Community detection must run on consumer-grade machines — laptops, small servers,
commodity cloud instances. The project must not assume access to GPU clusters or
high-memory hardware for correct operation. Resource efficiency is a first-class
concern.

### 6. Scale-out support

The architecture must support distributing community detection across multiple
nodes, not just scaling up on a single machine. As knowledge graphs grow, it
should be possible to add more modest machines rather than requiring bigger ones.

### 7. Fast by default

High-throughput parallel execution is the default mode, optimised for the RAG use
case where speed of update matters more than bit-exact reproducibility across
runs. The project serves document ingestion pipelines, not scientific
reproducibility.

### 8. Algorithm agility

The project currently implements HIT-Leiden (Algorithm 6 from
[arXiv:2601.08554](https://arxiv.org/abs/2601.08554)), but it is not permanently
tied to any single algorithm. Any approach that delivers correct, incremental,
hierarchical community updates is a candidate. The architecture should make it
straightforward to evaluate and adopt alternatives.

### 9. Correct clustering

Community assignments must be trustworthy. Summaries built on incorrect community
structure propagate errors through the entire RAG system. Correctness is validated
through invariant checks and quality metrics.

### 10. Optional deterministic mode

A deterministic execution mode is available for debugging and testing. It is not
the default. When enabled, identical inputs and configuration produce identical
outputs.

### 11. Extensible backends

The project supports multiple graph storage backends:

- **In-memory (CSR)** — default, fast, suitable for graphs that fit in RAM
- **Memory-mapped** — for larger graphs on machines with limited RAM
- **Neo4j snapshot projection** — for integration with existing Neo4j databases

### 12. Optional GPU acceleration

CUDA and ROCm acceleration paths are planned for future implementation, with
correctness parity against CPU backends and safe fallback when acceleration is
unavailable.

### 13. Accessible to non-experts

Sensible defaults, clear validation output, and minimal required configuration.
A user should be able to run a successful end-to-end pass with just a graph
source argument.

## Current Approach

HIT-Leiden was chosen as the starting point because the paper directly addresses
incremental community detection for dynamic graphs using the Leiden method, which
aligns with the core goals above. The algorithm maintains a hierarchical community
structure and updates it incrementally through movement, refinement, and
aggregation phases. This may be supplemented or replaced by other approaches as
the project evolves.

## Current Status

A CPU baseline has been implemented and merged (PR #1). Both high-throughput and
deterministic execution modes are functional. Benchmark scaffolding (incremental
runner, standard Leiden baseline comparison, release gate eligibility) is in
place.

Pending work includes: incremental community output, scale-out architecture, real
modularity scoring (currently placeholder), memory-mapped and Neo4j backend
qualification, GPU acceleration paths, and pinned-hardware benchmark evidence.

## References

- [HIT-Leiden paper (arXiv:2601.08554)](https://arxiv.org/abs/2601.08554)
- [Feature specification](specs/001-hit-leiden-rust/spec.md)
- [Project constitution](.specify/memory/constitution.md)
- [Developer guide](docs/guide/hit_leiden_explained.md)
- [Mathematical specification](docs/math/hit_leiden_spec.md)

## Licence

See [LICENSE](LICENSE).
