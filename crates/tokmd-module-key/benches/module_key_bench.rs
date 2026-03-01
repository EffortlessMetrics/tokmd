use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tokmd_module_key::module_key;

fn bench_module_key(c: &mut Criterion) {
    let roots = vec!["crates".to_string(), "packages".to_string()];
    let paths = vec![
        "Cargo.toml",
        "./Cargo.toml",
        "crates/foo/src/lib.rs",
        "./crates/foo/src/lib.rs",
        "crates/foo/src/main.rs",
        "packages/bar/src/lib.rs",
        "src/lib.rs",
        "tools/gen.rs",
        "a/b/c/d/e/f/g.rs",
    ];

    c.bench_function("module_key", |b| {
        b.iter(|| {
            for path in &paths {
                let _ = black_box(module_key(black_box(path), black_box(&roots), black_box(2)));
            }
        })
    });
}

criterion_group!(benches, bench_module_key);
criterion_main!(benches);
