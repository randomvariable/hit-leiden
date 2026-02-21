use crate::core::types::RunOutcome;

pub fn check(run: &RunOutcome) -> bool {
    if let Some(partition) = &run.partition {
        // Every node belongs to exactly one community (implicit in Vec<usize>)
        // Community IDs are valid (less than node_count)
        let node_count = partition.node_to_community.len();
        partition.node_to_community.iter().all(|&c| c < node_count)
    } else {
        false
    }
}
