use hit_leiden::{
    core::graph::{neo4j_mapping::ProjectionConfig, neo4j_snapshot::Neo4jSourceConfig},
    project_from_neo4j,
};

#[test]
fn neo4j_projection_parity_shape() {
    let source = Neo4jSourceConfig {
        uri: "bolt://localhost".to_string(),
    };
    let proj = ProjectionConfig {
        snapshot_id: "s1".to_string(),
        batched: true,
    };
    let graph = project_from_neo4j(&source, &proj).expect("projection");
    assert!(graph.dataset_id.starts_with("neo4j:"));
}
