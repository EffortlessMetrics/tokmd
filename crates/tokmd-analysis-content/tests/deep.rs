//! Deep integration tests for `tokmd-analysis-content`.
//!
//! Covers: TODO detection in various comment styles, import/dependency
//! extraction, duplicate detection, empty file handling, large file handling,
//! and serialization of results.

use std::collections::BTreeMap;
use std::path::PathBuf;

use tokmd_analysis_content::{
    ContentLimits, ImportGranularity, build_duplicate_report, build_import_report,
    build_todo_report,
};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn make_export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

// ---------------------------------------------------------------------------
// 1. TODO detection in various comment styles
// ---------------------------------------------------------------------------

#[test]
fn todo_in_c_style_line_comment() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(
        root.join("main.rs"),
        "// TODO: implement this\nfn main() {}\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("main.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 1000).unwrap();

    assert_eq!(report.total, 1);
    let tags: BTreeMap<String, usize> = report
        .tags
        .iter()
        .map(|t| (t.tag.clone(), t.count))
        .collect();
    assert_eq!(tags.get("TODO"), Some(&1));
}

#[test]
fn fixme_in_block_comment() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(
        root.join("code.c"),
        "/* FIXME: memory leak */\nint x = 0;\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("code.c")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 500).unwrap();

    assert_eq!(report.total, 1);
    let tags: BTreeMap<String, usize> = report
        .tags
        .iter()
        .map(|t| (t.tag.clone(), t.count))
        .collect();
    assert_eq!(tags.get("FIXME"), Some(&1));
}

#[test]
fn hack_in_python_comment() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(
        root.join("script.py"),
        "# HACK: workaround for bug\ndef main():\n    pass\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("script.py")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 500).unwrap();

    assert_eq!(report.total, 1);
    let tags: BTreeMap<String, usize> = report
        .tags
        .iter()
        .map(|t| (t.tag.clone(), t.count))
        .collect();
    assert_eq!(tags.get("HACK"), Some(&1));
}

#[test]
fn xxx_tag_detected() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(root.join("code.rs"), "// XXX: needs review\nfn foo() {}\n").unwrap();

    let files = vec![PathBuf::from("code.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 500).unwrap();

    assert_eq!(report.total, 1);
    let tags: BTreeMap<String, usize> = report
        .tags
        .iter()
        .map(|t| (t.tag.clone(), t.count))
        .collect();
    assert_eq!(tags.get("XXX"), Some(&1));
}

#[test]
fn all_four_tags_in_one_file() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let content = concat!(
        "// TODO: first\n",
        "// FIXME: second\n",
        "// HACK: third\n",
        "// XXX: fourth\n",
        "fn main() {}\n",
    );
    std::fs::write(root.join("mixed.rs"), content).unwrap();

    let files = vec![PathBuf::from("mixed.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 1000).unwrap();

    assert_eq!(report.total, 4);
    let tags: BTreeMap<String, usize> = report
        .tags
        .iter()
        .map(|t| (t.tag.clone(), t.count))
        .collect();
    assert_eq!(tags.get("TODO"), Some(&1));
    assert_eq!(tags.get("FIXME"), Some(&1));
    assert_eq!(tags.get("HACK"), Some(&1));
    assert_eq!(tags.get("XXX"), Some(&1));
}

#[test]
fn multiple_todos_on_same_line() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(
        root.join("double.rs"),
        "// TODO: first TODO: second on same line\nfn main() {}\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("double.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 1000).unwrap();

    assert_eq!(report.total, 2);
}

#[test]
fn case_insensitive_todo_detection() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(
        root.join("case.rs"),
        "// todo: lowercase\n// Todo: mixed\n// TODO: upper\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("case.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 1000).unwrap();

    // count_tags does case-insensitive matching
    assert_eq!(report.total, 3);
}

#[test]
fn todos_across_multiple_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(root.join("a.rs"), "// TODO: in file a\n").unwrap();
    std::fs::write(root.join("b.rs"), "// TODO: in file b\n// FIXME: also b\n").unwrap();

    let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 1000).unwrap();

    assert_eq!(report.total, 3);
}

#[test]
fn todo_density_calculation() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(
        root.join("code.rs"),
        "// TODO: a\n// TODO: b\n// TODO: c\n// TODO: d\n// TODO: e\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("code.rs")];
    // 5 TODOs in 2000 lines of code = 2.5 per kLOC
    let report = build_todo_report(root, &files, &ContentLimits::default(), 2000).unwrap();

    assert_eq!(report.total, 5);
    assert_eq!(report.density_per_kloc, 2.5);
}

