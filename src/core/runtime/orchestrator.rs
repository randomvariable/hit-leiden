use crate::core::config::RunConfig;
use crate::core::runtime::resolver;

pub fn resolve_with_fallback(
    config: &RunConfig,
    available: bool,
) -> crate::core::backend::ResolutionMetadata {
    let mut r = resolver::resolve(config);
    if !available {
        r.fallback_reason = Some("ACCEL_UNAVAILABLE".to_string());
    }
    r
}
