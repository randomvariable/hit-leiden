use hit_leiden::{core::backend::AccelerationTarget, core::runtime::orchestrator, RunConfig};

#[test]
fn rocm_fallback_when_unavailable() {
    let mut cfg = RunConfig::default();
    cfg.acceleration = AccelerationTarget::Rocm;
    let r = orchestrator::resolve_with_fallback(&cfg, false);
    assert_eq!(r.accel_resolved, AccelerationTarget::PureRust);
    assert!(r.fallback_reason.is_some());
}
