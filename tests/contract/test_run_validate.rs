use hit_leiden::{run, validate, GraphInput, RunConfig};

#[test]
fn run_and_validate_contract() {
    let graph = GraphInput {
        dataset_id: "d1".to_string(),
        node_count: 3,
        edges: vec![(0, 1, None), (1, 2, None)],
    };
    let config = RunConfig::default();
    let outcome = run(&graph, &config).expect("run should succeed");
    let validation = validate(&outcome, &outcome, config.mode);
    assert!(validation.equivalence_passed);
}