#[test]
fn zero_code_lines_density_is_zero() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(root.join("code.rs"), "// TODO: something\n").unwrap();

    let files = vec![PathBuf::from("code.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 0).unwrap();

    assert_eq!(report.total, 1);
    assert_eq!(report.density_per_kloc, 0.0);
}

#[test]
fn binary_file_skipped_for_todo_scan() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(root.join("binary.bin"), b"\x00\x01\x02TODO\x00\xff").unwrap();

    let files = vec![PathBuf::from("binary.bin")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 1000).unwrap();

    assert_eq!(report.total, 0);
}

#[test]
fn no_tags_file_returns_zero_total() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(
        root.join("clean.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("clean.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 500).unwrap();

    assert_eq!(report.total, 0);
}

// ---------------------------------------------------------------------------
// 2. Import/dependency extraction
// ---------------------------------------------------------------------------

#[test]
fn rust_use_statements_extracted() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(
        root.join("lib.rs"),
        "use std::io;\nuse serde::Serialize;\nfn main() {}\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("lib.rs")];
    let export = make_export(vec![file_row("lib.rs", "src", "Rust", 60)]);
    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();

    assert!(report.edges.len() >= 2);
    assert_eq!(report.granularity, "module");
}

#[test]
fn python_imports_extracted() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(
        root.join("app.py"),
        "import os\nimport sys\nfrom pathlib import Path\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("app.py")];
    let export = make_export(vec![file_row("app.py", "src", "Python", 50)]);
    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();

    assert!(report.edges.len() >= 2);
    assert!(report.edges.iter().all(|e| e.from == "src"));
}

#[test]
fn typescript_imports_extracted() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(
        root.join("index.ts"),
        "import React from \"react\";\nimport { useState } from \"react\";\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("index.ts")];
    let export = make_export(vec![file_row("index.ts", "web", "TypeScript", 70)]);
    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();

    assert!(!report.edges.is_empty());
    assert!(report.edges.iter().all(|e| e.from == "web"));
}

#[test]
fn file_granularity_uses_file_path_as_from() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(root.join("main.py"), "import requests\n").unwrap();

    let files = vec![PathBuf::from("main.py")];
    let export = make_export(vec![file_row("main.py", "pkg", "Python", 20)]);
    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::File,
        &ContentLimits::default(),
    )
    .unwrap();

    assert_eq!(report.granularity, "file");
    assert_eq!(report.edges.len(), 1);
    assert_eq!(report.edges[0].from, "main.py");
}

#[test]
fn unsupported_language_skipped_for_imports() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(root.join("data.json"), "{\"import\": \"not real\"}\n").unwrap();

    let files = vec![PathBuf::from("data.json")];
    let export = make_export(vec![file_row("data.json", "root", "JSON", 30)]);
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
fn import_edges_sorted_by_count_descending() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    // Multiple use of serde, single use of tokio
    std::fs::write(
        root.join("lib.rs"),
        "use serde::Serialize;\nuse serde::Deserialize;\nuse tokio::spawn;\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("lib.rs")];
    let export = make_export(vec![file_row("lib.rs", "src", "Rust", 80)]);
    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();

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
fn import_max_bytes_budget_respected() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(root.join("a.py"), "import os\n").unwrap();
    std::fs::write(root.join("b.py"), "import sys\n").unwrap();

    let files = vec![PathBuf::from("a.py"), PathBuf::from("b.py")];
    let export = make_export(vec![
        file_row("a.py", "root", "Python", 10),
        file_row("b.py", "root", "Python", 10),
    ]);

    let limits = ContentLimits {
        max_bytes: Some(5),
        max_file_bytes: None,
    };
    let report =
        build_import_report(root, &files, &export, ImportGranularity::Module, &limits).unwrap();

    assert!(report.edges.len() <= 1);
}

// ---------------------------------------------------------------------------
// 3. Duplicate detection
// ---------------------------------------------------------------------------

