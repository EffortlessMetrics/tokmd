//! W68 deep tests for `tokmd-analysis-content`.
//!
//! Exercises TODO density scanning, duplicate detection, import scanning,
//! content limits, edge cases, and determinism.

use std::path::PathBuf;

use tempfile::TempDir;
use tokmd_analysis_content::{ContentLimits, build_duplicate_report, build_todo_report};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

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

fn write_file(dir: &std::path::Path, rel: &str, content: &[u8]) -> PathBuf {
    let full = dir.join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&full, content).unwrap();
    PathBuf::from(rel)
}

// ── TODO density scanning ───────────────────────────────────────

mod todo_w68 {
    use super::*;

    #[test]
    fn fixme_counted_separately_from_todo() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "a.rs", b"// TODO: a\n// FIXME: b\n");
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 1000).unwrap();
        assert_eq!(r.total, 2);
        let todo_count = r.tags.iter().find(|t| t.tag == "TODO").map(|t| t.count).unwrap_or(0);
        let fixme_count = r.tags.iter().find(|t| t.tag == "FIXME").map(|t| t.count).unwrap_or(0);
        assert_eq!(todo_count, 1);
        assert_eq!(fixme_count, 1);
    }

    #[test]
    fn hack_and_xxx_detected() {
        let tmp = TempDir::new().unwrap();
        let content = "// HACK: workaround\n// XXX: danger\nfn main() {}\n";
        let f = write_file(tmp.path(), "a.rs", content.as_bytes());
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 1000).unwrap();
        let hack = r.tags.iter().find(|t| t.tag == "HACK").map(|t| t.count).unwrap_or(0);
        let xxx = r.tags.iter().find(|t| t.tag == "XXX").map(|t| t.count).unwrap_or(0);
        assert_eq!(hack, 1);
        assert_eq!(xxx, 1);
    }

    #[test]
    fn multiple_todos_in_one_line() {
        let tmp = TempDir::new().unwrap();
        let content = "// TODO: first TODO: second\n";
        let f = write_file(tmp.path(), "a.rs", content.as_bytes());
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 1000).unwrap();
        let todo_count = r.tags.iter().find(|t| t.tag == "TODO").map(|t| t.count).unwrap_or(0);
        assert!(todo_count >= 2, "expected at least 2 TODOs, got {todo_count}");
    }

    #[test]
    fn density_scales_with_kloc() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "a.rs", b"// TODO: a\n// TODO: b\n// TODO: c\n");
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 3000).unwrap();
        assert_eq!(r.total, 3);
        // 3 todos / 3 kloc = 1.0
        assert!((r.density_per_kloc - 1.0).abs() < 0.01);
    }

    #[test]
    fn empty_files_produce_no_tags() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "empty.rs", b"");
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 1000).unwrap();
        assert_eq!(r.total, 0);
    }

    #[test]
    fn todo_across_many_files() {
        let tmp = TempDir::new().unwrap();
        let mut files = Vec::new();
        for i in 0..10 {
            files.push(write_file(
                tmp.path(),
                &format!("f{i}.rs"),
                format!("// TODO: item {i}\n").as_bytes(),
            ));
        }
        let r = build_todo_report(tmp.path(), &files, &default_limits(), 5000).unwrap();
        assert_eq!(r.total, 10);
    }

    #[test]
    fn case_insensitive_tags_matched() {
        let tmp = TempDir::new().unwrap();
        // Tags are matched case-insensitively
        let f = write_file(tmp.path(), "a.rs", b"// todo: lowercase\n// Todo: mixed\n");
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 1000).unwrap();
        assert_eq!(r.total, 2, "case-insensitive matching should find both");
    }
}

// ── Content limits ──────────────────────────────────────────────

mod limits_w68 {
    use super::*;

