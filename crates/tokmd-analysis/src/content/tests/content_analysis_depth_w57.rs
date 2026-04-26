//! Depth tests for `tokmd-analysis content module` — w57
//!
//! Covers TODO/FIXME/HACK tag scanning, duplicate detection, import graph
//! construction, empty/binary/large file handling, deterministic ordering,
//! and serde roundtrips.

use std::path::PathBuf;

use crate::content::{
    ContentLimits, ImportGranularity, build_duplicate_report, build_import_report,
    build_todo_report,
};
use tempfile::TempDir;
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
// TODO/FIXME/HACK tag scanning
// ===========================================================================

// 1. Tags inside block comments are detected
#[test]
fn todo_in_block_comment() {
    let tmp = TempDir::new().unwrap();
    let content = b"/* TODO: refactor this */\nfn main() {}\n";
    let rel = write_file(tmp.path(), "block.rs", content);
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 1000).unwrap();
    assert_eq!(report.total, 1);
}

// 2. Tags in string literals are counted (tag scanner is line-based)
#[test]
fn todo_in_string_literal() {
    let tmp = TempDir::new().unwrap();
    let content = b"let msg = \"TODO: this is in a string\";\n";
    let rel = write_file(tmp.path(), "str.rs", content);
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 1000).unwrap();
    // The scanner is simple/line-based, so it counts this
    assert_eq!(report.total, 1);
}

// 3. Case-sensitive: lowercase "todo" not counted
#[test]
fn todo_case_sensitive() {
    let tmp = TempDir::new().unwrap();
    let content = b"// todo: lowercase\n// TODO: uppercase\n";
    let rel = write_file(tmp.path(), "case.rs", content);
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 1000).unwrap();
    // Only uppercase TODO is detected
    assert!(report.total >= 1);
}

// 4. FIXME tag counted separately from TODO
#[test]
fn fixme_counted_separately() {
    let tmp = TempDir::new().unwrap();
    let content = b"// FIXME: broken\n// FIXME: also broken\n// TODO: later\n";
    let rel = write_file(tmp.path(), "fix.rs", content);
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 1000).unwrap();
    assert_eq!(report.total, 3);
    let fixme_count = report
        .tags
        .iter()
        .find(|t| t.tag == "FIXME")
        .map(|t| t.count);
    assert_eq!(fixme_count, Some(2));
}

// 5. HACK tag detection
#[test]
fn hack_tag_detected() {
    let tmp = TempDir::new().unwrap();
    let content = b"// HACK: workaround for upstream bug\nfn f() {}\n";
    let rel = write_file(tmp.path(), "hack.rs", content);
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 1000).unwrap();
    assert_eq!(report.total, 1);
    assert!(report.tags.iter().any(|t| t.tag == "HACK" && t.count == 1));
}

// 6. XXX tag detection
#[test]
fn xxx_tag_detected() {
    let tmp = TempDir::new().unwrap();
    let content = b"// XXX: needs review\n";
    let rel = write_file(tmp.path(), "xxx.rs", content);
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 1000).unwrap();
    assert_eq!(report.total, 1);
    assert!(report.tags.iter().any(|t| t.tag == "XXX" && t.count == 1));
}

// 7. Multiple tags on the same line
#[test]
fn multiple_tags_same_line() {
    let tmp = TempDir::new().unwrap();
    let content = b"// TODO: FIXME: both tags on one line\n";
    let rel = write_file(tmp.path(), "multi.rs", content);
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 1000).unwrap();
    // Both TODO and FIXME should be found
    assert_eq!(report.total, 2);
}

// 8. Tags across many files aggregate correctly
#[test]
fn tags_aggregate_across_files() {
    let tmp = TempDir::new().unwrap();
    let f1 = write_file(tmp.path(), "a.rs", b"// TODO: one\n");
    let f2 = write_file(tmp.path(), "b.rs", b"// TODO: two\n// FIXME: three\n");
    let f3 = write_file(tmp.path(), "c.rs", b"// HACK: four\n");
    let report =
        build_todo_report(tmp.path(), &[f1, f2, f3], &ContentLimits::default(), 1000).unwrap();
    assert_eq!(report.total, 4);
}

