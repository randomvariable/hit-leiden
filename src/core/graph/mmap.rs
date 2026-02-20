use crate::core::graph::in_memory::InMemoryGraph;
use crate::core::types::GraphInput;

#[derive(Clone, Debug)]
pub struct MmapGraph {
    pub inner: InMemoryGraph,
}

impl From<&GraphInput> for MmapGraph {
    fn from(value: &GraphInput) -> Self {
        Self {
            inner: InMemoryGraph::from(value),
        }
    }
}
