# Quickstart: HIT-Leiden Planning Validation

## 1. Run default deterministic clustering (correctness-first)
1. Build the project in release mode.
2. Execute a deterministic run on a curated dataset.
3. Verify output artifacts include:
   - partition result,
   - validation report,
   - run configuration summary.

## 2. Validate deterministic identity
1. Re-run the same dataset and configuration.
2. Confirm exact partition identity match.
3. Confirm hard invariants are reported as passed.

## 3. Run explicit high-throughput mode
1. Enable high-throughput mode explicitly.
2. Execute on the same dataset.
3. Confirm equivalence policy:
   - hard invariants passed,
   - quality delta <= 0.1%.

## 4. Benchmark against frozen baseline
1. Use the frozen in-repo baseline commit and current candidate commit.
2. Run benchmark suite on a pinned hardware profile.
3. Produce a benchmark report artifact containing dataset IDs, config IDs, hardware/runtime details, and median throughput gain.
4. Mark release-gate status only for pinned-profile results.

## 5. Optional native acceleration verification
1. Enable acceleration backend.
2. Run compatibility check and execute workload.
3. If acceleration is unavailable or fails validation, verify automatic fallback to pure Rust and recorded fallback reason.

## 6. Optional CUDA/ROCm target verification
1. Execute workload with CUDA target explicitly enabled on CUDA-capable hardware.
2. Execute workload with ROCm target explicitly enabled on ROCm-capable hardware.
3. Verify each target satisfies mode-specific correctness policy versus CPU reference runs.
4. Verify compatibility failures produce actionable diagnostics and safe fallback behavior.

## 7. Optional mmap backend verification
1. Provide a mmap-compatible dataset artifact.
2. Execute with graph backend explicitly set to mmap.
3. Validate mode-specific correctness parity versus in-memory backend.
4. If mmap initialization/access fails, verify actionable diagnostics and safe fallback behavior.

## 8. Optional Neo4j/Cypher snapshot source verification
1. Configure Neo4j/Cypher source connection and snapshot parameters.
2. Execute batched snapshot projection into crate-native graph structures.
3. Run deterministic and throughput modes on projected graph.
4. Verify correctness parity with equivalent file-sourced dataset runs.
5. Confirm projection report metadata is stored for reproducibility.

## 9. Documentation outcomes
1. Produce the complete mathematical description document (symbols, objective, updates, termination).
2. Produce the complete developer-oriented algorithm explanation document.
3. Confirm both are referenced from main project documentation.
