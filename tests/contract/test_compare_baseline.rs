use hit_leiden::benchmark::hardware_profile::HardwareProfile;
use hit_leiden::compare_baseline;

#[test]
fn compare_baseline_contract() {
    let profile = HardwareProfile {
        id: "pinned".to_string(),
        pinned: true,
    };
    let out = compare_baseline("base", "cand", "suite", &profile);
    assert!(out.median_throughput_gain >= 1.0);
}
