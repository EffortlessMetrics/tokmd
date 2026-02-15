use std::path::PathBuf;
use std::time::Instant;
use tokei::{Language, LanguageType, Languages, Report};
use tokmd_model::{collect_file_rows, create_module_report};
use tokmd_types::ChildIncludeMode;

fn main() {
    let mut languages = Languages::new();
    let count = 10_000;

    // Create a large fake Languages structure
    let mut rust_lang = Language::new();
    for i in 0..count {
        let path = PathBuf::from(format!("crates/foo/src/mod_{}/file_{}.rs", i / 100, i));
        let mut report = Report::new(path);
        // Try setting fields directly assuming they are public
        report.stats.code = 100;
        report.stats.comments = 10;
        report.stats.blanks = 5;

        rust_lang.reports.push(report);
    }
    languages.insert(LanguageType::Rust, rust_lang);

    let module_roots = vec!["crates".to_string()];
    let module_depth = 2;
    let children = ChildIncludeMode::ParentsOnly;

    let iterations = 50;

    println!(
        "Benchmarking with {} files over {} iterations...",
        count, iterations
    );

    let start = Instant::now();

    for _ in 0..iterations {
        let _rows = collect_file_rows(&languages, &module_roots, module_depth, children, None);
    }

    let duration = start.elapsed();
    println!(
        "collect_file_rows: Total {:?} | Avg {:?}",
        duration,
        duration / iterations as u32
    );

    let start = Instant::now();
    for _ in 0..iterations {
        let _report = create_module_report(&languages, &module_roots, module_depth, children, 100);
    }
    let duration = start.elapsed();
    println!(
        "create_module_report: Total {:?} | Avg {:?}",
        duration,
        duration / iterations as u32
    );
}
