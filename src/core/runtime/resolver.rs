use crate::core::backend::{AccelerationTarget, GraphBackend, GraphSource, ResolutionMetadata};
use crate::core::config::RunConfig;

pub fn resolve(config: &RunConfig) -> ResolutionMetadata {
    ResolutionMetadata {
        source_resolved: config.graph_source,
        backend_resolved: config.graph_backend,
        accel_resolved: config.acceleration,
        fallback_reason: None,
    }
}

pub fn release_gate_eligible(source: GraphSource) -> (bool, Option<String>) {
    if source == GraphSource::LiveNeo4j {
        (
            false,
            Some("LIVE_QUERY_SOURCE_INELIGIBLE_FOR_RELEASE_GATE".to_string()),
        )
    } else {
        (true, None)
    }
}

pub fn fallback(
    source: GraphSource,
    backend: GraphBackend,
    accel: AccelerationTarget,
) -> ResolutionMetadata {
    ResolutionMetadata {
        source_resolved: source,
        backend_resolved: backend,
        accel_resolved: accel,
        fallback_reason: None,
    }
}
