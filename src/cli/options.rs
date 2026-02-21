use clap::{Parser, ValueEnum};

#[derive(Clone, Debug, ValueEnum)]
pub enum CliMode {
    Deterministic,
    Throughput,
}

#[derive(Parser, Debug)]
pub struct CliOptions {
    #[arg(long)]
    pub graph_source: String,
    #[arg(long, value_enum, default_value = "deterministic")]
    pub mode: CliMode,
    #[arg(long, default_value = "in-memory")]
    pub backend: String,
}
