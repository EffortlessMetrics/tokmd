//! W54: Comprehensive enricher coverage for `tokmd-analysis-content`.
//!
//! Targets TODO density, duplicate detection, import graph construction,
//! edge cases, determinism, and serialization contracts.

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
// TODO report tests
// ===========================================================================

// 1. Single TODO tag counted
#[test]
fn todo_single_tag() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "a.rs", b"// TODO: fix this\nfn main() {}\n");
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 1000).unwrap();
    assert_eq!(report.total, 1);
    assert!(report.tags.iter().any(|t| t.tag == "TODO" && t.count == 1));
}

// 2. Multiple different tags in same file
#[test]
fn todo_multiple_tags_same_file() {
    let tmp = TempDir::new().unwrap();
    let content = b"// TODO: a\n// FIXME: b\n// HACK: c\n// XXX: d\n";
    let rel = write_file(tmp.path(), "multi.rs", content);
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 100).unwrap();
    assert_eq!(report.total, 4);
    assert_eq!(report.tags.len(), 4);
}

// 3. Density calculation: total / kLOC
#[test]
fn todo_density_per_kloc() {
    let tmp = TempDir::new().unwrap();
    let content = b"// TODO: one\n// TODO: two\n";
    let rel = write_file(tmp.path(), "a.rs", content);
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 2000).unwrap();
    // density = 2 / (2000/1000) = 1.0
    assert!((report.density_per_kloc - 1.0).abs() < 0.01);
}

// 4. Zero total_code → density 0
#[test]
fn todo_density_zero_code() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "a.rs", b"// TODO: x\n");
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 0).unwrap();
    assert_eq!(report.density_per_kloc, 0.0);
}

// 5. Binary file skipped
#[test]
fn todo_binary_file_skipped() {
    let tmp = TempDir::new().unwrap();
    let mut content = vec![0u8; 200];
    content[10] = 0x00; // null bytes make it non-text
    let rel = write_file(tmp.path(), "binary.bin", &content);
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 100).unwrap();
    assert_eq!(report.total, 0);
}

// 6. max_bytes limit stops scanning
#[test]
fn todo_max_bytes_limit() {
    let tmp = TempDir::new().unwrap();
    let rel1 = write_file(tmp.path(), "a.rs", b"// TODO: first\n");
    let rel2 = write_file(tmp.path(), "b.rs", b"// TODO: second\n");
    let limits = ContentLimits {
        max_bytes: Some(15), // only enough for first file
        max_file_bytes: None,
    };
    let report = build_todo_report(tmp.path(), &[rel1, rel2], &limits, 1000).unwrap();
    // At least one TODO found, but not necessarily both due to byte limit
    assert!(report.total >= 1);
}

// 7. Tags sorted by count, then alphabetically
#[test]
fn todo_tags_sorted_by_count_then_alphabetically() {
    let tmp = TempDir::new().unwrap();
    let content = b"// XXX: a\n// FIXME: b\n// HACK: c\n// TODO: d\n// TODO: d2\n";
    let rel = write_file(tmp.path(), "sorted.rs", content);
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 100).unwrap();
    let names: Vec<&str> = report.tags.iter().map(|t| t.tag.as_str()).collect();
    assert_eq!(names, vec!["TODO", "FIXME", "HACK", "XXX"]);
}

// ===========================================================================
// Duplicate report tests
// ===========================================================================

// 8. No duplicates when all files are unique
#[test]
fn dup_no_duplicates() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.rs", b"fn a() {}"),
        write_file(tmp.path(), "b.rs", b"fn b() {}"),
    ];
    let exp = export(vec![
        file_row("a.rs", "src", "Rust", 9),
        file_row("b.rs", "src", "Rust", 9),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &files, &exp, &ContentLimits::default()).unwrap();
    assert_eq!(report.wasted_bytes, 0);
    assert!(report.groups.is_empty());
}

// 9. Exact duplicates detected
#[test]
fn dup_exact_duplicates_detected() {
    let tmp = TempDir::new().unwrap();
    let content = b"duplicate content here";
    let files = vec![
        write_file(tmp.path(), "a.rs", content),
        write_file(tmp.path(), "b.rs", content),
    ];
    let exp = export(vec![
        file_row("a.rs", "src", "Rust", content.len()),
        file_row("b.rs", "src", "Rust", content.len()),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &files, &exp, &ContentLimits::default()).unwrap();
    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.groups[0].files.len(), 2);
    assert_eq!(report.wasted_bytes, content.len() as u64);
}

// 10. Empty files not flagged as duplicates
#[test]
fn dup_empty_files_not_flagged() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "empty1.rs", b""),
        write_file(tmp.path(), "empty2.rs", b""),
    ];
    let exp = export(vec![
        file_row("empty1.rs", "src", "Rust", 0),
        file_row("empty2.rs", "src", "Rust", 0),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &files, &exp, &ContentLimits::default()).unwrap();
    assert!(report.groups.is_empty());
}

// 11. Duplicate strategy is exact-blake3
#[test]
fn dup_strategy_blake3() {
    let tmp = TempDir::new().unwrap();
    let report =
        build_duplicate_report(tmp.path(), &[], &export(vec![]), &ContentLimits::default())
            .unwrap();
    assert_eq!(report.strategy, "exact-blake3");
}

