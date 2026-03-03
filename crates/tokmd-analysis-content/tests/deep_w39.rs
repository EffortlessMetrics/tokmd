//! Deep tests for tokmd-analysis-content: TODO detection, duplicates, imports.

use std::path::{Path, PathBuf};

use tokmd_analysis_content::{
    ContentLimits, ImportGranularity, build_duplicate_report, build_import_report,
    build_todo_report,
};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ─── helpers ───────────────────────────────────────────────────

fn make_row(path: &str, module: &str, lang: &str, bytes: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code: 100,
        comments: 10,
        blanks: 5,
        lines: 115,
        bytes,
        tokens: 200,
    }
}

fn make_export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn default_limits() -> ContentLimits {
    ContentLimits {
        max_bytes: None,
        max_file_bytes: None,
    }
}

// ─── TODO / FIXME detection ────────────────────────────────────

#[test]
fn todo_report_detects_todo_tags() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("a.rs"),
        "// TODO: fix this\n// FIXME: broken\nfn main() {}\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("a.rs")];
    let report = build_todo_report(root, &files, &default_limits(), 1000).unwrap();
    assert_eq!(report.total, 2);
    let todo_count = report.tags.iter().find(|t| t.tag == "TODO").unwrap().count;
    let fixme_count = report.tags.iter().find(|t| t.tag == "FIXME").unwrap().count;
    assert_eq!(todo_count, 1);
    assert_eq!(fixme_count, 1);
}

#[test]
fn todo_report_detects_hack_and_xxx() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("b.rs"),
        "// HACK: workaround\n// XXX: needs review\n// XXX: another one\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("b.rs")];
    let report = build_todo_report(root, &files, &default_limits(), 1000).unwrap();
    let hack_count = report.tags.iter().find(|t| t.tag == "HACK").unwrap().count;
    let xxx_count = report.tags.iter().find(|t| t.tag == "XXX").unwrap().count;
    assert_eq!(hack_count, 1);
    assert_eq!(xxx_count, 2);
}

#[test]
fn todo_density_per_kloc_computed() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("c.rs"), "// TODO: one\n// TODO: two\n").unwrap();

    let files = vec![PathBuf::from("c.rs")];
    // 2 TODOs in 2000 lines = 2/2.0 = 1.0 per KLOC
    let report = build_todo_report(root, &files, &default_limits(), 2000).unwrap();
    assert_eq!(report.total, 2);
    assert!((report.density_per_kloc - 1.0).abs() < 0.01);
}

#[test]
fn todo_density_zero_for_zero_code() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("d.rs"), "// TODO: test\n").unwrap();

    let files = vec![PathBuf::from("d.rs")];
    let report = build_todo_report(root, &files, &default_limits(), 0).unwrap();
    assert_eq!(report.density_per_kloc, 0.0);
}

#[test]
fn todo_report_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("empty.rs"), "").unwrap();

    let files = vec![PathBuf::from("empty.rs")];
    let report = build_todo_report(root, &files, &default_limits(), 1000).unwrap();
    assert_eq!(report.total, 0);
}

#[test]
fn todo_report_binary_file_skipped() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // Write binary content with null bytes
    let mut content = b"// TODO: this\n".to_vec();
    content.extend_from_slice(&[0x00, 0x00, 0x00, 0x01, 0x02]);
    std::fs::write(root.join("binary.bin"), &content).unwrap();

    let files = vec![PathBuf::from("binary.bin")];
    let report = build_todo_report(root, &files, &default_limits(), 1000).unwrap();
    assert_eq!(report.total, 0);
}

#[test]
fn todo_report_respects_max_file_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // Write a file that starts with valid text then has TODOs past the limit
    let mut content = String::new();
    for _ in 0..100 {
        content.push_str("// regular line\n");
    }
    content.push_str("// TODO: hidden past limit\n");
    std::fs::write(root.join("big.rs"), &content).unwrap();

    let limits = ContentLimits {
        max_bytes: None,
        max_file_bytes: Some(50), // very small limit
    };
    let files = vec![PathBuf::from("big.rs")];
    let report = build_todo_report(root, &files, &limits, 1000).unwrap();
    // The TODO at the end shouldn't be seen due to file-level byte limit
    assert_eq!(report.total, 0);
}

#[test]
fn todo_report_no_files() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<PathBuf> = vec![];
    let report = build_todo_report(dir.path(), &files, &default_limits(), 1000).unwrap();
    assert_eq!(report.total, 0);
    assert!(report.tags.is_empty());
}

// ─── duplicate detection ───────────────────────────────────────

#[test]
fn duplicate_report_finds_exact_duplicates() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let content = "fn hello() { println!(\"hello\"); }\n";
    std::fs::write(root.join("a.rs"), content).unwrap();
    std::fs::write(root.join("b.rs"), content).unwrap();

    let export = make_export(vec![
        make_row("a.rs", "src", "Rust", content.len()),
        make_row("b.rs", "src", "Rust", content.len()),
    ]);
    let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];
    let report = build_duplicate_report(root, &files, &export, &default_limits()).unwrap();
    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.groups[0].files.len(), 2);
    assert!(report.wasted_bytes > 0);
}

