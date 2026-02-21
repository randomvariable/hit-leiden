use hit_leiden::benchmark::hardware_profile::HardwareProfile;
use hit_leiden::benchmark::release_gate::eligible;
use hit_leiden::core::backend::GraphSource;

#[test]
fn live_query_is_ineligible_for_release_gate() {
    let profile = HardwareProfile {
        id: "pinned".into(),
        pinned: true,
    };
    let (ok, reason) = eligible(&profile, GraphSource::LiveNeo4j);
    assert!(!ok);
    assert_eq!(
        reason.as_deref(),
        Some("LIVE_QUERY_SOURCE_INELIGIBLE_FOR_RELEASE_GATE")
    );
}
