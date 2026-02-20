use crate::benchmark::hardware_profile::HardwareProfile;

pub fn run_compare(baseline: &str, candidate: &str) -> crate::core::report::BenchmarkOutcome {
    let profile = HardwareProfile {
        id: "pinned-linux-x86_64".to_string(),
        pinned: true,
    };
    crate::compare_baseline(baseline, candidate, "default-suite", &profile)
}
