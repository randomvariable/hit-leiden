use hit_leiden::core::runtime::resolver;
use hit_leiden::RunConfig;

#[test]
fn backend_resolution_contract() {
    let cfg = RunConfig::default();
    let r = resolver::resolve(&cfg);
    assert!(r.fallback_reason.is_none());
}
