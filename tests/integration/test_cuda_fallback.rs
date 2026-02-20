use hit_leiden::{core::backend::AccelerationTarget, core::runtime::orchestrator, RunConfig};

#[test]
fn cuda_fallback_when_unavailable() {
    let mut cfg = RunConfig::default();
    cfg.acceleration = AccelerationTarget::Cuda;
    let r = orchestrator::resolve_with_fallback(&cfg, false);
    assert_eq!(r.accel_resolved, AccelerationTarget::PureRust);
    assert!(r.fallback_reason.is_some());
}
