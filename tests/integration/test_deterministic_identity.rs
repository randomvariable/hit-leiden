use hit_leiden::{run, GraphInput, RunConfig};

#[test]
fn deterministic_replay_identity() {
    let graph = GraphInput {
        dataset_id: "d2".to_string(),
        node_count: 4,
        edges: vec![(0, 1, None), (2, 3, None)],
    };
    let config = RunConfig::default();
    let a = run(&graph, &config).expect("a");
    let b = run(&graph, &config).expect("b");
    assert_eq!(a.partition, b.partition);
}
