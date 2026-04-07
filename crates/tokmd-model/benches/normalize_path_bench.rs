use criterion::{criterion_group, criterion_main, Criterion};
use std::path::Path;
use std::hint::black_box;
use tokmd_model::normalize_path;

fn criterion_benchmark(c: &mut Criterion) {
    let p1 = Path::new("crates/tokmd-model/src/lib.rs");
    let p2 = Path::new("./crates/tokmd-model/src/lib.rs");
    let p3 = Path::new("crates\\tokmd-model\\src\\lib.rs");
    let pref = Path::new("crates/tokmd-model");
    let pref2 = Path::new("./crates");

    c.bench_function("normalize_path_clean", |b| b.iter(|| normalize_path(black_box(p1), black_box(None))));
    c.bench_function("normalize_path_dot_slash", |b| b.iter(|| normalize_path(black_box(p2), black_box(None))));
    c.bench_function("normalize_path_backslash", |b| b.iter(|| normalize_path(black_box(p3), black_box(None))));
    c.bench_function("normalize_path_with_prefix", |b| b.iter(|| normalize_path(black_box(p1), black_box(Some(pref)))));
    c.bench_function("normalize_path_with_dot_prefix", |b| b.iter(|| normalize_path(black_box(p1), black_box(Some(pref2)))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