    #[test]
    fn max_file_bytes_truncates_large_file() {
        let tmp = TempDir::new().unwrap();
        // Large file: TODO is way past the limit
        let mut content = "fn main() {}\n".repeat(1000);
        content.push_str("// TODO: hidden\n");
        let f = write_file(tmp.path(), "big.rs", content.as_bytes());
        let limits = ContentLimits {
            max_bytes: None,
            max_file_bytes: Some(100),
        };
        let r = build_todo_report(tmp.path(), &[f], &limits, 1000).unwrap();
        assert_eq!(r.total, 0, "TODO past file byte limit should not be found");
    }

    #[test]
    fn max_bytes_zero_scans_nothing() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "a.rs", b"// TODO: x\n");
        let limits = ContentLimits {
            max_bytes: Some(0),
            max_file_bytes: None,
        };
        let r = build_todo_report(tmp.path(), &[f], &limits, 1000).unwrap();
        assert_eq!(r.total, 0);
    }
}

// ── Duplicate detection ─────────────────────────────────────────

mod duplicate_w68 {
    use super::*;

    #[test]
    fn identical_files_detected_as_duplicates() {
        let tmp = TempDir::new().unwrap();
        let content = b"fn shared() { println!(\"hello\"); }\n";
        let f1 = write_file(tmp.path(), "a.rs", content);
        let f2 = write_file(tmp.path(), "b.rs", content);
        let e = make_export(vec![
            make_row("a.rs", "src", "Rust", content.len()),
            make_row("b.rs", "src", "Rust", content.len()),
        ]);
        let r = build_duplicate_report(tmp.path(), &[f1, f2], &e, &default_limits()).unwrap();
        assert_eq!(r.groups.len(), 1);
        assert_eq!(r.groups[0].files.len(), 2);
        assert_eq!(r.wasted_bytes, content.len() as u64);
    }

    #[test]
    fn no_duplicates_when_content_differs() {
        let tmp = TempDir::new().unwrap();
        let f1 = write_file(tmp.path(), "a.rs", b"fn a() { 1 }");
        let f2 = write_file(tmp.path(), "b.rs", b"fn b() { 2 }");
        let e = make_export(vec![
            make_row("a.rs", "src", "Rust", 12),
            make_row("b.rs", "src", "Rust", 12),
        ]);
        let r = build_duplicate_report(tmp.path(), &[f1, f2], &e, &default_limits()).unwrap();
        assert!(r.groups.is_empty());
        assert_eq!(r.wasted_bytes, 0);
    }

    #[test]
    fn three_identical_files_one_group() {
        let tmp = TempDir::new().unwrap();
        let content = b"duplicate content here\n";
        let f1 = write_file(tmp.path(), "a.rs", content);
        let f2 = write_file(tmp.path(), "b.rs", content);
        let f3 = write_file(tmp.path(), "c.rs", content);
        let e = make_export(vec![
            make_row("a.rs", "src", "Rust", content.len()),
            make_row("b.rs", "src", "Rust", content.len()),
            make_row("c.rs", "src", "Rust", content.len()),
        ]);
        let r = build_duplicate_report(tmp.path(), &[f1, f2, f3], &e, &default_limits()).unwrap();
        assert_eq!(r.groups.len(), 1);
        assert_eq!(r.groups[0].files.len(), 3);
        // Wasted = (3-1) * size
        assert_eq!(r.wasted_bytes, 2 * content.len() as u64);
    }

    #[test]
    fn empty_files_not_duplicates() {
        let tmp = TempDir::new().unwrap();
        let f1 = write_file(tmp.path(), "a.rs", b"");
        let f2 = write_file(tmp.path(), "b.rs", b"");
        let e = make_export(vec![
            make_row("a.rs", "src", "Rust", 0),
            make_row("b.rs", "src", "Rust", 0),
        ]);
        let r = build_duplicate_report(tmp.path(), &[f1, f2], &e, &default_limits()).unwrap();
        assert_eq!(r.wasted_bytes, 0);
    }

