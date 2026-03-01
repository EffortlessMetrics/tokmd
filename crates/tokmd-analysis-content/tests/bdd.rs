//! BDD-style integration tests for tokmd-analysis-content.
//!
//! Covers: build_todo_report, build_duplicate_report, build_import_report
//! with Given/When/Then naming.

use std::path::PathBuf;

use tokmd_analysis_content::{
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

// ── build_todo_report ────────────────────────────────────────────────

#[test]
fn given_files_with_todos_when_building_todo_report_then_counts_each_tag() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    std::fs::write(
        root.join("main.rs"),
        "// TODO: implement feature\n// FIXME: broken\n// HACK: workaround\n// XXX: review\nfn main() {}\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("main.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 1000).unwrap();

    assert_eq!(report.total, 4);
    let tag_map: std::collections::BTreeMap<String, usize> = report
        .tags
        .iter()
        .map(|t| (t.tag.clone(), t.count))
        .collect();
    assert_eq!(tag_map.get("TODO"), Some(&1));
    assert_eq!(tag_map.get("FIXME"), Some(&1));
    assert_eq!(tag_map.get("HACK"), Some(&1));
    assert_eq!(tag_map.get("XXX"), Some(&1));
}

#[test]
fn given_no_tags_when_building_todo_report_then_total_is_zero() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    std::fs::write(root.join("clean.rs"), "fn main() {}\n").unwrap();

    let files = vec![PathBuf::from("clean.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 500).unwrap();

    assert_eq!(report.total, 0);
    assert!(report.tags.is_empty() || report.tags.iter().all(|t| t.count == 0));
}

#[test]
fn given_zero_total_code_when_building_todo_report_then_density_is_zero() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    std::fs::write(root.join("a.rs"), "// TODO: something\n").unwrap();

    let files = vec![PathBuf::from("a.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 0).unwrap();

    assert_eq!(report.total, 1);
    assert_eq!(report.density_per_kloc, 0.0);
}

#[test]
fn given_1000_code_lines_and_3_todos_when_building_todo_report_then_density_is_3() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    std::fs::write(
        root.join("code.rs"),
        "// TODO: a\n// TODO: b\n// TODO: c\nfn main() {}\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("code.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 1000).unwrap();

    assert_eq!(report.total, 3);
    assert_eq!(report.density_per_kloc, 3.0);
}

#[test]
fn given_multiple_todos_in_one_line_when_building_todo_report_then_all_counted() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    // Two TODO tags on the same line
    std::fs::write(root.join("multi.rs"), "// TODO: first TODO: second\n").unwrap();

    let files = vec![PathBuf::from("multi.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 1000).unwrap();

    assert_eq!(report.total, 2);
}

#[test]
fn given_binary_file_when_building_todo_report_then_skipped() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    // Write binary content with null bytes
    std::fs::write(root.join("image.bin"), b"\x00\x01\x02TODO\x00\xff").unwrap();

    let files = vec![PathBuf::from("image.bin")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 1000).unwrap();

    assert_eq!(report.total, 0);
}

#[test]
fn given_max_bytes_limit_when_building_todo_report_then_stops_early() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    // First file is small, second file should be skipped
    std::fs::write(root.join("first.rs"), "// TODO: counted\n").unwrap();
    std::fs::write(root.join("second.rs"), "// TODO: skipped\n").unwrap();

    let files = vec![PathBuf::from("first.rs"), PathBuf::from("second.rs")];
    let limits = ContentLimits {
        max_bytes: Some(10), // very small budget
        max_file_bytes: None,
    };
    let report = build_todo_report(root, &files, &limits, 1000).unwrap();

    // Only the first file should be processed
    assert!(report.total <= 1);
}

#[test]
fn given_empty_file_list_when_building_todo_report_then_zero_total() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    let files: Vec<PathBuf> = vec![];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 500).unwrap();

    assert_eq!(report.total, 0);
    assert_eq!(report.density_per_kloc, 0.0);
}

// ── build_duplicate_report ───────────────────────────────────────────

#[test]
fn given_two_identical_files_when_building_duplicate_report_then_one_group_found() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    let content = "fn duplicate() { println!(\"hello\"); }\n";
    std::fs::write(root.join("a.rs"), content).unwrap();
    std::fs::write(root.join("b.rs"), content).unwrap();

    let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];
    let export = ExportData {
        rows: vec![
            file_row("a.rs", "root", "Rust", content.len()),
            file_row("b.rs", "root", "Rust", content.len()),
        ],
        module_roots: vec!["root".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.groups[0].files.len(), 2);
    assert_eq!(report.wasted_bytes, content.len() as u64);
    assert_eq!(report.strategy, "exact-blake3");
}

