#[derive(Clone, Debug, PartialEq)]
pub struct BenchmarkManifest {
    pub dataset_id: String,
    pub baseline_commit: String,
    pub candidate_commit: String,
}