#[test]
fn two_identical_files_form_one_group() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let content = "fn duplicate() { println!(\"hello\"); }\n";
    std::fs::write(root.join("a.rs"), content).unwrap();
    std::fs::write(root.join("b.rs"), content).unwrap();

    let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];
    let export = make_export(vec![
        file_row("a.rs", "root", "Rust", content.len()),
        file_row("b.rs", "root", "Rust", content.len()),
    ]);
    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.groups[0].files.len(), 2);
    assert_eq!(report.wasted_bytes, content.len() as u64);
    assert_eq!(report.strategy, "exact-blake3");
}

#[test]
fn three_identical_files_waste_2x_size() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let content = "identical content for triplication\n";
    std::fs::write(root.join("a.txt"), content).unwrap();
    std::fs::write(root.join("b.txt"), content).unwrap();
    std::fs::write(root.join("c.txt"), content).unwrap();

    let files = vec![
        PathBuf::from("a.txt"),
        PathBuf::from("b.txt"),
        PathBuf::from("c.txt"),
    ];
    let export = make_export(vec![
        file_row("a.txt", "root", "Text", content.len()),
        file_row("b.txt", "root", "Text", content.len()),
        file_row("c.txt", "root", "Text", content.len()),
    ]);
    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.wasted_bytes, 2 * content.len() as u64);
}

#[test]
fn no_duplicates_yields_empty_groups() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(root.join("x.rs"), "fn unique_x() {}\n").unwrap();
    std::fs::write(root.join("y.rs"), "fn unique_y() {}\n").unwrap();

    let files = vec![PathBuf::from("x.rs"), PathBuf::from("y.rs")];
    let export = make_export(vec![
        file_row("x.rs", "root", "Rust", 18),
        file_row("y.rs", "root", "Rust", 18),
    ]);
    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    assert!(report.groups.is_empty());
    assert_eq!(report.wasted_bytes, 0);
}

#[test]
fn duplicate_groups_sorted_by_size_descending() {
    let temp = tempfile::tempdir().unwrap();
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
    let export = make_export(vec![]);
    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    assert_eq!(report.groups.len(), 2);
    assert!(report.groups[0].bytes >= report.groups[1].bytes);
}

#[test]
fn duplicate_files_within_group_sorted_alphabetically() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let content = "same content for sort test\n";
    std::fs::write(root.join("z.txt"), content).unwrap();
    std::fs::write(root.join("a.txt"), content).unwrap();
    std::fs::write(root.join("m.txt"), content).unwrap();

    let files = vec![
        PathBuf::from("z.txt"),
        PathBuf::from("a.txt"),
        PathBuf::from("m.txt"),
    ];
    let export = make_export(vec![]);
    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    assert_eq!(report.groups.len(), 1);
    let group_files = &report.groups[0].files;
    let mut sorted = group_files.clone();
    sorted.sort();
    assert_eq!(group_files, &sorted, "files within group should be sorted");
}

#[test]
fn density_report_has_correct_metrics() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let content = "exactly same content for density test\n";
    std::fs::write(root.join("d1.rs"), content).unwrap();
    std::fs::write(root.join("d2.rs"), content).unwrap();

    let files = vec![PathBuf::from("d1.rs"), PathBuf::from("d2.rs")];
    let export = make_export(vec![
        file_row("d1.rs", "src", "Rust", content.len()),
        file_row("d2.rs", "src", "Rust", content.len()),
    ]);
    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    let density = report.density.as_ref().expect("density present");
    assert_eq!(density.duplicate_groups, 1);
    assert_eq!(density.duplicate_files, 2);
    assert_eq!(density.wasted_bytes, content.len() as u64);
    assert!(density.wasted_pct_of_codebase > 0.0);
}

#[test]
fn density_by_module_tracks_per_module_metrics() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::create_dir_all(root.join("mod_a")).unwrap();
    std::fs::create_dir_all(root.join("mod_b")).unwrap();

    let content = "cross-module duplicate content\n";
    std::fs::write(root.join("mod_a/dup.rs"), content).unwrap();
    std::fs::write(root.join("mod_b/dup.rs"), content).unwrap();

    let files = vec![PathBuf::from("mod_a/dup.rs"), PathBuf::from("mod_b/dup.rs")];
    let export = make_export(vec![
        file_row("mod_a/dup.rs", "mod_a", "Rust", content.len()),
        file_row("mod_b/dup.rs", "mod_b", "Rust", content.len()),
    ]);
    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    let density = report.density.as_ref().expect("density present");
    assert!(density.by_module.len() >= 2);
}

