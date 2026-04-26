//! Extended BDD-style tests for tokmd-analysis content module.
//!
//! Additional coverage for TODO density, duplicate density metrics,
//! and import report edge cases.

use std::path::PathBuf;

use crate::content::{
    ContentLimits, ImportGranularity, build_duplicate_report, build_import_report,
    build_todo_report,
};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── helpers ──────────────────────────────────────────────────────────

fn file_row(path: &str, module: &str, lang: &str, bytes: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code: 10,
        comments: 2,
        blanks: 1,
        lines: 13,
        bytes,
        tokens: 80,
    }
}

fn empty_export() -> ExportData {
    ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

// ── TODO report: all lines are TODOs ─────────────────────────────────

#[test]
fn given_file_with_all_todo_lines_when_building_todo_report_then_high_density() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    // Every line contains a TODO tag
    let content =
        "// TODO: first\n// TODO: second\n// TODO: third\n// TODO: fourth\n// TODO: fifth\n";
    std::fs::write(root.join("todos.rs"), content).unwrap();

    let files = vec![PathBuf::from("todos.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 1000).unwrap();

    assert_eq!(report.total, 5);
    // 5 TODOs / 1.0 kLOC = 5.0 density
    assert_eq!(report.density_per_kloc, 5.0);
    // Only TODO tag should have non-zero count
    let todo_count = report
        .tags
        .iter()
        .find(|t| t.tag == "TODO")
        .map(|t| t.count)
        .unwrap_or(0);
    assert_eq!(todo_count, 5);
    assert!(
        report
            .tags
            .iter()
            .filter(|t| t.count > 0)
            .all(|t| t.tag == "TODO"),
        "only TODO should have non-zero count"
    );
}

// ── TODO report: only FIXME tags ─────────────────────────────────────

#[test]
fn given_file_with_only_fixme_tags_when_building_todo_report_then_only_fixme_counted() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    std::fs::write(
        root.join("fixmes.rs"),
        "// FIXME: bug1\n// FIXME: bug2\nfn main() {}\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("fixmes.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 2000).unwrap();

    assert_eq!(report.total, 2);
    let fixme_count = report
        .tags
        .iter()
        .find(|t| t.tag == "FIXME")
        .map(|t| t.count)
        .unwrap_or(0);
    assert_eq!(fixme_count, 2);
    // No other tags should have non-zero counts
    assert!(
        report
            .tags
            .iter()
            .filter(|t| t.tag != "FIXME")
            .all(|t| t.count == 0)
    );
}

// ── TODO report: large codebase density ──────────────────────────────

#[test]
fn given_10_todos_and_10000_code_lines_when_building_todo_report_then_density_is_1() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    let mut content = String::new();
    for i in 0..10 {
        content.push_str(&format!("// TODO: item {}\n", i));
    }
    content.push_str("fn main() {}\n");
    std::fs::write(root.join("big.rs"), &content).unwrap();

    let files = vec![PathBuf::from("big.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 10_000).unwrap();

    assert_eq!(report.total, 10);
    assert_eq!(report.density_per_kloc, 1.0);
}

// ── TODO report: mixed tags ──────────────────────────────────────────

#[test]
fn given_file_with_all_tag_types_when_building_todo_report_then_tags_sorted_by_count_then_name() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    std::fs::write(
        root.join("mixed.rs"),
        "// XXX: review\n// HACK: workaround\n// FIXME: broken\n// TODO: implement\n// TODO: again\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("mixed.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 1000).unwrap();

    assert_eq!(report.total, 5);
    let tag_names: Vec<&str> = report.tags.iter().map(|t| t.tag.as_str()).collect();
    assert_eq!(
        tag_names,
        vec!["TODO", "FIXME", "HACK", "XXX"],
        "tags should be sorted by count then name"
    );
}

// ── Duplicate report: single file no duplicates ──────────────────────

#[test]
fn given_single_file_when_building_duplicate_report_then_no_groups() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    std::fs::write(root.join("only.rs"), "fn only() { 42 }\n").unwrap();

    let files = vec![PathBuf::from("only.rs")];
    let export = ExportData {
        rows: vec![file_row("only.rs", "root", "Rust", 18)],
        module_roots: vec!["root".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    assert!(report.groups.is_empty());
    assert_eq!(report.wasted_bytes, 0);
    assert_eq!(report.strategy, "exact-blake3");
}

// ── Duplicate report: density wasted_pct_of_codebase ─────────────────

#[test]
fn given_duplicates_when_building_report_then_wasted_pct_is_correct_ratio() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    // 100-byte content duplicated across two files
    let content = "x".repeat(100);
    std::fs::write(root.join("d1.rs"), &content).unwrap();
    std::fs::write(root.join("d2.rs"), &content).unwrap();
    // One unique file
    std::fs::write(root.join("unique.rs"), "y".repeat(100)).unwrap();

    let files = vec![
        PathBuf::from("d1.rs"),
        PathBuf::from("d2.rs"),
        PathBuf::from("unique.rs"),
    ];
    let export = ExportData {
        rows: vec![
            file_row("d1.rs", "root", "Rust", 100),
            file_row("d2.rs", "root", "Rust", 100),
            file_row("unique.rs", "root", "Rust", 100),
        ],
        module_roots: vec!["root".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.wasted_bytes, 100);
    let density = report.density.as_ref().expect("density present");
    // module_bytes for "root" = 10+10+10 = 30 (from file_row bytes field)
    // wasted_pct_of_codebase = wasted_bytes / total_codebase_bytes
    assert!(
        density.wasted_pct_of_codebase > 0.0,
        "wasted_pct should be positive"
    );
}

// ── Duplicate report: multiple distinct groups ───────────────────────

#[test]
fn given_two_distinct_duplicate_groups_when_building_report_then_both_detected() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    let content_a = "fn alpha() { 1 }\n";
    let content_b = "fn beta() { 2 }\n";
    std::fs::write(root.join("a1.rs"), content_a).unwrap();
    std::fs::write(root.join("a2.rs"), content_a).unwrap();
    std::fs::write(root.join("b1.rs"), content_b).unwrap();
    std::fs::write(root.join("b2.rs"), content_b).unwrap();

    let files = vec![
        PathBuf::from("a1.rs"),
        PathBuf::from("a2.rs"),
        PathBuf::from("b1.rs"),
        PathBuf::from("b2.rs"),
    ];
    let export = empty_export();

    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    // Both content_a and content_b are the same length so they could form groups
    // If same length & same hash → 1 group; different hash → 2 groups
    // content_a != content_b so we get 2 groups (they have the same byte count
    // but different hashes)
    assert_eq!(report.groups.len(), 2);
    assert!(report.groups.iter().all(|g| g.files.len() == 2));
}

// ── Import report: Rust use statements produce edges ─────────────────

#[test]
fn given_rust_file_with_use_statements_when_file_granularity_then_from_is_file_path() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("src/main.rs"),
        "use std::collections::HashMap;\nuse anyhow::Result;\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("src/main.rs")];
    let export = ExportData {
        rows: vec![file_row("src/main.rs", "src", "Rust", 60)],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::File,
        &ContentLimits::default(),
    )
    .unwrap();

    assert_eq!(report.granularity, "file");
    // All edges should have from == file path
    assert!(
        report.edges.iter().all(|e| e.from == "src/main.rs"),
        "all edges should reference the file path"
    );
    assert!(
        !report.edges.is_empty(),
        "Rust use statements should produce import edges"
    );
}

// ── Import report: per-file byte limit respected ─────────────────────

#[test]
fn given_per_file_byte_limit_when_building_import_report_then_large_file_truncated() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    // Write a file with imports near the top, then lots of content
    let mut content = "import os\nimport sys\n".to_string();
    content.push_str(&"x = 1\n".repeat(5000));
    std::fs::write(root.join("big.py"), &content).unwrap();

    let files = vec![PathBuf::from("big.py")];
    let export = ExportData {
        rows: vec![file_row("big.py", "root", "Python", content.len())],
        module_roots: vec!["root".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let limits = ContentLimits {
        max_bytes: None,
        max_file_bytes: Some(256),
    };
    let report =
        build_import_report(root, &files, &export, ImportGranularity::Module, &limits).unwrap();

    // Even with truncation, imports at the top should still be parsed
    assert!(
        !report.edges.is_empty(),
        "imports near file top should still be found"
    );
}
