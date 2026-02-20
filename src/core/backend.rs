#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphSource {
    File,
    Neo4jSnapshot,
    LiveNeo4j,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphBackend {
    InMemory,
    Mmap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccelerationTarget {
    PureRust,
    Native,
    Cuda,
    Rocm,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolutionMetadata {
    pub source_resolved: GraphSource,
    pub backend_resolved: GraphBackend,
    pub accel_resolved: AccelerationTarget,
    pub fallback_reason: Option<String>,
}
