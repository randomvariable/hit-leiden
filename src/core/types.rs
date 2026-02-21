#[derive(Clone, Debug, PartialEq)]
pub enum GraphFormat {
    EdgeList,
    CsrBinary,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GraphSourceType {
    File,
    Neo4jSnapshot,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphDataset {
    pub dataset_id: String,
    pub source_uri: String,
    pub is_weighted: bool,
    pub node_count: usize,
    pub edge_count: usize,
    pub checksum: String,
    pub format: GraphFormat,
    pub mmap_compatible: bool,
    pub mmap_path: Option<String>,
    pub source_type: GraphSourceType,
    pub source_snapshot_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RunMode {
    Deterministic,
    Throughput,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GraphBackend {
    InMemory,
    Mmap,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunConfiguration {
    pub config_id: String,
    pub mode: RunMode,
    pub acceleration_enabled: bool,
    pub seed: Option<u64>,
    pub max_iterations: usize,
    pub quality_tolerance: f64,
    pub pinned_profile_id: Option<String>,
    pub graph_backend: GraphBackend,
    pub graph_source: GraphSourceType,
}

impl Default for RunConfiguration {
    fn default() -> Self {
        Self {
            config_id: "default".to_string(),
            mode: RunMode::Deterministic,
            acceleration_enabled: false,
            seed: None,
            max_iterations: 10,
            quality_tolerance: 0.001,
            pinned_profile_id: None,
            graph_backend: GraphBackend::InMemory,
            graph_source: GraphSourceType::File,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RunStatus {
    Running,
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BackendType {
    PureRust,
    NativeAccel,
    CudaAccel,
    RocmAccel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunExecution {
    pub run_id: String,
    pub dataset_id: String,
    pub config_id: String,
    pub started_at: u64, // timestamp
    pub completed_at: Option<u64>,
    pub status: RunStatus,
    pub backend: BackendType,
    pub graph_backend_resolved: GraphBackend,
    pub graph_source_resolved: GraphSourceType,
    pub fallback_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PartitionResult {
    pub run_id: String,
    pub node_to_community: Vec<usize>,
    pub community_count: usize,
    pub quality_score: f64,
    pub iteration_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidationReport {
    pub run_id: String,
    pub hard_invariants_passed: bool,
    pub deterministic_identity_passed: Option<bool>,
    pub quality_delta_vs_reference: Option<f64>,
    pub equivalence_passed: bool,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunOutcome {
    pub execution: RunExecution,
    pub partition: Option<PartitionResult>,
    pub validation: Option<ValidationReport>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphInput {
    pub dataset_id: String,
    pub node_count: usize,
    pub edges: Vec<(usize, usize, Option<f64>)>,
}

impl GraphInput {
    pub fn empty(dataset_id: impl Into<String>) -> Self {
        Self {
            dataset_id: dataset_id.into(),
            node_count: 0,
            edges: Vec::new(),
        }
    }
}

/// Results from a single batch update
#[derive(Clone, Debug)]
pub struct BatchResult {
    pub batch_idx: usize,
    pub edges_added: usize,
    pub total_edges: usize,
    pub nodes_in_graph: usize,
    pub hit_leiden_time_ms: f64,
    pub st_leiden_time_ms: f64,
    pub speedup: f64,
    pub hit_leiden_iterations: usize,
    pub modularity: f64,
}

/// Aggregate results across all incremental batches
#[derive(Clone, Debug)]
pub struct IncrementalOutcome {
    pub batches: Vec<BatchResult>,
    pub total_time_seconds: f64,
    pub avg_speedup: f64,
    pub cumulative_speedup: f64,
}
