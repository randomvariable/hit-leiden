use hit_leiden::{cli::run::run_default, GraphInput};

#[test]
fn default_run_with_minimal_required_graph_source() {
    let graph = GraphInput {
        dataset_id: "min".into(),
        node_count: 1,
        edges: vec![],
    };
    let out = run_default(&graph).expect("default run should succeed");
    assert_eq!(out.partition.unwrap().node_to_community.len(), 1);
}
