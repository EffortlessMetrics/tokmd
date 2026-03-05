//! W76 deep tests for `tokmd-analysis-content`.
//!
//! Exercises TODO/FIXME detection edge cases, duplicate detection with
//! cross-module density, import report granularity, content limits
//! boundary conditions, and determinism invariants.

use std::path::PathBuf;

use tempfile::TempDir;
use tokmd_analysis_content::{
    ContentLimits, ImportGranularity, build_duplicate_report, build_import_report,
    build_todo_report,
};
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

// ═══════════════════════════════════════════════════════════════════
// § 1. TODO/FIXME detection
// ═══════════════════════════════════════════════════════════════════

mod todo_w76 {
    use super::*;

    #[test]
    fn density_zero_when_total_code_is_zero() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "a.rs", b"// TODO: something\n");
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 0).unwrap();
        assert_eq!(r.total, 1);
        assert_eq!(r.density_per_kloc, 0.0, "density should be 0 when kloc is 0");
    }

    #[test]
    fn tags_in_multiline_comments_detected() {
        let tmp = TempDir::new().unwrap();
        let content = "/* TODO: first */\n/* FIXME: second */\n/* HACK: third */\n";
        let f = write_file(tmp.path(), "a.rs", content.as_bytes());
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 1000).unwrap();
        assert_eq!(r.total, 3);
    }

    #[test]
    fn binary_files_skipped() {
        let tmp = TempDir::new().unwrap();
        // Binary content with a TODO string embedded
        let mut content = vec![0u8; 100];
        content.extend_from_slice(b"TODO: hidden in binary");
        let f = write_file(tmp.path(), "a.bin", &content);
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 1000).unwrap();
        assert_eq!(r.total, 0, "binary files should be skipped");
    }

    #[test]
    fn tags_sorted_alphabetically_in_btreemap() {
        let tmp = TempDir::new().unwrap();
        let content = "// XXX: z\n// FIXME: a\n// HACK: m\n// TODO: x\n";
        let f = write_file(tmp.path(), "a.rs", content.as_bytes());
        let r = build_todo_report(tmp.path(), &[f], &default_limits(), 1000).unwrap();
        let tag_names: Vec<&str> = r.tags.iter().map(|t| t.tag.as_str()).collect();
        let mut sorted = tag_names.clone();
        sorted.sort();
        assert_eq!(tag_names, sorted, "tags should be in alphabetical order");
    }

    #[test]
    fn max_bytes_limit_stops_scanning_across_files() {
        let tmp = TempDir::new().unwrap();
        let content = b"// TODO: item\n";
        let f1 = write_file(tmp.path(), "a.rs", content);
        let f2 = write_file(tmp.path(), "b.rs", content);
        let f3 = write_file(tmp.path(), "c.rs", content);
        let limits = ContentLimits {
            max_bytes: Some(content.len() as u64),
            max_file_bytes: None,
        };
        let r = build_todo_report(tmp.path(), &[f1, f2, f3], &limits, 1000).unwrap();
        // Only the first file should be scanned before budget is exhausted
        assert!(r.total <= 2, "max_bytes should cap total scanning, got {}", r.total);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 2. Duplicate detection with cross-module density
// ═══════════════════════════════════════════════════════════════════

mod duplicate_w76 {
    use super::*;

    #[test]
    fn cross_module_density_tracks_separate_modules() {
        let tmp = TempDir::new().unwrap();
        let content = b"shared code content for duplication\n";
        let f1 = write_file(tmp.path(), "src/a.rs", content);
        let f2 = write_file(tmp.path(), "lib/a.rs", content);
        let e = make_export(vec![
            make_row("src/a.rs", "src", "Rust", content.len()),
            make_row("lib/a.rs", "lib", "Rust", content.len()),
        ]);
        let r = build_duplicate_report(tmp.path(), &[f1, f2], &e, &default_limits()).unwrap();
        let density = r.density.as_ref().unwrap();
        assert!(
            density.by_module.len() >= 2,
            "density should track both modules"
        );
    }

    #[test]
    fn file_size_limit_excludes_large_files() {
        let tmp = TempDir::new().unwrap();
        let big = vec![0u8; 200];
        let small = b"tiny\n";
        let f1 = write_file(tmp.path(), "big1.rs", &big);
        let f2 = write_file(tmp.path(), "big2.rs", &big);
        let f3 = write_file(tmp.path(), "sm1.rs", small);
        let f4 = write_file(tmp.path(), "sm2.rs", small);
        let e = make_export(vec![
            make_row("big1.rs", "src", "Rust", 200),
            make_row("big2.rs", "src", "Rust", 200),
            make_row("sm1.rs", "src", "Rust", 5),
            make_row("sm2.rs", "src", "Rust", 5),
        ]);
        let limits = ContentLimits {
            max_bytes: None,
            max_file_bytes: Some(100),
        };
        let r = build_duplicate_report(tmp.path(), &[f1, f2, f3, f4], &e, &limits).unwrap();
        // Big files should be excluded; only small files considered
        assert_eq!(r.groups.len(), 1);
        assert_eq!(r.groups[0].bytes, small.len() as u64);
    }

    #[test]
    fn single_file_never_duplicate() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "only.rs", b"unique\n");
        let e = make_export(vec![make_row("only.rs", "src", "Rust", 7)]);
        let r = build_duplicate_report(tmp.path(), &[f], &e, &default_limits()).unwrap();
        assert!(r.groups.is_empty());
        assert_eq!(r.wasted_bytes, 0);
    }

    #[test]
    fn different_sizes_same_content_prefix_not_duplicate() {
        let tmp = TempDir::new().unwrap();
        let f1 = write_file(tmp.path(), "a.rs", b"prefix content\n");
        let f2 = write_file(tmp.path(), "b.rs", b"prefix content\nextra line\n");
        let e = make_export(vec![
            make_row("a.rs", "src", "Rust", 15),
            make_row("b.rs", "src", "Rust", 27),
        ]);
        let r = build_duplicate_report(tmp.path(), &[f1, f2], &e, &default_limits()).unwrap();
        assert!(r.groups.is_empty(), "different-size files should not match");
    }

    #[test]
    fn wasted_pct_of_codebase_correct() {
        let tmp = TempDir::new().unwrap();
        let content = b"exactly 10 b\n";
        let f1 = write_file(tmp.path(), "a.rs", content);
        let f2 = write_file(tmp.path(), "b.rs", content);
        let e = make_export(vec![
            make_row("a.rs", "src", "Rust", content.len()),
            make_row("b.rs", "src", "Rust", content.len()),
        ]);
        let r = build_duplicate_report(tmp.path(), &[f1, f2], &e, &default_limits()).unwrap();
        let density = r.density.as_ref().unwrap();
        assert!(density.wasted_pct_of_codebase > 0.0);
        assert!(density.wasted_pct_of_codebase <= 1.0);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 3. Import report
// ═══════════════════════════════════════════════════════════════════

mod import_w76 {
    use super::*;

    #[test]
    fn module_granularity_aggregates_by_module() {
        let tmp = TempDir::new().unwrap();
        let content = "use std::collections::HashMap;\nuse std::io;\n";
        let f = write_file(tmp.path(), "src/lib.rs", content.as_bytes());
        let e = make_export(vec![make_row("src/lib.rs", "src", "Rust", content.len())]);
        let r = build_import_report(
            tmp.path(),
            &[f],
            &e,
            ImportGranularity::Module,
            &default_limits(),
        )
        .unwrap();
        assert_eq!(r.granularity, "module");
        for edge in &r.edges {
            assert_eq!(edge.from, "src", "module granularity should use module name");
        }
    }

    #[test]
    fn file_granularity_uses_file_path() {
        let tmp = TempDir::new().unwrap();
        let content = "use std::collections::HashMap;\n";
        let f = write_file(tmp.path(), "src/lib.rs", content.as_bytes());
        let e = make_export(vec![make_row("src/lib.rs", "src", "Rust", content.len())]);
        let r = build_import_report(
            tmp.path(),
            &[f],
            &e,
            ImportGranularity::File,
            &default_limits(),
        )
        .unwrap();
        assert_eq!(r.granularity, "file");
        if !r.edges.is_empty() {
            assert_eq!(r.edges[0].from, "src/lib.rs");
        }
    }

    #[test]
    fn no_imports_produce_empty_edges() {
        let tmp = TempDir::new().unwrap();
        let content = "fn main() { println!(\"hello\"); }\n";
        let f = write_file(tmp.path(), "src/main.rs", content.as_bytes());
        let e = make_export(vec![make_row("src/main.rs", "src", "Rust", content.len())]);
        let r = build_import_report(
            tmp.path(),
            &[f],
            &e,
            ImportGranularity::Module,
            &default_limits(),
        )
        .unwrap();
        assert!(r.edges.is_empty());
    }

    #[test]
    fn edges_sorted_by_count_descending() {
        let tmp = TempDir::new().unwrap();
        let content = "use std::collections::HashMap;\nuse std::io;\nuse std::io::Read;\n";
        let f = write_file(tmp.path(), "src/lib.rs", content.as_bytes());
        let e = make_export(vec![make_row("src/lib.rs", "src", "Rust", content.len())]);
        let r = build_import_report(
            tmp.path(),
            &[f],
            &e,
            ImportGranularity::Module,
            &default_limits(),
        )
        .unwrap();
        for w in r.edges.windows(2) {
            assert!(
                w[0].count >= w[1].count,
                "edges should be sorted desc by count"
            );
        }
    }

    #[test]
    fn unsupported_language_produces_no_edges() {
        let tmp = TempDir::new().unwrap();
        let content = "some random text with import statements\n";
        let f = write_file(tmp.path(), "data.txt", content.as_bytes());
        let e = make_export(vec![make_row("data.txt", "root", "Text", content.len())]);
        let r = build_import_report(
            tmp.path(),
            &[f],
            &e,
            ImportGranularity::Module,
            &default_limits(),
        )
        .unwrap();
        assert!(r.edges.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 4. Determinism and structural invariants
// ═══════════════════════════════════════════════════════════════════

mod invariants_w76 {
    use super::*;

    #[test]
    fn todo_report_stable_across_runs() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(
            tmp.path(),
            "a.rs",
            b"// TODO: a\n// FIXME: b\n// HACK: c\n// XXX: d\n",
        );
        let files = vec![f];
        let r1 = build_todo_report(tmp.path(), &files, &default_limits(), 2000).unwrap();
        let r2 = build_todo_report(tmp.path(), &files, &default_limits(), 2000).unwrap();
        assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap(),
        );
    }

    #[test]
    fn duplicate_density_by_module_sorted_by_wasted_bytes() {
        let tmp = TempDir::new().unwrap();
        let small = b"sm\n";
        let big = b"this is a much bigger duplicate content here\n";
        let fs = write_file(tmp.path(), "s1.rs", small);
        let fs2 = write_file(tmp.path(), "s2.rs", small);
        let fb = write_file(tmp.path(), "b1.rs", big);
        let fb2 = write_file(tmp.path(), "b2.rs", big);
        let e = make_export(vec![
            make_row("s1.rs", "small_mod", "Rust", small.len()),
            make_row("s2.rs", "small_mod", "Rust", small.len()),
            make_row("b1.rs", "big_mod", "Rust", big.len()),
            make_row("b2.rs", "big_mod", "Rust", big.len()),
        ]);
        let r =
            build_duplicate_report(tmp.path(), &[fs, fs2, fb, fb2], &e, &default_limits())
                .unwrap();
        let density = r.density.as_ref().unwrap();
        for w in density.by_module.windows(2) {
            assert!(
                w[0].wasted_bytes >= w[1].wasted_bytes,
                "by_module should be sorted desc by wasted_bytes"
            );
        }
    }

    #[test]
    fn import_report_deterministic() {
        let tmp = TempDir::new().unwrap();
        let content = "use std::io;\nuse std::collections::HashMap;\n";
        let f = write_file(tmp.path(), "src/lib.rs", content.as_bytes());
        let e = make_export(vec![make_row("src/lib.rs", "src", "Rust", content.len())]);
        let files = vec![f];
        let r1 = build_import_report(
            tmp.path(),
            &files,
            &e,
            ImportGranularity::Module,
            &default_limits(),
        )
        .unwrap();
        let r2 = build_import_report(
            tmp.path(),
            &files,
            &e,
            ImportGranularity::Module,
            &default_limits(),
        )
        .unwrap();
        assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap(),
        );
    }

    #[test]
    fn empty_file_list_produces_zero_todo_report() {
        let tmp = TempDir::new().unwrap();
        let r = build_todo_report(tmp.path(), &[], &default_limits(), 5000).unwrap();
        assert_eq!(r.total, 0);
        assert!(r.tags.is_empty());
        assert_eq!(r.density_per_kloc, 0.0);
    }
}
