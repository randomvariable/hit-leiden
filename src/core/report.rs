use crate::core::backend::ResolutionMetadata;

#[derive(Clone, Debug, PartialEq)]
pub struct RunOutcome {
    pub run_id: String,
    pub partition: Vec<usize>,
    pub quality_score: f64,
    pub resolution: ResolutionMetadata,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidationOutcome {
    pub hard_invariants_passed: bool,
    pub deterministic_identity_passed: Option<bool>,
    pub quality_delta_vs_reference: Option<f64>,
    pub equivalence_passed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BenchmarkOutcome {
    pub baseline_commit: String,
    pub candidate_commit: String,
    pub benchmark_suite: String,
    pub median_throughput_gain: f64,
    pub reproducible: bool,
    pub release_gate_eligible: bool,
    pub release_gate_reason: Option<String>,
}

pub mod writer;