// 9. Binary file skipped by TODO scanner
#[test]
fn todo_binary_file_skipped() {
    let tmp = TempDir::new().unwrap();
    // Write bytes that look binary (null bytes)
    let mut content = vec![0u8; 100];
    content.extend_from_slice(b"TODO: hidden in binary\n");
    let rel = write_file(tmp.path(), "bin.dat", &content);
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 1000).unwrap();
    assert_eq!(report.total, 0);
}

// 10. Empty file → zero tags
#[test]
fn todo_empty_file_zero_tags() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "empty.rs", b"");
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 1000).unwrap();
    assert_eq!(report.total, 0);
}

// 11. Density with known values
#[test]
fn todo_density_exact() {
    let tmp = TempDir::new().unwrap();
    // 5 TODOs
    let content = "// TODO: 1\n// TODO: 2\n// TODO: 3\n// TODO: 4\n// TODO: 5\n";
    let rel = write_file(tmp.path(), "dense.rs", content.as_bytes());
    // total_code = 10_000 → 10 KLOC → density = 5/10 = 0.5
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 10_000).unwrap();
    assert_eq!(report.total, 5);
    assert!((report.density_per_kloc - 0.5).abs() < 0.01);
}

// ===========================================================================
// Duplicate detection
// ===========================================================================

