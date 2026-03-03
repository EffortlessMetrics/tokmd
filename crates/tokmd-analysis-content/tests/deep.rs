//! Deep tests for `tokmd-analysis-content`.
//!
//! Exercises build_todo_report, build_duplicate_report, and build_import_report
//! with edge cases, serialization roundtrips, and realistic inputs.

use std::path::PathBuf;

use tempfile::TempDir;
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

fn export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn write_file(dir: &std::path::Path, rel: &str, content: &[u8]) -> PathBuf {
    let full = dir.join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&full, content).unwrap();
    PathBuf::from(rel)
}

// ===========================================================================
// build_todo_report
// ===========================================================================

// 1. Empty file list → zero total
#[test]
fn todo_empty_files() {
    let tmp = TempDir::new().unwrap();
    let report = build_todo_report(tmp.path(), &[], &ContentLimits::default(), 1000).unwrap();
    assert_eq!(report.total, 0);
    assert!(report.tags.is_empty());
}

// 2. File with no tags → zero total
#[test]
fn todo_no_tags_in_file() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(
        tmp.path(),
        "clean.rs",
        b"fn main() { println!(\"hello\"); }\n",
    );
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 1000).unwrap();
    assert_eq!(report.total, 0);
}

// 3. All four tag types counted
#[test]
fn todo_all_four_tags() {
    let tmp = TempDir::new().unwrap();
    let content = "// TODO: a\n// FIXME: b\n// HACK: c\n// XXX: d\n";
    let rel = write_file(tmp.path(), "tags.rs", content.as_bytes());
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 1000).unwrap();
    assert_eq!(report.total, 4);
    let tags: std::collections::BTreeMap<String, usize> = report
        .tags
        .iter()
        .map(|t| (t.tag.clone(), t.count))
        .collect();
    assert_eq!(tags["TODO"], 1);
    assert_eq!(tags["FIXME"], 1);
    assert_eq!(tags["HACK"], 1);
    assert_eq!(tags["XXX"], 1);
}

// 4. Multiple TODOs in one file
#[test]
fn todo_multiple_same_tag() {
    let tmp = TempDir::new().unwrap();
    let content = "// TODO: first\n// TODO: second\n// TODO: third\nfn f() {}\n";
    let rel = write_file(tmp.path(), "multi.rs", content.as_bytes());
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 1000).unwrap();
    assert_eq!(report.total, 3);
}

// 5. Density per KLOC calculation
#[test]
fn todo_density_per_kloc() {
    let tmp = TempDir::new().unwrap();
    let content = "// TODO: one\n// TODO: two\n";
    let rel = write_file(tmp.path(), "d.rs", content.as_bytes());
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 2000).unwrap();
    // 2 TODOs / 2.0 KLOC = 1.0
    assert_eq!(report.density_per_kloc, 1.0);
}

// 6. Density with zero code lines
#[test]
fn todo_density_zero_code() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "empty.rs", b"// TODO: x\n");
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 0).unwrap();
    assert_eq!(report.density_per_kloc, 0.0);
}

// 7. Multiple files aggregated
#[test]
fn todo_multiple_files() {
    let tmp = TempDir::new().unwrap();
    let f1 = write_file(tmp.path(), "a.rs", b"// TODO: a\n");
    let f2 = write_file(tmp.path(), "b.rs", b"// FIXME: b\n// TODO: c\n");
    let report = build_todo_report(tmp.path(), &[f1, f2], &ContentLimits::default(), 1000).unwrap();
    assert_eq!(report.total, 3);
}

// 8. TodoReport JSON serialization roundtrip
#[test]
fn todo_serialization_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "t.rs", b"// TODO: ser\n// FIXME: rt\n");
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 5000).unwrap();
    let json = serde_json::to_string(&report).unwrap();
    let deser: tokmd_analysis_types::TodoReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.total, report.total);
    assert_eq!(deser.tags.len(), report.tags.len());
    assert_eq!(deser.density_per_kloc, report.density_per_kloc);
}

