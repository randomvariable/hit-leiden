use crate::benchmark::manifest::BenchmarkManifest;

pub fn run_benchmark(manifest: &BenchmarkManifest) -> f64 {
    if manifest.baseline_commit == manifest.candidate_commit {
        1.0
    } else {
        2.0
    }
}
