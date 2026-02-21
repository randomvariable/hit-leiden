use hit_leiden::{core::backend::GraphBackend, run, GraphInput, RunConfig};

#[test]
fn mmap_backend_parity() {
    let graph = GraphInput {
        dataset_id: "m1".to_string(),
        node_count: 3,
        edges: vec![(0, 1, None)],
    };
    let mem = run(&graph, &RunConfig::default()).expect("mem");
    let mut mmap_cfg = RunConfig::default();
    mmap_cfg.graph_backend = GraphBackend::Mmap;
    let mmap = run(&graph, &mmap_cfg).expect("mmap");
    assert_eq!(mem.partition, mmap.partition);
}
