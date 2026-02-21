use crate::core::backend::{AccelerationTarget, GraphBackend, GraphSource};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunMode {
    Deterministic,
    Throughput,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunConfig {
    pub mode: RunMode,
    pub graph_source: GraphSource,
    pub graph_backend: GraphBackend,
    pub acceleration: AccelerationTarget,
    pub quality_tolerance: f64,
    pub max_iterations: usize,
    pub pinned_profile: Option<String>,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            mode: RunMode::Deterministic,
            graph_source: GraphSource::File,
            graph_backend: GraphBackend::InMemory,
            acceleration: AccelerationTarget::PureRust,
            quality_tolerance: 0.001,
            max_iterations: 10,
            pinned_profile: None,
        }
    }
}

impl RunConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_iterations == 0 {
            return Err("max_iterations must be > 0".to_string());
        }
        if self.quality_tolerance < 0.0 {
            return Err("quality_tolerance must be >= 0".to_string());
        }
        Ok(())
    }
}