#[test]
fn max_file_bytes_skips_large_files_for_duplicates() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let big = "x".repeat(1000);
    std::fs::write(root.join("big1.txt"), &big).unwrap();
    std::fs::write(root.join("big2.txt"), &big).unwrap();

    let files = vec![PathBuf::from("big1.txt"), PathBuf::from("big2.txt")];
    let limits = ContentLimits {
        max_bytes: None,
        max_file_bytes: Some(100),
    };
    let report = build_duplicate_report(root, &files, &make_export(vec![]), &limits).unwrap();

    assert!(report.groups.is_empty());
}

// ---------------------------------------------------------------------------
// 4. Empty file handling
// ---------------------------------------------------------------------------

#[test]
fn empty_file_list_todo_report() {
    let temp = tempfile::tempdir().unwrap();
    let report = build_todo_report(temp.path(), &[], &ContentLimits::default(), 1000).unwrap();

    assert_eq!(report.total, 0);
    assert_eq!(report.density_per_kloc, 0.0);
}

#[test]
fn empty_file_list_duplicate_report() {
    let temp = tempfile::tempdir().unwrap();
    let report = build_duplicate_report(
        temp.path(),
        &[],
        &make_export(vec![]),
        &ContentLimits::default(),
    )
    .unwrap();

    assert!(report.groups.is_empty());
    assert_eq!(report.wasted_bytes, 0);
}

#[test]
fn empty_file_list_import_report() {
    let temp = tempfile::tempdir().unwrap();
    let report = build_import_report(
        temp.path(),
        &[],
        &make_export(vec![]),
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();

    assert!(report.edges.is_empty());
    assert_eq!(report.granularity, "module");
}

#[test]
fn zero_byte_files_not_treated_as_duplicates() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(root.join("empty1.txt"), "").unwrap();
    std::fs::write(root.join("empty2.txt"), "").unwrap();

    let files = vec![PathBuf::from("empty1.txt"), PathBuf::from("empty2.txt")];
    let report = build_duplicate_report(
        root,
        &files,
        &make_export(vec![]),
        &ContentLimits::default(),
    )
    .unwrap();

    assert!(report.groups.is_empty());
}

#[test]
fn empty_text_file_for_todo_yields_zero() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(root.join("blank.rs"), "").unwrap();

    let files = vec![PathBuf::from("blank.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 500).unwrap();

    assert_eq!(report.total, 0);
}

// ---------------------------------------------------------------------------
// 5. Large file handling
// ---------------------------------------------------------------------------

#[test]
fn large_file_todo_scan_with_max_file_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();

    // Create a file with TODO at the end, beyond the byte limit
    let mut content = String::new();
    for _ in 0..500 {
        content.push_str("// normal line of code\n");
    }
    content.push_str("// TODO: hidden at end\n");
    std::fs::write(root.join("large.rs"), &content).unwrap();

    let files = vec![PathBuf::from("large.rs")];
    let limits = ContentLimits {
        max_bytes: None,
        max_file_bytes: Some(200),
    };
    let report = build_todo_report(root, &files, &limits, 1000).unwrap();

    // With 200 byte limit, the TODO at the end should not be found
    assert_eq!(report.total, 0);
}

#[test]
fn large_file_todo_scan_within_limits_finds_tags() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();

    // TODO at the start, within limits
    let mut content = String::from("// TODO: found at start\n");
    for _ in 0..500 {
        content.push_str("// normal line\n");
    }
    std::fs::write(root.join("large.rs"), &content).unwrap();

    let files = vec![PathBuf::from("large.rs")];
    let limits = ContentLimits {
        max_bytes: None,
        max_file_bytes: Some(200),
    };
    let report = build_todo_report(root, &files, &limits, 1000).unwrap();

    assert_eq!(report.total, 1);
}

#[test]
fn max_bytes_budget_limits_total_processing() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();

    std::fs::write(root.join("f1.rs"), "// TODO: in file 1\n").unwrap();
    std::fs::write(root.join("f2.rs"), "// TODO: in file 2\n").unwrap();
    std::fs::write(root.join("f3.rs"), "// TODO: in file 3\n").unwrap();

    let files = vec![
        PathBuf::from("f1.rs"),
        PathBuf::from("f2.rs"),
        PathBuf::from("f3.rs"),
    ];
    let limits = ContentLimits {
        max_bytes: Some(15),
        max_file_bytes: None,
    };
    let report = build_todo_report(root, &files, &limits, 1000).unwrap();

    // With a tiny budget, not all files are scanned
    assert!(report.total < 3);
}

