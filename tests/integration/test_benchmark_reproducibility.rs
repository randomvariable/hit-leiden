use hit_leiden::benchmark::hardware_profile::HardwareProfile;
use hit_leiden::compare_baseline;

#[test]
fn benchmark_reproducible() {
    let profile = HardwareProfile {
        id: "pinned".to_string(),
        pinned: true,
    };
    let out = compare_baseline("a", "b", "suite", &profile);
    assert!(out.reproducible);
}
