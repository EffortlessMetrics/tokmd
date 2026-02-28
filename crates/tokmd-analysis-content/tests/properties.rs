//! Property-based tests for tokmd-analysis-content.
//!
//! Verifies invariants of build_todo_report, build_duplicate_report,
//! and build_import_report across random inputs.

use std::path::PathBuf;

use proptest::prelude::*;
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
        comments: 0,
        blanks: 0,
        lines: 10,
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

// ── build_todo_report properties ─────────────────────────────────────

proptest! {
    #[test]
    fn todo_total_equals_sum_of_tag_counts(
        todo_count in 0usize..10,
        fixme_count in 0usize..10,
    ) {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        let mut content = String::new();
        for _ in 0..todo_count {
            content.push_str("// TODO: item\n");
        }
        for _ in 0..fixme_count {
            content.push_str("// FIXME: item\n");
        }
        content.push_str("fn main() {}\n");

        std::fs::write(root.join("test.rs"), &content).unwrap();

        let files = vec![PathBuf::from("test.rs")];
        let report = build_todo_report(root, &files, &ContentLimits::default(), 1000).unwrap();

        let tag_sum: usize = report.tags.iter().map(|t| t.count).sum();
        prop_assert_eq!(report.total, tag_sum, "total must equal sum of tag counts");
    }

    #[test]
    fn todo_density_is_non_negative(total_code in 0usize..100_000) {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        std::fs::write(root.join("f.rs"), "// TODO: x\nfn f() {}\n").unwrap();

        let files = vec![PathBuf::from("f.rs")];
        let report = build_todo_report(root, &files, &ContentLimits::default(), total_code).unwrap();

        prop_assert!(report.density_per_kloc >= 0.0, "density must be non-negative");
    }

    #[test]
    fn todo_density_zero_when_code_zero(tag_lines in 0usize..5) {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        let mut content = String::new();
        for _ in 0..tag_lines {
            content.push_str("// TODO: task\n");
        }
        content.push_str("fn main() {}\n");

        std::fs::write(root.join("z.rs"), &content).unwrap();

        let files = vec![PathBuf::from("z.rs")];
        let report = build_todo_report(root, &files, &ContentLimits::default(), 0).unwrap();

        prop_assert_eq!(report.density_per_kloc, 0.0, "density must be 0 when total_code is 0");
    }
}

// ── build_duplicate_report properties ────────────────────────────────

proptest! {
    #[test]
    fn duplicate_wasted_bytes_never_negative(
        file_count in 2usize..6,
    ) {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        let content = "duplicate content for property test\n";
        let mut files = Vec::new();
        let mut rows = Vec::new();
        for i in 0..file_count {
            let name = format!("f{i}.txt");
            std::fs::write(root.join(&name), content).unwrap();
            files.push(PathBuf::from(&name));
            rows.push(file_row(&name, "root", "Text", content.len()));
        }

        let export = ExportData {
            rows,
            module_roots: vec!["root".to_string()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };

        let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

        prop_assert!(report.wasted_bytes <= (file_count as u64 - 1) * content.len() as u64);
        // At least one group should exist (all identical)
        prop_assert!(!report.groups.is_empty(), "identical files must form a group");
    }

    #[test]
    fn duplicate_report_strategy_is_always_exact_blake3(
        n in 0usize..3,
    ) {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        let mut files = Vec::new();
        for i in 0..n {
            let name = format!("file{i}.txt");
            std::fs::write(root.join(&name), format!("content {i}\n")).unwrap();
            files.push(PathBuf::from(&name));
        }

        let report = build_duplicate_report(root, &files, &empty_export(), &ContentLimits::default()).unwrap();

        prop_assert_eq!(report.strategy, "exact-blake3");
    }

    #[test]
    fn duplicate_groups_sorted_by_bytes_desc(
        _seed in 0u32..100,
    ) {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        // Create two groups of duplicates with different sizes
        let small = "sm\n";
        let large = "x".repeat(50) + "\n";
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

        let report = build_duplicate_report(root, &files, &empty_export(), &ContentLimits::default()).unwrap();

        for w in report.groups.windows(2) {
            prop_assert!(w[0].bytes >= w[1].bytes, "groups must be sorted by bytes desc");
        }
    }

    #[test]
    fn duplicate_group_files_are_sorted(
        _seed in 0u32..100,
    ) {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        let content = "identical for sorting test\n";
        std::fs::write(root.join("z.txt"), content).unwrap();
        std::fs::write(root.join("a.txt"), content).unwrap();
        std::fs::write(root.join("m.txt"), content).unwrap();

        let files = vec![
            PathBuf::from("z.txt"),
            PathBuf::from("a.txt"),
            PathBuf::from("m.txt"),
        ];

        let report = build_duplicate_report(root, &files, &empty_export(), &ContentLimits::default()).unwrap();

        for group in &report.groups {
            let sorted: Vec<_> = {
                let mut s = group.files.clone();
                s.sort();
                s
            };
            prop_assert_eq!(&group.files, &sorted, "files within group must be sorted");
        }
    }
}

// ── build_import_report properties ───────────────────────────────────

proptest! {
    #[test]
    fn import_edges_have_positive_counts(
        _seed in 0u32..50,
    ) {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        std::fs::write(root.join("main.py"), "import os\nimport sys\n").unwrap();

        let files = vec![PathBuf::from("main.py")];
        let export = ExportData {
            rows: vec![file_row("main.py", "root", "Python", 20)],
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
        ).unwrap();

        for edge in &report.edges {
            prop_assert!(edge.count > 0, "edge count must be positive");
        }
    }

    #[test]
    fn import_edges_sorted_by_count_desc(
        _seed in 0u32..50,
    ) {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        std::fs::write(
            root.join("lib.rs"),
            "use serde::Serialize;\nuse serde::Deserialize;\nuse tokio;\n",
        ).unwrap();

        let files = vec![PathBuf::from("lib.rs")];
        let export = ExportData {
            rows: vec![file_row("lib.rs", "root", "Rust", 80)],
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
        ).unwrap();

        for w in report.edges.windows(2) {
            prop_assert!(w[0].count >= w[1].count, "edges must be sorted by count desc");
        }
    }

    #[test]
    fn import_granularity_field_matches_input(use_file in proptest::bool::ANY) {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        let gran = if use_file {
            ImportGranularity::File
        } else {
            ImportGranularity::Module
        };

        let report = build_import_report(
            root,
            &[],
            &empty_export(),
            gran,
            &ContentLimits::default(),
        ).unwrap();

        let expected = if use_file { "file" } else { "module" };
        prop_assert_eq!(report.granularity, expected);
    }
}

// ── cross-function invariants ────────────────────────────────────────

proptest! {
    #[test]
    fn empty_files_always_produce_empty_results(_seed in 0u32..20) {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let files: Vec<PathBuf> = vec![];
        let export = empty_export();

        let todo = build_todo_report(root, &files, &ContentLimits::default(), 100).unwrap();
        prop_assert_eq!(todo.total, 0);

        let dup = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();
        prop_assert!(dup.groups.is_empty());

        let imp = build_import_report(
            root, &files, &export,
            ImportGranularity::Module,
            &ContentLimits::default(),
        ).unwrap();
        prop_assert!(imp.edges.is_empty());
    }
}
