use hit_leiden::{core::backend::AccelerationTarget, run, GraphInput, RunConfig};

#[test]
fn rocm_successful_run_parity() {
    let graph = GraphInput {
        dataset_id: "r1".into(),
        node_count: 2,
        edges: vec![(0, 1, None)],
    };
    let cpu = run(&graph, &RunConfig::default()).expect("cpu");
    let mut cfg = RunConfig::default();
    cfg.acceleration = AccelerationTarget::Rocm;
    let gpu = run(&graph, &cfg).expect("gpu");
    assert_eq!(cpu.partition, gpu.partition);
}
