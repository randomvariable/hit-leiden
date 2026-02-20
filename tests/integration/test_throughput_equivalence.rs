use hit_leiden::{core::config::RunMode, run, validate, GraphInput, RunConfig};

#[test]
fn throughput_equivalence_bounds() {
    let graph = GraphInput {
        dataset_id: "d3".to_string(),
        node_count: 2,
        edges: vec![(0, 1, Some(1.0))],
    };
    let mut config = RunConfig::default();
    config.mode = RunMode::Throughput;
    let a = run(&graph, &config).expect("a");
    let b = run(&graph, &config).expect("b");
    let v = validate(&a, &b, RunMode::Throughput);
    assert!(v.hard_invariants_passed);
    assert!(v.equivalence_passed);
}