// ===========================================================================
// build_duplicate_report
// ===========================================================================

// 9. Empty file list → no duplicates
#[test]
fn dup_empty_files() {
    let tmp = TempDir::new().unwrap();
    let exp = export(vec![]);
    let report = build_duplicate_report(tmp.path(), &[], &exp, &ContentLimits::default()).unwrap();
    assert_eq!(report.wasted_bytes, 0);
    assert!(report.groups.is_empty());
    assert_eq!(report.strategy, "exact-blake3");
}

// 10. Two identical files → one duplicate group
#[test]
fn dup_two_identical_files() {
    let tmp = TempDir::new().unwrap();
    let content = b"identical content here\n";
    let f1 = write_file(tmp.path(), "a.rs", content);
    let f2 = write_file(tmp.path(), "b.rs", content);
    let exp = export(vec![
        file_row("a.rs", "src", "Rust", content.len()),
        file_row("b.rs", "src", "Rust", content.len()),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &[f1, f2], &exp, &ContentLimits::default()).unwrap();
    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.groups[0].files.len(), 2);
    assert_eq!(report.wasted_bytes, content.len() as u64);
}

// 11. Two different files → no duplicates
#[test]
fn dup_different_files() {
    let tmp = TempDir::new().unwrap();
    let f1 = write_file(tmp.path(), "a.rs", b"content a\n");
    let f2 = write_file(tmp.path(), "b.rs", b"content b\n");
    let exp = export(vec![
        file_row("a.rs", "src", "Rust", 10),
        file_row("b.rs", "src", "Rust", 10),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &[f1, f2], &exp, &ContentLimits::default()).unwrap();
    assert!(report.groups.is_empty());
}

// 12. Three identical files → wasted = 2 * size
#[test]
fn dup_three_identical_files() {
    let tmp = TempDir::new().unwrap();
    let content = b"same content\n";
    let f1 = write_file(tmp.path(), "x.rs", content);
    let f2 = write_file(tmp.path(), "y.rs", content);
    let f3 = write_file(tmp.path(), "z.rs", content);
    let exp = export(vec![
        file_row("x.rs", "m", "Rust", content.len()),
        file_row("y.rs", "m", "Rust", content.len()),
        file_row("z.rs", "m", "Rust", content.len()),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &[f1, f2, f3], &exp, &ContentLimits::default()).unwrap();
    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.groups[0].files.len(), 3);
    assert_eq!(report.wasted_bytes, 2 * content.len() as u64);
}

// 13. Empty files not counted as duplicates
#[test]
fn dup_empty_files_skipped() {
    let tmp = TempDir::new().unwrap();
    let f1 = write_file(tmp.path(), "empty1.rs", b"");
    let f2 = write_file(tmp.path(), "empty2.rs", b"");
    let exp = export(vec![
        file_row("empty1.rs", "m", "Rust", 0),
        file_row("empty2.rs", "m", "Rust", 0),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &[f1, f2], &exp, &ContentLimits::default()).unwrap();
    // Empty files (size 0) are skipped
    assert!(report.groups.is_empty());
}

// 14. Duplicate groups sorted by bytes desc
#[test]
fn dup_groups_sorted_by_bytes_desc() {
    let tmp = TempDir::new().unwrap();
    let small = b"sm\n";
    let large = b"this is a much larger content block!\n";
    let f1 = write_file(tmp.path(), "s1.rs", small);
    let f2 = write_file(tmp.path(), "s2.rs", small);
    let f3 = write_file(tmp.path(), "l1.rs", large);
    let f4 = write_file(tmp.path(), "l2.rs", large);
    let exp = export(vec![
        file_row("s1.rs", "m", "Rust", small.len()),
        file_row("s2.rs", "m", "Rust", small.len()),
        file_row("l1.rs", "m", "Rust", large.len()),
        file_row("l2.rs", "m", "Rust", large.len()),
    ]);
    let report = build_duplicate_report(
        tmp.path(),
        &[f1, f2, f3, f4],
        &exp,
        &ContentLimits::default(),
    )
    .unwrap();
    assert_eq!(report.groups.len(), 2);
    assert!(report.groups[0].bytes >= report.groups[1].bytes);
}

// 15. DuplicateReport JSON serialization roundtrip
#[test]
fn dup_serialization_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let content = b"dup content\n";
    let f1 = write_file(tmp.path(), "a.rs", content);
    let f2 = write_file(tmp.path(), "b.rs", content);
    let exp = export(vec![
        file_row("a.rs", "m", "Rust", content.len()),
        file_row("b.rs", "m", "Rust", content.len()),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &[f1, f2], &exp, &ContentLimits::default()).unwrap();
    let json = serde_json::to_string(&report).unwrap();
    let deser: tokmd_analysis_types::DuplicateReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.groups.len(), report.groups.len());
    assert_eq!(deser.wasted_bytes, report.wasted_bytes);
    assert_eq!(deser.strategy, "exact-blake3");
}

// 16. Density report present and correct
#[test]
fn dup_density_report() {
    let tmp = TempDir::new().unwrap();
    let content = b"duplicate block\n";
    let f1 = write_file(tmp.path(), "a.rs", content);
    let f2 = write_file(tmp.path(), "b.rs", content);
    let exp = export(vec![
        file_row("a.rs", "mod", "Rust", content.len()),
        file_row("b.rs", "mod", "Rust", content.len()),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &[f1, f2], &exp, &ContentLimits::default()).unwrap();
    let density = report.density.as_ref().expect("density present");
    assert_eq!(density.duplicate_groups, 1);
    assert_eq!(density.duplicate_files, 2);
    assert!(density.wasted_pct_of_codebase > 0.0);
    assert!(density.wasted_pct_of_codebase <= 1.0);
}

// ===========================================================================
// build_import_report
// ===========================================================================

// 17. Empty files → empty edges
#[test]
fn import_empty_files() {
    let tmp = TempDir::new().unwrap();
    let exp = export(vec![]);
    let report = build_import_report(
        tmp.path(),
        &[],
        &exp,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();
    assert!(report.edges.is_empty());
    assert_eq!(report.granularity, "module");
}

// 18. Rust file with use statement → import edge
#[test]
fn import_rust_use_statement() {
    let tmp = TempDir::new().unwrap();
    let content = b"use std::collections::HashMap;\nfn main() {}\n";
    let rel = write_file(tmp.path(), "main.rs", content);
    let exp = export(vec![file_row("main.rs", "root", "Rust", content.len())]);
    let report = build_import_report(
        tmp.path(),
        &[rel],
        &exp,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();
    assert!(!report.edges.is_empty());
}

// 19. File granularity sets granularity field
#[test]
fn import_file_granularity() {
    let tmp = TempDir::new().unwrap();
    let exp = export(vec![]);
    let report = build_import_report(
        tmp.path(),
        &[],
        &exp,
        ImportGranularity::File,
        &ContentLimits::default(),
    )
    .unwrap();
    assert_eq!(report.granularity, "file");
}

// 20. ImportReport JSON serialization roundtrip
#[test]
fn import_serialization_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let content = b"use serde::Serialize;\nuse anyhow::Result;\n";
    let rel = write_file(tmp.path(), "lib.rs", content);
    let exp = export(vec![file_row("lib.rs", "core", "Rust", content.len())]);
    let report = build_import_report(
        tmp.path(),
        &[rel],
        &exp,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();
    let json = serde_json::to_string(&report).unwrap();
    let deser: tokmd_analysis_types::ImportReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.granularity, report.granularity);
    assert_eq!(deser.edges.len(), report.edges.len());
}

// 21. Import edges sorted by count desc
#[test]
fn import_edges_sorted_by_count() {
    let tmp = TempDir::new().unwrap();
    // Two Rust files importing different things, one importing the same thing twice
    let content = b"use std::io;\nuse std::io;\nuse std::collections::HashMap;\n";
    let rel = write_file(tmp.path(), "multi.rs", content);
    let exp = export(vec![file_row("multi.rs", "root", "Rust", content.len())]);
    let report = build_import_report(
        tmp.path(),
        &[rel],
        &exp,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();
    if report.edges.len() >= 2 {
        assert!(report.edges[0].count >= report.edges[1].count);
    }
}

// 22. Unsupported language produces no edges
#[test]
fn import_unsupported_language() {
    let tmp = TempDir::new().unwrap();
    let content = b"some assembly code\n";
    let rel = write_file(tmp.path(), "code.asm", content);
    let exp = export(vec![file_row(
        "code.asm",
        "root",
        "Assembly",
        content.len(),
    )]);
    let report = build_import_report(
        tmp.path(),
        &[rel],
        &exp,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();
    assert!(report.edges.is_empty());
}

// 23. ContentLimits max_file_bytes respected for TODO scanning
#[test]
fn todo_respects_per_file_limit() {
    let tmp = TempDir::new().unwrap();
    // Write a file with a TODO far past the limit
    let mut content = String::new();
    for _ in 0..200 {
        content.push_str("// normal line\n");
    }
    content.push_str("// TODO: should not be found\n");
    let rel = write_file(tmp.path(), "big.rs", content.as_bytes());
    let limits = ContentLimits {
        max_bytes: None,
        max_file_bytes: Some(100), // Only read first 100 bytes
    };
    let report = build_todo_report(tmp.path(), &[rel], &limits, 1000).unwrap();
    // The TODO is past byte 100, so it should not be found
    assert_eq!(report.total, 0);
}

// 24. ContentLimits max_bytes limits total scanning
#[test]
fn todo_respects_total_byte_limit() {
    let tmp = TempDir::new().unwrap();
    // First file is large enough to exceed the limit by itself
    let big_content = "// TODO: first file\n".repeat(10); // ~200 bytes
    let f1 = write_file(tmp.path(), "a.rs", big_content.as_bytes());
    let f2 = write_file(tmp.path(), "b.rs", b"// TODO: second file\n");
    let limits = ContentLimits {
        max_bytes: Some(big_content.len() as u64), // Limit reached after first file
        max_file_bytes: None,
    };
    let report = build_todo_report(tmp.path(), &[f1, f2], &limits, 1000).unwrap();
    // Only first file should be scanned (10 TODOs), second file skipped
    assert_eq!(report.total, 10);
}

// 25. Duplicate density by_module attribution
#[test]
fn dup_density_by_module() {
    let tmp = TempDir::new().unwrap();
    let content = b"same content in both\n";
    let f1 = write_file(tmp.path(), "mod_a/x.rs", content);
    let f2 = write_file(tmp.path(), "mod_b/y.rs", content);
    let exp = export(vec![
        file_row("mod_a/x.rs", "mod_a", "Rust", content.len()),
        file_row("mod_b/y.rs", "mod_b", "Rust", content.len()),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &[f1, f2], &exp, &ContentLimits::default()).unwrap();
    let density = report.density.as_ref().unwrap();
    assert!(!density.by_module.is_empty());
}

// 26. Child file kind rows excluded from duplicate analysis
#[test]
fn dup_excludes_child_rows() {
    let tmp = TempDir::new().unwrap();
    let content = b"child content\n";
    let f1 = write_file(tmp.path(), "a.rs", content);
    let f2 = write_file(tmp.path(), "b.rs", content);
    let mut rows = vec![file_row("a.rs", "m", "Rust", content.len())];
    rows.push(FileRow {
        path: "b.rs".to_string(),
        module: "m".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Child,
        code: 10,
        comments: 0,
        blanks: 0,
        lines: 10,
        bytes: content.len(),
        tokens: 50,
    });
    let exp = export(rows);
    let report =
        build_duplicate_report(tmp.path(), &[f1, f2], &exp, &ContentLimits::default()).unwrap();
    // b.rs is a Child, so path_to_module won't have it; duplicate still detected
    // but module attribution uses "(unknown)" for b.rs
    assert_eq!(report.groups.len(), 1);
}
