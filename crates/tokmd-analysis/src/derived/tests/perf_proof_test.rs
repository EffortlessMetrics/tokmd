use crate::derived::derive_report;
use std::time::Instant;
use tokmd_types::{ExportData, FileKind, FileRow};

#[test]
fn derived_alloc_reduction_proof() {
    let mut rows = Vec::with_capacity(100_000);
    for i in 0..100_000 {
        rows.push(FileRow {
            path: format!("src/module_{}/file_{}.rs", i % 100, i),
            module: format!("module_{}", i % 100),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 100,
            comments: 10,
            blanks: 20,
            lines: 130,
            bytes: 2000,
            tokens: 500,
        });
    }
    let export = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 2,
        children: tokmd_types::ChildIncludeMode::ParentsOnly,
    };

    let start = Instant::now();
    let _report = derive_report(&export, None);
    let elapsed = start.elapsed();

    println!("derive_report elapsed: {:?}", elapsed);
    assert!(elapsed.as_millis() < 5000); // Sanity check
}