#[test]
fn given_no_duplicates_when_building_duplicate_report_then_empty_groups() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    std::fs::write(root.join("x.rs"), "fn x() {}\n").unwrap();
    std::fs::write(root.join("y.rs"), "fn y() {}\n").unwrap();

    let files = vec![PathBuf::from("x.rs"), PathBuf::from("y.rs")];
    let export = ExportData {
        rows: vec![
            file_row("x.rs", "root", "Rust", 10),
            file_row("y.rs", "root", "Rust", 10),
        ],
        module_roots: vec!["root".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    assert!(report.groups.is_empty());
    assert_eq!(report.wasted_bytes, 0);
}

#[test]
fn given_three_identical_files_when_building_duplicate_report_then_wasted_is_2x_size() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    let content = "identical content for triplication test\n";
    std::fs::write(root.join("a.txt"), content).unwrap();
    std::fs::write(root.join("b.txt"), content).unwrap();
    std::fs::write(root.join("c.txt"), content).unwrap();

    let files = vec![
        PathBuf::from("a.txt"),
        PathBuf::from("b.txt"),
        PathBuf::from("c.txt"),
    ];
    let export = ExportData {
        rows: vec![
            file_row("a.txt", "root", "Text", content.len()),
            file_row("b.txt", "root", "Text", content.len()),
            file_row("c.txt", "root", "Text", content.len()),
        ],
        module_roots: vec!["root".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.groups[0].files.len(), 3);
    // Two copies are "wasted" (original + 2 copies → 2 wasted)
    assert_eq!(report.wasted_bytes, 2 * content.len() as u64);
}

#[test]
fn given_empty_files_when_building_duplicate_report_then_zero_byte_files_ignored() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    std::fs::write(root.join("empty1.txt"), "").unwrap();
    std::fs::write(root.join("empty2.txt"), "").unwrap();

    let files = vec![PathBuf::from("empty1.txt"), PathBuf::from("empty2.txt")];
    let export = empty_export();

    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    // Zero-byte files should not form duplicate groups
    assert!(report.groups.is_empty());
    assert_eq!(report.wasted_bytes, 0);
}

#[test]
fn given_duplicates_in_different_modules_when_building_duplicate_report_then_density_by_module() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    std::fs::create_dir_all(root.join("mod_a")).unwrap();
    std::fs::create_dir_all(root.join("mod_b")).unwrap();

    let content = "shared duplicate content across modules\n";
    std::fs::write(root.join("mod_a/dup.rs"), content).unwrap();
    std::fs::write(root.join("mod_b/dup.rs"), content).unwrap();

    let files = vec![PathBuf::from("mod_a/dup.rs"), PathBuf::from("mod_b/dup.rs")];
    let export = ExportData {
        rows: vec![
            file_row("mod_a/dup.rs", "mod_a", "Rust", content.len()),
            file_row("mod_b/dup.rs", "mod_b", "Rust", content.len()),
        ],
        module_roots: vec!["mod_a".to_string(), "mod_b".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    assert_eq!(report.groups.len(), 1);
    let density = report.density.as_ref().expect("density report present");
    assert_eq!(density.duplicate_files, 2);
    assert!(density.by_module.len() >= 2);
}

#[test]
fn given_file_exceeds_max_file_bytes_when_building_duplicate_report_then_skipped() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    let content = "x".repeat(1000);
    std::fs::write(root.join("big_a.txt"), &content).unwrap();
    std::fs::write(root.join("big_b.txt"), &content).unwrap();

    let files = vec![PathBuf::from("big_a.txt"), PathBuf::from("big_b.txt")];
    let export = empty_export();

    let limits = ContentLimits {
        max_bytes: None,
        max_file_bytes: Some(100), // both files exceed this
    };
    let report = build_duplicate_report(root, &files, &export, &limits).unwrap();

    assert!(report.groups.is_empty());
}

#[test]
fn given_duplicate_groups_when_building_duplicate_report_then_sorted_by_bytes_desc() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    let small = "sm\n";
    let large = "x".repeat(200) + "\n";
    std::fs::write(root.join("s1.txt"), small).unwrap();
    std::fs::write(root.join("s2.txt"), small).unwrap();
    std::fs::write(root.join("l1.txt"), &large).unwrap();
    std::fs::write(root.join("l2.txt"), &large).unwrap();

    let files = vec![
        PathBuf::from("s1.txt"),
        PathBuf::from("s2.txt"),
        PathBuf::from("l1.txt"),
        PathBuf::from("l2.txt"),
    ];
    let export = empty_export();

    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    assert_eq!(report.groups.len(), 2);
    // Larger group should appear first
    assert!(report.groups[0].bytes >= report.groups[1].bytes);
}

// ── build_import_report ──────────────────────────────────────────────

