use crate::benchmark::hardware_profile::HardwareProfile;
use crate::core::backend::GraphSource;

pub fn eligible(profile: &HardwareProfile, source: GraphSource) -> (bool, Option<String>) {
    if !profile.pinned {
        return (false, Some("UNPINNED_HARDWARE_PROFILE".to_string()));
    }
    if source == GraphSource::LiveNeo4j {
        return (
            false,
            Some("LIVE_QUERY_SOURCE_INELIGIBLE_FOR_RELEASE_GATE".to_string()),
        );
    }
    (true, None)
}
