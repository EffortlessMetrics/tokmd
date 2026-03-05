//! W66 deep tests for `tokmd-analysis-content`.
//!
//! Exercises TODO detection, duplicate reporting, edge cases, and density calculations.

use std::path::PathBuf;

use tempfile::TempDir;
use tokmd_analysis_content::{ContentLimits, build_todo_report};
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

// ── TODO density calculation ────────────────────────────────────

mod todo_density_w66 {
    use super::*;

    #[test]
    fn zero_code_lines_yields_zero_density() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "a.rs", b"// TODO: something\n");
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 0).unwrap();
        assert_eq!(r.total, 1);
        assert_eq!(r.density_per_kloc, 0.0);
    }

    #[test]
    fn single_todo_in_1000_lines() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "a.rs", b"// TODO: fix\nfn main() {}\n");
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 1000).unwrap();
        assert_eq!(r.total, 1);
        assert!((r.density_per_kloc - 1.0).abs() < 0.01);
    }

    #[test]
    fn multiple_tags_in_same_file() {
        let tmp = TempDir::new().unwrap();
        let content = "// TODO: a\n// FIXME: b\n// HACK: c\n// XXX: d\nfn main() {}\n";
        let f = write_file(tmp.path(), "a.rs", content.as_bytes());
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 1000).unwrap();
        assert_eq!(r.total, 4);
        let tag_names: Vec<&str> = r.tags.iter().map(|t| t.tag.as_str()).collect();
        let mut sorted = tag_names.clone();
        sorted.sort();
        assert_eq!(tag_names, sorted, "tags should be in BTreeMap order");
    }

    #[test]
    fn todo_density_across_multiple_files() {
        let tmp = TempDir::new().unwrap();
        let f1 = write_file(tmp.path(), "a.rs", b"// TODO: first\n");
        let f2 = write_file(tmp.path(), "b.rs", b"// TODO: second\n// FIXME: third\n");
        let r = build_todo_report(tmp.path(), &[f1, f2], &default_limits(), 2000).unwrap();
        assert_eq!(r.total, 3);
        assert!((r.density_per_kloc - 1.5).abs() < 0.01);
    }

    #[test]
    fn no_tags_produces_zero_total() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "clean.rs", b"fn main() { println!(\"hello\"); }\n");
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 500).unwrap();
        assert_eq!(r.total, 0);
        assert_eq!(r.density_per_kloc, 0.0);
        for tag in &r.tags {
            assert_eq!(tag.count, 0, "tag {} should have 0 count", tag.tag);
        }
    }

    #[test]
    fn empty_file_list_produces_zero() {
        let tmp = TempDir::new().unwrap();
        let r = build_todo_report(tmp.path(), &[], &default_limits(), 1000).unwrap();
        assert_eq!(r.total, 0);
    }

    #[test]
    fn binary_files_skipped() {
        let tmp = TempDir::new().unwrap();
        let mut content = vec![0u8; 512];
        content[0..4].copy_from_slice(b"\x00\x01\x02\x03");
        let f = write_file(tmp.path(), "binary.bin", &content);
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 1000).unwrap();
        assert_eq!(r.total, 0);
    }

    #[test]
    fn todo_in_string_literal_still_counted() {
        let tmp = TempDir::new().unwrap();
        let content = "let s = \"TODO: this is in a string\";\n";
        let f = write_file(tmp.path(), "a.rs", content.as_bytes());
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 100).unwrap();
        assert_eq!(r.total, 1);
    }
}

// ── Content limits ──────────────────────────────────────────────

mod content_limits_w66 {
    use super::*;

    #[test]
    fn max_bytes_limit_stops_scanning() {
        let tmp = TempDir::new().unwrap();
        let mut files = Vec::new();
        for i in 0..100 {
            files.push(write_file(
                tmp.path(),
                &format!("f{i}.rs"),
                b"// TODO: item\nfn main() {}\n",
            ));
        }
        let limits = ContentLimits {
            max_bytes: Some(50),
            max_file_bytes: None,
        };
        let r = build_todo_report(tmp.path(), &files, &limits, 10000).unwrap();
        assert!(r.total < 100, "byte limit should cap scanning; found {}", r.total);
    }