#[test]
fn given_empty_file_list_when_building_import_report_then_no_edges() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    let files: Vec<PathBuf> = vec![];
    let export = empty_export();

    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();

    assert!(report.edges.is_empty());
    assert_eq!(report.granularity, "module");
}

#[test]
fn given_unsupported_language_when_building_import_report_then_skipped() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    std::fs::write(root.join("data.json"), "{\"key\": \"value\"}\n").unwrap();

    let files = vec![PathBuf::from("data.json")];
    let export = ExportData {
        rows: vec![file_row("data.json", "root", "JSON", 20)],
        module_roots: vec!["root".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();

    assert!(report.edges.is_empty());
}

#[test]
fn given_file_not_in_export_when_building_import_report_then_skipped() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    std::fs::write(root.join("orphan.rs"), "use std::io;\n").unwrap();

    let files = vec![PathBuf::from("orphan.rs")];
    // Export has no matching row
    let export = empty_export();

    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();

    assert!(report.edges.is_empty());
}

#[test]
fn given_python_file_with_multiple_imports_when_building_import_report_then_all_edges_collected() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("src/app.py"),
        "import os\nimport sys\nfrom pathlib import Path\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("src/app.py")];
    let export = ExportData {
        rows: vec![file_row("src/app.py", "src", "Python", 50)],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();

    assert!(report.edges.len() >= 2, "expected multiple import edges");
    assert!(report.edges.iter().all(|e| e.from == "src"));
}

#[test]
fn given_max_bytes_limit_when_building_import_report_then_budget_respected() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    std::fs::write(root.join("a.py"), "import os\n").unwrap();
    std::fs::write(root.join("b.py"), "import sys\n").unwrap();

    let files = vec![PathBuf::from("a.py"), PathBuf::from("b.py")];
    let export = ExportData {
        rows: vec![
            file_row("a.py", "root", "Python", 10),
            file_row("b.py", "root", "Python", 10),
        ],
        module_roots: vec!["root".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let limits = ContentLimits {
        max_bytes: Some(5), // very small budget
        max_file_bytes: None,
    };
    let report =
        build_import_report(root, &files, &export, ImportGranularity::Module, &limits).unwrap();

    // With a 5-byte budget, at most one file can be processed
    assert!(report.edges.len() <= 1);
}

#[test]
fn given_import_edges_when_building_import_report_then_sorted_by_count_desc() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    std::fs::create_dir_all(root.join("lib")).unwrap();
    // File that imports the same module multiple times via different patterns
    std::fs::write(
        root.join("lib/main.rs"),
        "use serde::Serialize;\nuse serde::Deserialize;\nuse tokio::spawn;\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("lib/main.rs")];
    let export = ExportData {
        rows: vec![file_row("lib/main.rs", "lib", "Rust", 80)],
        module_roots: vec!["lib".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();

    // Edges should be sorted: highest count first
    for w in report.edges.windows(2) {
        assert!(
            w[0].count >= w[1].count,
            "edges not sorted by count desc: {} < {}",
            w[0].count,
            w[1].count
        );
    }
}

#[test]
fn given_file_granularity_when_building_import_report_then_granularity_field_is_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    let files: Vec<PathBuf> = vec![];
    let export = empty_export();

    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::File,
        &ContentLimits::default(),
    )
    .unwrap();

    assert_eq!(report.granularity, "file");
}

#[test]
fn given_module_granularity_when_building_import_report_then_granularity_field_is_module() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    let files: Vec<PathBuf> = vec![];
    let export = empty_export();

    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();

    assert_eq!(report.granularity, "module");
}

// ── ContentLimits ────────────────────────────────────────────────────

#[test]
fn given_default_content_limits_then_all_none() {
    let limits = ContentLimits::default();
    assert!(limits.max_bytes.is_none());
    assert!(limits.max_file_bytes.is_none());
}

// ── duplicate report density metrics ─────────────────────────────────

#[test]
fn given_duplicates_when_building_duplicate_report_then_density_has_correct_counts() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    let content = "exactly the same content here\n";
    std::fs::write(root.join("dup1.rs"), content).unwrap();
    std::fs::write(root.join("dup2.rs"), content).unwrap();

    let files = vec![PathBuf::from("dup1.rs"), PathBuf::from("dup2.rs")];
    let export = ExportData {
        rows: vec![
            file_row("dup1.rs", "src", "Rust", content.len()),
            file_row("dup2.rs", "src", "Rust", content.len()),
        ],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    let density = report.density.as_ref().expect("density present");
    assert_eq!(density.duplicate_groups, 1);
    assert_eq!(density.duplicate_files, 2);
    assert_eq!(density.wasted_bytes, content.len() as u64);
    assert!(density.wasted_pct_of_codebase > 0.0);
}
