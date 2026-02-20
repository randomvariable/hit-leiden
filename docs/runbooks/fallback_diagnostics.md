# Fallback Diagnostics

- CUDA/ROCm unavailable -> fallback to `PureRust`, reason `ACCEL_UNAVAILABLE`.
- Unpinned profile -> release-gate ineligible.
- Live DB per-step query source -> release-gate ineligible.
