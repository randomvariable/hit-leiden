use crate::core::error::HitLeidenError;
use crate::core::graph::neo4j_mapping::ProjectionConfig;
use crate::core::types::GraphInput;

#[derive(Clone, Debug)]
pub struct Neo4jSourceConfig {
    pub uri: String,
}

pub fn project_from_neo4j(
    _source_config: &Neo4jSourceConfig,
    projection_config: &ProjectionConfig,
) -> Result<GraphInput, HitLeidenError> {
    Ok(GraphInput {
        dataset_id: format!("neo4j:{}", projection_config.snapshot_id),
        node_count: 0,
        edges: Vec::new(),
    })
}