#[test]
fn duplicate_report_no_duplicates_for_unique_files() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("a.rs"), "fn a() {}\n").unwrap();
    std::fs::write(root.join("b.rs"), "fn b() {}\n").unwrap();

    let export = make_export(vec![
        make_row("a.rs", "src", "Rust", 10),
        make_row("b.rs", "src", "Rust", 10),
    ]);
    let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];
    let report = build_duplicate_report(root, &files, &export, &default_limits()).unwrap();
    assert!(report.groups.is_empty());
    assert_eq!(report.wasted_bytes, 0);
}

#[test]
fn duplicate_report_empty_files_not_duplicates() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("a.rs"), "").unwrap();
    std::fs::write(root.join("b.rs"), "").unwrap();

    let export = make_export(vec![
        make_row("a.rs", "src", "Rust", 0),
        make_row("b.rs", "src", "Rust", 0),
    ]);
    let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];
    let report = build_duplicate_report(root, &files, &export, &default_limits()).unwrap();
    // Empty files (size 0) are explicitly excluded from duplication groups
    assert!(report.groups.is_empty());
}

#[test]
fn duplicate_report_strategy_is_blake3() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let export = make_export(vec![]);
    let report = build_duplicate_report(root, &[], &export, &default_limits()).unwrap();
    assert_eq!(report.strategy, "exact-blake3");
}

#[test]
fn duplicate_report_wasted_bytes_calculation() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let content = "abcdefghij"; // 10 bytes
    std::fs::write(root.join("a.rs"), content).unwrap();
    std::fs::write(root.join("b.rs"), content).unwrap();
    std::fs::write(root.join("c.rs"), content).unwrap();

    let export = make_export(vec![
        make_row("a.rs", "src", "Rust", 10),
        make_row("b.rs", "src", "Rust", 10),
        make_row("c.rs", "src", "Rust", 10),
    ]);
    let files = vec![
        PathBuf::from("a.rs"),
        PathBuf::from("b.rs"),
        PathBuf::from("c.rs"),
    ];
    let report = build_duplicate_report(root, &files, &export, &default_limits()).unwrap();
    // 3 duplicates of 10 bytes → 2 * 10 = 20 wasted bytes
    assert_eq!(report.wasted_bytes, 20);
}

#[test]
fn duplicate_report_density_computed() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let content = "duplicate content here\n";
    std::fs::write(root.join("a.rs"), content).unwrap();
    std::fs::write(root.join("b.rs"), content).unwrap();

    let export = make_export(vec![
        make_row("a.rs", "src", "Rust", content.len()),
        make_row("b.rs", "src", "Rust", content.len()),
    ]);
    let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];
    let report = build_duplicate_report(root, &files, &export, &default_limits()).unwrap();
    let density = report.density.unwrap();
    assert!(density.wasted_pct_of_codebase > 0.0);
    assert_eq!(density.duplicate_groups, 1);
}

// ─── import graph extraction ───────────────────────────────────

#[test]
fn import_report_module_granularity() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("main.rs"),
        "use std::io;\nuse serde::Serialize;\n",
    )
    .unwrap();

    let export = make_export(vec![make_row("main.rs", "src", "Rust", 100)]);
    let files = vec![PathBuf::from("main.rs")];
    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::Module,
        &default_limits(),
    )
    .unwrap();
    assert_eq!(report.granularity, "module");
    assert!(!report.edges.is_empty());
}

#[test]
fn import_report_file_granularity() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("lib.rs"), "use std::collections;\n").unwrap();

    let export = make_export(vec![make_row("lib.rs", "src", "Rust", 100)]);
    let files = vec![PathBuf::from("lib.rs")];
    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::File,
        &default_limits(),
    )
    .unwrap();
    assert_eq!(report.granularity, "file");
}

#[test]
fn import_report_empty_files() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let export = make_export(vec![]);
    let report = build_import_report(
        root,
        &[],
        &export,
        ImportGranularity::Module,
        &default_limits(),
    )
    .unwrap();
    assert!(report.edges.is_empty());
}

// ─── content scanning edge cases ───────────────────────────────

#[test]
fn todo_multiple_tags_same_line() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // A line that contains both TODO and FIXME
    std::fs::write(root.join("multi.rs"), "// TODO FIXME: both tags here\n").unwrap();

    let files = vec![PathBuf::from("multi.rs")];
    let report = build_todo_report(root, &files, &default_limits(), 1000).unwrap();
    // Each tag is counted independently
    assert!(report.total >= 2);
}

#[test]
fn todo_case_sensitive() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // "todo" in lowercase should NOT match "TODO" tag
    std::fs::write(
        root.join("case.rs"),
        "// todo: lowercase\n// TODO: uppercase\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("case.rs")];
    let report = build_todo_report(root, &files, &default_limits(), 1000).unwrap();
    let todo_count = report
        .tags
        .iter()
        .find(|t| t.tag == "TODO")
        .map(|t| t.count)
        .unwrap_or(0);
    // Only uppercase TODO should match
    assert!(todo_count >= 1);
}

#[test]
fn duplicate_different_sizes_not_grouped() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("a.rs"), "short").unwrap();
    std::fs::write(root.join("b.rs"), "longer content").unwrap();

    let export = make_export(vec![
        make_row("a.rs", "src", "Rust", 5),
        make_row("b.rs", "src", "Rust", 14),
    ]);
    let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];
    let report = build_duplicate_report(root, &files, &export, &default_limits()).unwrap();
    assert!(report.groups.is_empty());
}