#[test]
fn many_duplicate_files_produces_correct_wasted_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();

    let content = "common content for many-file test\n";
    let mut files = Vec::new();
    for i in 0..10 {
        let name = format!("file{}.txt", i);
        std::fs::write(root.join(&name), content).unwrap();
        files.push(PathBuf::from(name));
    }

    let report = build_duplicate_report(
        root,
        &files,
        &make_export(vec![]),
        &ContentLimits::default(),
    )
    .unwrap();

    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.groups[0].files.len(), 10);
    // 9 copies are "wasted"
    assert_eq!(report.wasted_bytes, 9 * content.len() as u64);
}

// ---------------------------------------------------------------------------
// 6. Serialization of results
// ---------------------------------------------------------------------------

#[test]
fn todo_report_serializes_to_json() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(
        root.join("code.rs"),
        "// TODO: first\n// FIXME: second\nfn main() {}\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("code.rs")];
    let report = build_todo_report(root, &files, &ContentLimits::default(), 1000).unwrap();

    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains("\"total\""));
    assert!(json.contains("\"density_per_kloc\""));
    assert!(json.contains("\"tags\""));

    let deser: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(deser["total"], 2);
}

#[test]
fn duplicate_report_serializes_to_json() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let content = "duplicate content for serialization test\n";
    std::fs::write(root.join("a.rs"), content).unwrap();
    std::fs::write(root.join("b.rs"), content).unwrap();

    let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];
    let export = make_export(vec![
        file_row("a.rs", "root", "Rust", content.len()),
        file_row("b.rs", "root", "Rust", content.len()),
    ]);
    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains("\"groups\""));
    assert!(json.contains("\"wasted_bytes\""));
    assert!(json.contains("\"strategy\""));
    assert!(json.contains("exact-blake3"));

    let deser: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(deser["groups"].as_array().unwrap().len(), 1);
}

#[test]
fn import_report_serializes_to_json() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(root.join("lib.rs"), "use serde::Serialize;\n").unwrap();

    let files = vec![PathBuf::from("lib.rs")];
    let export = make_export(vec![file_row("lib.rs", "src", "Rust", 30)]);
    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();

    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains("\"granularity\""));
    assert!(json.contains("\"edges\""));
    assert!(json.contains("module"));
}

#[test]
fn todo_report_is_deterministic() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(
        root.join("code.rs"),
        "// TODO: a\n// FIXME: b\n// HACK: c\n// XXX: d\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("code.rs")];
    let r1 = build_todo_report(root, &files, &ContentLimits::default(), 1000).unwrap();
    let r2 = build_todo_report(root, &files, &ContentLimits::default(), 1000).unwrap();

    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(j1, j2, "TODO reports should be deterministic");
}

#[test]
fn duplicate_report_is_deterministic() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let content = "deterministic duplicate test content\n";
    std::fs::write(root.join("a.rs"), content).unwrap();
    std::fs::write(root.join("b.rs"), content).unwrap();

    let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];
    let export = make_export(vec![
        file_row("a.rs", "root", "Rust", content.len()),
        file_row("b.rs", "root", "Rust", content.len()),
    ]);

    let r1 = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();
    let r2 = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(j1, j2, "Duplicate reports should be deterministic");
}

// ---------------------------------------------------------------------------
// Additional: ContentLimits edge cases
// ---------------------------------------------------------------------------

#[test]
fn content_limits_default_has_no_limits() {
    let limits = ContentLimits::default();
    assert!(limits.max_bytes.is_none());
    assert!(limits.max_file_bytes.is_none());
}

#[test]
fn content_limits_is_debug_clone_copy() {
    let limits = ContentLimits {
        max_bytes: Some(1024),
        max_file_bytes: Some(512),
    };
    let debug = format!("{:?}", limits);
    assert!(debug.contains("ContentLimits"));
    let cloned = limits;
    assert_eq!(cloned.max_bytes, Some(1024));
    assert_eq!(cloned.max_file_bytes, Some(512));
}

#[test]
fn import_granularity_is_debug_clone_copy() {
    let g = ImportGranularity::Module;
    let debug = format!("{:?}", g);
    assert!(debug.contains("Module"));
    let _copy = g;
}
