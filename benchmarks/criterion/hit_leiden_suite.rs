use criterion::{criterion_group, criterion_main, Criterion};
use hit_leiden::{run, GraphInput, RunConfig};

fn bench_run(c: &mut Criterion) {
    let graph = GraphInput {
        dataset_id: "bench".to_string(),
        node_count: 1_000,
        edges: vec![],
    };
    c.bench_function("hit_leiden_run", |b| {
        b.iter(|| run(&graph, &RunConfig::default()).expect("run"));
    });
}

criterion_group!(benches, bench_run);
criterion_main!(benches);