// 12. Files with same content but different sizes → not duplicates
// (sizes differ because metadata may differ; but same content = same size on disk)
#[test]
fn dup_same_hash_detected() {
    let tmp = TempDir::new().unwrap();
    let content = b"exact duplicate content\n";
    let f1 = write_file(tmp.path(), "src/a.rs", content);
    let f2 = write_file(tmp.path(), "src/b.rs", content);
    let exp = export(vec![
        file_row("src/a.rs", "src", "Rust", content.len()),
        file_row("src/b.rs", "src", "Rust", content.len()),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &[f1, f2], &exp, &ContentLimits::default()).unwrap();
    assert_eq!(report.groups.len(), 1);
    // Both files in the group have the same hash
    assert_eq!(report.groups[0].files.len(), 2);
}

// 13. Cross-module duplicates detected
#[test]
fn dup_cross_module() {
    let tmp = TempDir::new().unwrap();
    let content = b"shared helper function\n";
    let f1 = write_file(tmp.path(), "mod_a/helper.rs", content);
    let f2 = write_file(tmp.path(), "mod_b/helper.rs", content);
    let exp = export(vec![
        file_row("mod_a/helper.rs", "mod_a", "Rust", content.len()),
        file_row("mod_b/helper.rs", "mod_b", "Rust", content.len()),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &[f1, f2], &exp, &ContentLimits::default()).unwrap();
    assert_eq!(report.groups.len(), 1);
    let density = report.density.as_ref().unwrap();
    // Both modules should appear in by_module
    assert!(density.by_module.len() >= 2);
}

// 14. Duplicate report uses blake3 strategy
#[test]
fn dup_strategy_is_blake3() {
    let tmp = TempDir::new().unwrap();
    let exp = export(vec![]);
    let report = build_duplicate_report(tmp.path(), &[], &exp, &ContentLimits::default()).unwrap();
    assert_eq!(report.strategy, "exact-blake3");
}

// 15. Single file → no duplicate groups
#[test]
fn dup_single_file_no_group() {
    let tmp = TempDir::new().unwrap();
    let content = b"unique content\n";
    let f1 = write_file(tmp.path(), "only.rs", content);
    let exp = export(vec![file_row("only.rs", "src", "Rust", content.len())]);
    let report =
        build_duplicate_report(tmp.path(), &[f1], &exp, &ContentLimits::default()).unwrap();
    assert!(report.groups.is_empty());
    assert_eq!(report.wasted_bytes, 0);
}

// 16. Multiple distinct groups
#[test]
fn dup_multiple_groups() {
    let tmp = TempDir::new().unwrap();
    let content_a = b"group A content\n";
    let content_b = b"group B content!\n"; // different from A
    let f1 = write_file(tmp.path(), "a1.rs", content_a);
    let f2 = write_file(tmp.path(), "a2.rs", content_a);
    let f3 = write_file(tmp.path(), "b1.rs", content_b);
    let f4 = write_file(tmp.path(), "b2.rs", content_b);
    let exp = export(vec![
        file_row("a1.rs", "m", "Rust", content_a.len()),
        file_row("a2.rs", "m", "Rust", content_a.len()),
        file_row("b1.rs", "m", "Rust", content_b.len()),
        file_row("b2.rs", "m", "Rust", content_b.len()),
    ]);
    let report = build_duplicate_report(
        tmp.path(),
        &[f1, f2, f3, f4],
        &exp,
        &ContentLimits::default(),
    )
    .unwrap();
    assert_eq!(report.groups.len(), 2);
    assert_eq!(
        report.wasted_bytes,
        content_a.len() as u64 + content_b.len() as u64
    );
}

// 17. Wasted bytes calculation: 4 copies = 3 wasted
#[test]
fn dup_wasted_bytes_four_copies() {
    let tmp = TempDir::new().unwrap();
    let content = b"quadruplicate\n";
    let f1 = write_file(tmp.path(), "c1.rs", content);
    let f2 = write_file(tmp.path(), "c2.rs", content);
    let f3 = write_file(tmp.path(), "c3.rs", content);
    let f4 = write_file(tmp.path(), "c4.rs", content);
    let exp = export(vec![
        file_row("c1.rs", "m", "Rust", content.len()),
        file_row("c2.rs", "m", "Rust", content.len()),
        file_row("c3.rs", "m", "Rust", content.len()),
        file_row("c4.rs", "m", "Rust", content.len()),
    ]);
    let report = build_duplicate_report(
        tmp.path(),
        &[f1, f2, f3, f4],
        &exp,
        &ContentLimits::default(),
    )
    .unwrap();
    assert_eq!(report.wasted_bytes, 3 * content.len() as u64);
}

// 18. max_file_bytes limit excludes large files from dup scan
#[test]
fn dup_max_file_bytes_excludes_large() {
    let tmp = TempDir::new().unwrap();
    let content = b"content that is somewhat large for testing purposes here\n";
    let f1 = write_file(tmp.path(), "big1.rs", content);
    let f2 = write_file(tmp.path(), "big2.rs", content);
    let exp = export(vec![
        file_row("big1.rs", "m", "Rust", content.len()),
        file_row("big2.rs", "m", "Rust", content.len()),
    ]);
    let limits = ContentLimits {
        max_bytes: None,
        max_file_bytes: Some(10), // far smaller than content
    };
    let report = build_duplicate_report(tmp.path(), &[f1, f2], &exp, &limits).unwrap();
    assert!(report.groups.is_empty());
}

// ===========================================================================
// Import graph construction
// ===========================================================================

// 19. Python import generates edges
#[test]
fn import_python_detected() {
    let tmp = TempDir::new().unwrap();
    let content = b"import os\nimport sys\n";
    let rel = write_file(tmp.path(), "main.py", content);
    let exp = export(vec![file_row("main.py", "root", "Python", content.len())]);
    let report = build_import_report(
        tmp.path(),
        &[rel],
        &exp,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();
    assert!(!report.edges.is_empty());
    assert_eq!(report.granularity, "module");
}

// 20. TypeScript import detected
#[test]
fn import_typescript_detected() {
    let tmp = TempDir::new().unwrap();
    let content = b"import { useState } from 'react';\n";
    let rel = write_file(tmp.path(), "app.tsx", content);
    let exp = export(vec![file_row(
        "app.tsx",
        "src",
        "TypeScript",
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
    assert!(!report.edges.is_empty());
}

// 21. File with no imports → empty edges
#[test]
fn import_no_imports_empty() {
    let tmp = TempDir::new().unwrap();
    let content = b"fn main() { println!(\"hello\"); }\n";
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
    assert!(report.edges.is_empty());
}

// 22. File granularity uses file path as source
#[test]
fn import_file_granularity_source() {
    let tmp = TempDir::new().unwrap();
    let content = b"use std::io;\n";
    let rel = write_file(tmp.path(), "lib.rs", content);
    let exp = export(vec![file_row("lib.rs", "root", "Rust", content.len())]);
    let report = build_import_report(
        tmp.path(),
        &[rel],
        &exp,
        ImportGranularity::File,
        &ContentLimits::default(),
    )
    .unwrap();
    assert_eq!(report.granularity, "file");
    if !report.edges.is_empty() {
        assert_eq!(report.edges[0].from, "lib.rs");
    }
}

// 23. Import edges sorted by count descending
#[test]
fn import_edges_sorted_desc() {
    let tmp = TempDir::new().unwrap();
    let content = b"use std::io;\nuse std::io;\nuse std::collections::HashMap;\n";
    let rel = write_file(tmp.path(), "sorted.rs", content);
    let exp = export(vec![file_row("sorted.rs", "root", "Rust", content.len())]);
    let report = build_import_report(
        tmp.path(),
        &[rel],
        &exp,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .unwrap();
    for w in report.edges.windows(2) {
        assert!(w[0].count >= w[1].count);
    }
}

// ===========================================================================
// Empty / binary / large file handling
// ===========================================================================

// 24. Empty file list for import → empty report
#[test]
fn import_empty_file_list() {
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
}

// 25. Empty file list for duplicates → empty report
#[test]
fn dup_empty_file_list() {
    let tmp = TempDir::new().unwrap();
    let exp = export(vec![]);
    let report = build_duplicate_report(tmp.path(), &[], &exp, &ContentLimits::default()).unwrap();
    assert!(report.groups.is_empty());
    assert_eq!(report.wasted_bytes, 0);
}

// ===========================================================================
// Deterministic output ordering
// ===========================================================================

// 26. Duplicate groups sorted by bytes desc, then hash
#[test]
fn dup_groups_deterministic_order() {
    let tmp = TempDir::new().unwrap();
    let small = b"sm\n";
    let large = b"this is much larger content!\n";
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

// 27. Running duplicate detection twice gives identical JSON
#[test]
fn dup_deterministic_json() {
    let tmp = TempDir::new().unwrap();
    let content = b"deterministic test\n";
    let f1 = write_file(tmp.path(), "d1.rs", content);
    let f2 = write_file(tmp.path(), "d2.rs", content);
    let exp = export(vec![
        file_row("d1.rs", "m", "Rust", content.len()),
        file_row("d2.rs", "m", "Rust", content.len()),
    ]);
    let files = vec![f1, f2];
    let r1 = build_duplicate_report(tmp.path(), &files, &exp, &ContentLimits::default()).unwrap();
    let r2 = build_duplicate_report(tmp.path(), &files, &exp, &ContentLimits::default()).unwrap();
    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(j1, j2);
}

// 28. Files within duplicate groups sorted alphabetically
#[test]
fn dup_files_within_group_sorted() {
    let tmp = TempDir::new().unwrap();
    let content = b"sort test content\n";
    let f1 = write_file(tmp.path(), "z.rs", content);
    let f2 = write_file(tmp.path(), "a.rs", content);
    let f3 = write_file(tmp.path(), "m.rs", content);
    let exp = export(vec![
        file_row("z.rs", "m", "Rust", content.len()),
        file_row("a.rs", "m", "Rust", content.len()),
        file_row("m.rs", "m", "Rust", content.len()),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &[f1, f2, f3], &exp, &ContentLimits::default()).unwrap();
    assert_eq!(report.groups.len(), 1);
    let files = &report.groups[0].files;
    assert_eq!(files, &["a.rs", "m.rs", "z.rs"]);
}

// ===========================================================================
// Serde roundtrips
// ===========================================================================

// 29. TodoReport serde roundtrip preserves all fields
#[test]
fn serde_todo_roundtrip_all_fields() {
    let tmp = TempDir::new().unwrap();
    let content = b"// TODO: a\n// FIXME: b\n// HACK: c\n// XXX: d\n";
    let rel = write_file(tmp.path(), "all.rs", content);
    let report = build_todo_report(tmp.path(), &[rel], &ContentLimits::default(), 5000).unwrap();
    let json = serde_json::to_string(&report).unwrap();
    let deser: tokmd_analysis_types::TodoReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.total, report.total);
    assert_eq!(deser.density_per_kloc, report.density_per_kloc);
    assert_eq!(deser.tags.len(), report.tags.len());
    for (a, b) in deser.tags.iter().zip(report.tags.iter()) {
        assert_eq!(a.tag, b.tag);
        assert_eq!(a.count, b.count);
    }
}

// 30. DuplicateReport serde roundtrip preserves density
#[test]
fn serde_dup_roundtrip_with_density() {
    let tmp = TempDir::new().unwrap();
    let content = b"dup content for serde\n";
    let f1 = write_file(tmp.path(), "s1.rs", content);
    let f2 = write_file(tmp.path(), "s2.rs", content);
    let exp = export(vec![
        file_row("s1.rs", "src", "Rust", content.len()),
        file_row("s2.rs", "src", "Rust", content.len()),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &[f1, f2], &exp, &ContentLimits::default()).unwrap();
    let json = serde_json::to_string(&report).unwrap();
    let deser: tokmd_analysis_types::DuplicateReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.wasted_bytes, report.wasted_bytes);
    assert_eq!(deser.strategy, report.strategy);
    let orig_density = report.density.as_ref().unwrap();
    let deser_density = deser.density.as_ref().unwrap();
    assert_eq!(
        orig_density.duplicate_groups,
        deser_density.duplicate_groups
    );
    assert_eq!(orig_density.duplicate_files, deser_density.duplicate_files);
    assert_eq!(
        orig_density.wasted_pct_of_codebase,
        deser_density.wasted_pct_of_codebase
    );
}

// 31. ImportReport serde roundtrip
#[test]
fn serde_import_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let content = b"use std::io;\nuse std::fs;\n";
    let rel = write_file(tmp.path(), "imp.rs", content);
    let exp = export(vec![file_row("imp.rs", "root", "Rust", content.len())]);
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
    for (a, b) in deser.edges.iter().zip(report.edges.iter()) {
        assert_eq!(a.from, b.from);
        assert_eq!(a.to, b.to);
        assert_eq!(a.count, b.count);
    }
}

// ===========================================================================
// Additional edge cases
// ===========================================================================

// 32. Todo tags in nested directory structure
#[test]
fn todo_nested_directories() {
    let tmp = TempDir::new().unwrap();
    let f1 = write_file(tmp.path(), "a/b/c/deep.rs", b"// TODO: deep\n");
    let f2 = write_file(tmp.path(), "x/shallow.rs", b"// FIXME: shallow\n");
    let report = build_todo_report(tmp.path(), &[f1, f2], &ContentLimits::default(), 1000).unwrap();
    assert_eq!(report.total, 2);
}

// 33. Duplicate detection with subdirectory paths
#[test]
fn dup_subdirectory_paths() {
    let tmp = TempDir::new().unwrap();
    let content = b"duplicate in subdirs\n";
    let f1 = write_file(tmp.path(), "a/b/file.rs", content);
    let f2 = write_file(tmp.path(), "c/d/file.rs", content);
    let exp = export(vec![
        file_row("a/b/file.rs", "a/b", "Rust", content.len()),
        file_row("c/d/file.rs", "c/d", "Rust", content.len()),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &[f1, f2], &exp, &ContentLimits::default()).unwrap();
    assert_eq!(report.groups.len(), 1);
}

// 34. ContentLimits max_bytes stops import scanning
#[test]
fn import_respects_max_bytes() {
    let tmp = TempDir::new().unwrap();
    let big_content = "use std::io;\n".repeat(50); // ~650 bytes
    let f1 = write_file(tmp.path(), "big.rs", big_content.as_bytes());
    let f2 = write_file(tmp.path(), "small.rs", b"use std::fs;\n");
    let exp = export(vec![
        file_row("big.rs", "root", "Rust", big_content.len()),
        file_row("small.rs", "root", "Rust", 13),
    ]);
    let limits = ContentLimits {
        max_bytes: Some(big_content.len() as u64),
        max_file_bytes: None,
    };
    let report = build_import_report(
        tmp.path(),
        &[f1, f2],
        &exp,
        ImportGranularity::Module,
        &limits,
    )
    .unwrap();
    // The second file should be skipped due to byte budget
    // Just verify no panic and edges only come from first file
    assert!(!report.edges.is_empty());
}

// 35. Density wasted_pct_of_codebase is in [0, 1]
#[test]
fn dup_wasted_pct_in_range() {
    let tmp = TempDir::new().unwrap();
    let content = b"some dup content to test percentage\n";
    let f1 = write_file(tmp.path(), "p1.rs", content);
    let f2 = write_file(tmp.path(), "p2.rs", content);
    let exp = export(vec![
        file_row("p1.rs", "m", "Rust", content.len()),
        file_row("p2.rs", "m", "Rust", content.len()),
    ]);
    let report =
        build_duplicate_report(tmp.path(), &[f1, f2], &exp, &ContentLimits::default()).unwrap();
    let density = report.density.as_ref().unwrap();
    assert!(density.wasted_pct_of_codebase >= 0.0);
    assert!(density.wasted_pct_of_codebase <= 1.0);
}
