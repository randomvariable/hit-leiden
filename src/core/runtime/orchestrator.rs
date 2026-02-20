use crate::core::backend::AccelerationTarget;
use crate::core::config::RunConfig;
use crate::core::runtime::resolver;

pub fn resolve_with_fallback(
    config: &RunConfig,
    available: bool,
) -> crate::core::backend::ResolutionMetadata {
    let mut r = resolver::resolve(config);
    if matches!(
        r.accel_resolved,
        AccelerationTarget::Cuda | AccelerationTarget::Rocm
    ) && !available
    {
        r.accel_resolved = AccelerationTarget::PureRust;
        r.fallback_reason = Some("ACCEL_UNAVAILABLE".to_string());
    }
    r
}
