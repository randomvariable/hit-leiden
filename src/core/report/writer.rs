use crate::core::report::ValidationOutcome;
use crate::core::types::RunOutcome;

pub fn write_run_artifact(run: &RunOutcome, validation: &ValidationOutcome) -> String {
    format!(
        "Run ID: {}\nQuality: {}\nEquivalence Passed: {}",
        run.execution.run_id,
        run.partition.as_ref().map_or(0.0, |p| p.quality_score),
        validation.equivalence_passed
    )
}
