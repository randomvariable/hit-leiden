use crate::core::backend::GraphBackend;

pub fn default_backend() -> GraphBackend {
    GraphBackend::InMemory
}
