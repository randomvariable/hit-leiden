use crate::cli::options::{CliMode, CliOptions};
use crate::core::backend::{AccelerationTarget, GraphBackend, GraphSource};
use crate::core::config::{RunConfig, RunMode};
use crate::core::types::GraphInput;

pub fn run_from_cli(
    options: &CliOptions,
    graph: &GraphInput,
) -> Result<crate::core::types::RunOutcome, crate::core::error::HitLeidenError> {
    let mode = match options.mode {
        CliMode::Deterministic => RunMode::Deterministic,
        CliMode::Throughput => RunMode::Throughput,
    };

    let graph_backend = match options.backend.as_str() {
        "mmap" => GraphBackend::Mmap,
        _ => GraphBackend::InMemory,
    };

    let config = RunConfig {
        mode,
        graph_source: GraphSource::File, // Assuming file for now
        graph_backend,
        acceleration: AccelerationTarget::PureRust,
        quality_tolerance: 0.001,
        max_iterations: 10,
        pinned_profile: None,
    };

    crate::run(graph, &config)
}

pub fn run_default(
    graph: &GraphInput,
) -> Result<crate::core::types::RunOutcome, crate::core::error::HitLeidenError> {
    crate::run(graph, &RunConfig::default())
}