// 12. Duplicate groups sorted by bytes descending
#[test]
fn dup_groups_sorted_by_bytes_desc() {
    let tmp = TempDir::new().unwrap();
    let small = b"sm";
    let large = b"large content here!!!";
    let files = vec![
        write_file(tmp.path(), "s1.rs", small),
        write_file(tmp.path(), "s2.rs", small),
        write_file(tmp.path(), "l1.rs", large),
        write_file(tmp.path(), "l2.rs", large),
    ];
    let exp = export(vec![
        file_row("s1.rs", "src", "Rust", small.len()),
        file_row("s2.rs", "src", "Rust", small.len()),
        file_row("l1.rs", "src", "Rust", large.len()),
        file_row("l2.rs", "src", "Rust", large.len()),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &files, &exp, &ContentLimits::default()).unwrap();
    assert_eq!(report.groups.len(), 2);
    assert!(report.groups[0].bytes >= report.groups[1].bytes);
}

// 13. Density report populated
#[test]
fn dup_density_report_present() {
    let tmp = TempDir::new().unwrap();
    let content = b"dup content";
    let files = vec![
        write_file(tmp.path(), "a.rs", content),
        write_file(tmp.path(), "b.rs", content),
    ];
    let exp = export(vec![
        file_row("a.rs", "src", "Rust", content.len()),
        file_row("b.rs", "src", "Rust", content.len()),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &files, &exp, &ContentLimits::default()).unwrap();
    let density = report.density.as_ref().unwrap();
    assert_eq!(density.duplicate_groups, 1);
    assert!(density.duplicate_files >= 2);
}

// 14. max_file_bytes limit excludes large files
#[test]
fn dup_max_file_bytes_limit() {
    let tmp = TempDir::new().unwrap();
    let content = vec![0u8; 200];
    let files = vec![
        write_file(tmp.path(), "big1.rs", &content),
        write_file(tmp.path(), "big2.rs", &content),
    ];
    let exp = export(vec![
        file_row("big1.rs", "src", "Rust", 200),
        file_row("big2.rs", "src", "Rust", 200),
    ]);
    let limits = ContentLimits {
        max_bytes: None,
        max_file_bytes: Some(100),
    };
    let report = build_duplicate_report(tmp.path(), &files, &exp, &limits).unwrap();
    assert!(report.groups.is_empty());
}

// ===========================================================================
// Import report tests
// ===========================================================================

// 15. Empty files list → empty edges
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

// 16. File-level granularity sets correct string
#[test]
fn import_file_granularity_string() {
    let tmp = TempDir::new().unwrap();
    let report = build_import_report(
        tmp.path(),
        &[],
        &export(vec![]),
        ImportGranularity::File,
        &ContentLimits::default(),
    )
    .unwrap();
    assert_eq!(report.granularity, "file");
}

// 17. Import from Rust file with `use` statement
#[test]
fn import_rust_use_detected() {
    let tmp = TempDir::new().unwrap();
    let content = b"use std::collections::HashMap;\nfn main() {}\n";
    let rel = write_file(tmp.path(), "main.rs", content);
    let exp = export(vec![file_row("main.rs", "src", "Rust", content.len())]);
    let report = build_import_report(
        tmp.path(),
        &[rel],
        &exp,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();
    // Should detect at least one import edge
    assert!(!report.edges.is_empty());
}

// 18. Import edges sorted by count descending
#[test]
fn import_edges_sorted_desc() {
    let tmp = TempDir::new().unwrap();
    let content = b"\
use std::io;\n\
use std::io::Read;\n\
use serde::Serialize;\n\
";
    let rel = write_file(tmp.path(), "lib.rs", content);
    let exp = export(vec![file_row("lib.rs", "src", "Rust", content.len())]);
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

// 19. Unsupported language produces no edges
#[test]
fn import_unsupported_language_no_edges() {
    let tmp = TempDir::new().unwrap();
    let content = b"# This is a Makefile\nall: build\n";
    let rel = write_file(tmp.path(), "Makefile", content);
    let exp = export(vec![file_row("Makefile", ".", "Makefile", content.len())]);
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

// 20. Child rows filtered out of import scanning
#[test]
fn import_child_rows_excluded() {
    let tmp = TempDir::new().unwrap();
    let content = b"use std::io;\n";
    let rel = write_file(tmp.path(), "embedded.rs", content);
    let exp = ExportData {
        rows: vec![FileRow {
            path: "embedded.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes: content.len(),
            tokens: 5,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
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

// 21. Duplicate report deterministic
#[test]
fn dup_deterministic_ordering() {
    let tmp = TempDir::new().unwrap();
    let content = b"some data";
    let files = vec![
        write_file(tmp.path(), "x.rs", content),
        write_file(tmp.path(), "y.rs", content),
    ];
    let exp = export(vec![
        file_row("x.rs", "src", "Rust", content.len()),
        file_row("y.rs", "src", "Rust", content.len()),
    ]);
    let r1 = build_duplicate_report(tmp.path(), &files, &exp, &ContentLimits::default()).unwrap();
    let r2 = build_duplicate_report(tmp.path(), &files, &exp, &ContentLimits::default()).unwrap();
    assert_eq!(r1.groups.len(), r2.groups.len());
    assert_eq!(r1.wasted_bytes, r2.wasted_bytes);
    assert_eq!(r1.groups[0].hash, r2.groups[0].hash);
}

// 22. TODO report with many files
#[test]
fn todo_many_files() {
    let tmp = TempDir::new().unwrap();
    let files: Vec<PathBuf> = (0..20)
        .map(|i| {
            write_file(
                tmp.path(),
                &format!("f{i}.rs"),
                format!("// TODO: item {i}\n").as_bytes(),
            )
        })
        .collect();
    let report = build_todo_report(tmp.path(), &files, &ContentLimits::default(), 5000).unwrap();
    assert_eq!(report.total, 20);
}
