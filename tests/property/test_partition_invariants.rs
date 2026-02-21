use hit_leiden::{run, GraphInput, RunConfig};
use proptest::prelude::*;

proptest! {
    #[test]
    fn partition_len_matches_nodes(node_count in 0usize..50) {
        let graph = GraphInput {
            dataset_id: "p1".to_string(),
            node_count,
            edges: vec![],
        };
        let config = RunConfig::default();
        let out = run(&graph, &config).expect("run");
        prop_assert_eq!(out.partition.unwrap().node_to_community.len(), node_count);
    }
}