    #[test]
    fn zero_max_bytes_scans_nothing() {
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

// ── Duplicate detection helpers ─────────────────────────────────

mod duplicate_detection_w66 {
    use super::*;
    use tokmd_analysis_content::build_duplicate_report;

    #[test]
    fn no_duplicates_when_all_unique() {
        let tmp = TempDir::new().unwrap();
        let f1 = write_file(tmp.path(), "a.rs", b"fn a() {}");
        let f2 = write_file(tmp.path(), "b.rs", b"fn b() {}");
        let export = make_export(vec![
            make_row("a.rs", "src", "Rust", 9),
            make_row("b.rs", "src", "Rust", 9),
        ]);
        let r = build_duplicate_report(tmp.path(), &[f1, f2], &export, &default_limits()).unwrap();
        assert_eq!(r.wasted_bytes, 0);
        assert!(r.groups.is_empty());
    }

    #[test]
    fn exact_duplicates_detected() {
        let tmp = TempDir::new().unwrap();
        let content = b"fn shared() { println!(\"hello\"); }\n";
        let f1 = write_file(tmp.path(), "a.rs", content);
        let f2 = write_file(tmp.path(), "b.rs", content);
        let export = make_export(vec![
            make_row("a.rs", "src", "Rust", content.len()),
            make_row("b.rs", "src", "Rust", content.len()),
        ]);
        let r = build_duplicate_report(tmp.path(), &[f1, f2], &export, &default_limits()).unwrap();
        assert_eq!(r.groups.len(), 1);
        assert_eq!(r.groups[0].files.len(), 2);
        assert_eq!(r.wasted_bytes, content.len() as u64);
    }

    #[test]
    fn empty_files_not_counted_as_duplicates() {
        let tmp = TempDir::new().unwrap();
        let f1 = write_file(tmp.path(), "a.rs", b"");
        let f2 = write_file(tmp.path(), "b.rs", b"");
        let export = make_export(vec![
            make_row("a.rs", "src", "Rust", 0),
            make_row("b.rs", "src", "Rust", 0),
        ]);
        let r = build_duplicate_report(tmp.path(), &[f1, f2], &export, &default_limits()).unwrap();
        assert_eq!(r.wasted_bytes, 0);
    }

    #[test]
    fn duplicate_groups_sorted_by_bytes_desc() {
        let tmp = TempDir::new().unwrap();
        let small = b"small";
        let big = b"a much bigger duplicated file content here";
        let f1 = write_file(tmp.path(), "s1.rs", small);
        let f2 = write_file(tmp.path(), "s2.rs", small);
        let f3 = write_file(tmp.path(), "b1.rs", big);
        let f4 = write_file(tmp.path(), "b2.rs", big);
        let export = make_export(vec![
            make_row("s1.rs", "src", "Rust", small.len()),
            make_row("s2.rs", "src", "Rust", small.len()),
            make_row("b1.rs", "src", "Rust", big.len()),
            make_row("b2.rs", "src", "Rust", big.len()),
        ]);
        let r = build_duplicate_report(tmp.path(), &[f1, f2, f3, f4], &export, &default_limits()).unwrap();
        assert_eq!(r.groups.len(), 2);
        assert!(r.groups[0].bytes >= r.groups[1].bytes);
    }

    #[test]
    fn strategy_is_exact_blake3() {
        let tmp = TempDir::new().unwrap();
        let r = build_duplicate_report(tmp.path(), &[], &make_export(vec![]), &default_limits()).unwrap();
        assert_eq!(r.strategy, "exact-blake3");
    }
}

// ── Determinism ─────────────────────────────────────────────────

mod determinism_w66 {
    use super::*;

    #[test]
    fn todo_report_deterministic() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "a.rs", b"// TODO: x\n// FIXME: y\n");
        let r1 = build_todo_report(tmp.path(), &[f.clone()], &default_limits(), 1000).unwrap();
        let r2 = build_todo_report(tmp.path(), &[f], &default_limits(), 1000).unwrap();
        assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap(),
        );
    }

    #[test]
    fn todo_tags_sorted_alphabetically() {
        let tmp = TempDir::new().unwrap();
        let content = "// XXX: z\n// FIXME: f\n// TODO: t\n// HACK: h\n";
        let f = write_file(tmp.path(), "a.rs", content.as_bytes());
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 1000).unwrap();
        let names: Vec<&str> = r.tags.iter().map(|t| t.tag.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }
}
