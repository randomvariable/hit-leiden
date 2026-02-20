use crate::core::config::RunMode;
use crate::core::report::ValidationOutcome;
use crate::core::types::RunOutcome;

pub fn validate(
    reference: &RunOutcome,
    candidate: &RunOutcome,
    mode: RunMode,
) -> ValidationOutcome {
    let ref_part = reference.partition.as_ref().unwrap();
    let cand_part = candidate.partition.as_ref().unwrap();
    let same_partition = ref_part.node_to_community == cand_part.node_to_community;
    let quality_delta = (ref_part.quality_score - cand_part.quality_score).abs();
    match mode {
        RunMode::Deterministic => ValidationOutcome {
            hard_invariants_passed: true,
            deterministic_identity_passed: Some(same_partition),
            quality_delta_vs_reference: Some(quality_delta),
            equivalence_passed: same_partition,
        },
        RunMode::Throughput => ValidationOutcome {
            hard_invariants_passed: true,
            deterministic_identity_passed: None,
            quality_delta_vs_reference: Some(quality_delta),
            equivalence_passed: quality_delta <= 0.001,
        },
    }
}
