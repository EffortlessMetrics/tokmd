use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::path::PathBuf;
use tokei::{LanguageType, Languages, Report};
use tokmd_model::collect_file_rows;
use tokmd_types::ChildIncludeMode;

fn criterion_benchmark(c: &mut Criterion) {
    let mut langs = Languages::new();
    let mut rust_lang = tokei::Language::new();
    for i in 0..10000 {
        let name = PathBuf::from(format!("crates/foo/src/file_{i}.rs"));
        let report = Report::new(name);
        rust_lang.reports.push(report);
    }
    langs.insert(LanguageType::Rust, rust_lang);

    let mut html_lang = tokei::Language::new();
    for i in 0..5000 {
        let name = PathBuf::from(format!("crates/foo/src/file_{i}.rs"));
        let report = Report::new(name);
        html_lang.reports.push(report);
    }
    langs.insert(LanguageType::Html, html_lang);
    let module_roots = vec!["crates".to_string()];

    c.bench_function("collect_file_rows_10000", |b| b.iter(|| {
        collect_file_rows(
            black_box(&langs),
            black_box(&module_roots),
            black_box(2),
            black_box(ChildIncludeMode::ParentsOnly),
            black_box(None),
        )
    }));

    c.bench_function("collect_file_rows_10000_separate", |b| b.iter(|| {
        collect_file_rows(
            black_box(&langs),
            black_box(&module_roots),
            black_box(2),
            black_box(ChildIncludeMode::Separate),
            black_box(None),
        )
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
