use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tokmd_model::module_key;

pub fn criterion_benchmark(c: &mut Criterion) {
    let roots = vec!["crates".to_string(), "packages".to_string()];

    c.bench_function("module_key root", |b| {
        b.iter(|| {
            module_key(black_box("Cargo.toml"), black_box(&roots), black_box(2))
        })
    });

    c.bench_function("module_key shallow", |b| {
        b.iter(|| {
            module_key(black_box("src/lib.rs"), black_box(&roots), black_box(2))
        })
    });

    c.bench_function("module_key deep", |b| {
        b.iter(|| {
            module_key(
                black_box("crates/tokmd/src/main.rs"),
                black_box(&roots),
                black_box(2)
            )
        })
    });

    c.bench_function("module_key deep win", |b| {
        b.iter(|| {
            module_key(
                black_box("crates\\tokmd\\src\\main.rs"),
                black_box(&roots),
                black_box(2)
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