    #[test]
    fn duplicate_groups_sorted_by_bytes_desc() {
        let tmp = TempDir::new().unwrap();
        let small = b"sm";
        let big = b"a much bigger duplicated file content here!!!";
        let f1 = write_file(tmp.path(), "s1.rs", small);
        let f2 = write_file(tmp.path(), "s2.rs", small);
        let f3 = write_file(tmp.path(), "b1.rs", big);
        let f4 = write_file(tmp.path(), "b2.rs", big);
        let e = make_export(vec![
            make_row("s1.rs", "src", "Rust", small.len()),
            make_row("s2.rs", "src", "Rust", small.len()),
            make_row("b1.rs", "src", "Rust", big.len()),
            make_row("b2.rs", "src", "Rust", big.len()),
        ]);
        let r = build_duplicate_report(tmp.path(), &[f1, f2, f3, f4], &e, &default_limits()).unwrap();
        assert_eq!(r.groups.len(), 2);
        assert!(r.groups[0].bytes >= r.groups[1].bytes);
    }

    #[test]
    fn strategy_is_exact_blake3() {
        let tmp = TempDir::new().unwrap();
        let r = build_duplicate_report(tmp.path(), &[], &make_export(vec![]), &default_limits()).unwrap();
        assert_eq!(r.strategy, "exact-blake3");
    }

    #[test]
    fn density_report_present() {
        let tmp = TempDir::new().unwrap();
        let content = b"shared content\n";
        let f1 = write_file(tmp.path(), "a.rs", content);
        let f2 = write_file(tmp.path(), "b.rs", content);
        let e = make_export(vec![
            make_row("a.rs", "src", "Rust", content.len()),
            make_row("b.rs", "src", "Rust", content.len()),
        ]);
        let r = build_duplicate_report(tmp.path(), &[f1, f2], &e, &default_limits()).unwrap();
        let density = r.density.as_ref().expect("density should be present");
        assert_eq!(density.duplicate_groups, 1);
        assert_eq!(density.duplicate_files, 2);
        assert!(density.wasted_pct_of_codebase > 0.0);
    }

    #[test]
    fn duplicate_files_in_subdirectories() {
        let tmp = TempDir::new().unwrap();
        let content = b"sub-directory duplicate\n";
        let f1 = write_file(tmp.path(), "src/a.rs", content);
        let f2 = write_file(tmp.path(), "tests/a.rs", content);
        let e = make_export(vec![
            make_row("src/a.rs", "src", "Rust", content.len()),
            make_row("tests/a.rs", "tests", "Rust", content.len()),
        ]);
        let r = build_duplicate_report(tmp.path(), &[f1, f2], &e, &default_limits()).unwrap();
        assert_eq!(r.groups.len(), 1);
    }
}

// ── Determinism ─────────────────────────────────────────────────

mod determinism_w68 {
    use super::*;

    #[test]
    fn todo_report_deterministic() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "a.rs", b"// TODO: x\n// FIXME: y\n// HACK: z\n");
        let r1 = build_todo_report(tmp.path(), &[f.clone()], &default_limits(), 1000).unwrap();
        let r2 = build_todo_report(tmp.path(), &[f], &default_limits(), 1000).unwrap();
        assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap(),
        );
    }

    #[test]
    fn duplicate_report_deterministic() {
        let tmp = TempDir::new().unwrap();
        let content = b"deterministic content\n";
        let f1 = write_file(tmp.path(), "a.rs", content);
        let f2 = write_file(tmp.path(), "b.rs", content);
        let e = make_export(vec![
            make_row("a.rs", "src", "Rust", content.len()),
            make_row("b.rs", "src", "Rust", content.len()),
        ]);
        let r1 = build_duplicate_report(tmp.path(), &[f1.clone(), f2.clone()], &e, &default_limits()).unwrap();
        let r2 = build_duplicate_report(tmp.path(), &[f1, f2], &e, &default_limits()).unwrap();
        assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap(),
        );
    }
}
