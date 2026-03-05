//! W59 – Property-based tests for tokmd-export-tree.

use proptest::prelude::*;
use tokmd_export_tree::{render_analysis_tree, render_handoff_tree};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

fn path_segment() -> impl Strategy<Value = String> {
    "[a-z]{1,8}".prop_map(String::from)
}

fn filename() -> impl Strategy<Value = String> {
    "[a-z]{1,6}\\.rs".prop_map(String::from)
}

fn path_strategy() -> impl Strategy<Value = String> {
    (prop::collection::vec(path_segment(), 0..4), filename()).prop_map(|(dirs, file)| {
        if dirs.is_empty() {
            file
        } else {
            format!("{}/{}", dirs.join("/"), file)
        }
    })
}

fn parent_row(path: String, lines: usize, tokens: usize) -> FileRow {
    let module = path
        .split('/')
        .next()
        .map_or_else(|| "(root)".to_string(), String::from);
    FileRow {
        path,
        module,
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: lines,
        comments: 0,
        blanks: 0,
        lines,
        bytes: lines * 10,
        tokens,
    }
}

fn export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::ParentsOnly,
    }
}

proptest! {
    /// Handoff tree output never contains more indented lines than `max_depth` levels.
    #[test]
    fn handoff_depth_never_exceeds_limit(
        paths in prop::collection::vec(path_strategy(), 1..32),
        max_depth in 0usize..6,
    ) {
        let rows: Vec<FileRow> = paths
            .into_iter()
            .enumerate()
            .map(|(i, p)| parent_row(p, i + 1, (i + 1) * 2))
            .collect();
        let tree = render_handoff_tree(&export(rows), max_depth);
        for line in tree.lines() {
            let leading_spaces = line.len() - line.trim_start().len();
            let indent_level = leading_spaces / 2;
            prop_assert!(
                indent_level <= max_depth,
                "indent_level={indent_level} > max_depth={max_depth} in line: {line}"
            );
        }
    }

    /// Handoff tree root file count always equals the number of Parent rows.
    #[test]
    fn handoff_root_files_eq_parent_count(
        paths in prop::collection::vec(path_strategy(), 1..32),
    ) {
        let rows: Vec<FileRow> = paths
            .into_iter()
            .enumerate()
            .map(|(i, p)| parent_row(p, i + 1, (i + 1) * 2))
            .collect();
        let count = rows.len();
        let tree = render_handoff_tree(&export(rows), 10);
        if let Some(first_line) = tree.lines().next() {
            let expected = format!("(root) (files: {count},");
            prop_assert!(
                first_line.starts_with(&expected),
                "expected '{expected}', got '{first_line}'"
            );
        }
    }

    /// Analysis tree is deterministic: same input ⇒ same output.
    #[test]
    fn analysis_tree_deterministic(
        paths in prop::collection::vec(path_strategy(), 0..32),
    ) {
        let rows: Vec<FileRow> = paths
            .into_iter()
            .enumerate()
            .map(|(i, p)| parent_row(p, i + 1, (i + 1) * 2))
            .collect();
        let e = export(rows);
        let a = render_analysis_tree(&e);
        let b = render_analysis_tree(&e);
        prop_assert_eq!(a, b);
    }

    /// Handoff tree is deterministic: same input ⇒ same output.
    #[test]
    fn handoff_tree_deterministic(
        paths in prop::collection::vec(path_strategy(), 0..32),
        depth in 0usize..8,
    ) {
        let rows: Vec<FileRow> = paths
            .into_iter()
            .enumerate()
            .map(|(i, p)| parent_row(p, i + 1, (i + 1) * 2))
            .collect();
        let e = export(rows);
        let a = render_handoff_tree(&e, depth);
        let b = render_handoff_tree(&e, depth);
        prop_assert_eq!(a, b);
    }

    /// Reversing input rows produces identical output (order independence).
    #[test]
    fn order_independence(
        paths in prop::collection::vec(path_strategy(), 0..32),
        depth in 0usize..8,
    ) {
        let rows: Vec<FileRow> = paths
            .into_iter()
            .enumerate()
            .map(|(i, p)| parent_row(p, i + 1, (i + 1) * 2))
            .collect();
        let mut rev = rows.clone();
        rev.reverse();
        prop_assert_eq!(
            render_analysis_tree(&export(rows.clone())),
            render_analysis_tree(&export(rev.clone())),
        );
        prop_assert_eq!(
            render_handoff_tree(&export(rows), depth),
            render_handoff_tree(&export(rev), depth),
        );
    }

    /// Non-empty parent rows always produce non-empty analysis tree output.
    #[test]
    fn non_empty_parents_produce_non_empty_analysis(
        paths in prop::collection::vec(path_strategy(), 1..16),
    ) {
        let rows: Vec<FileRow> = paths
            .into_iter()
            .enumerate()
            .map(|(i, p)| parent_row(p, i + 1, (i + 1) * 2))
            .collect();
        let tree = render_analysis_tree(&export(rows));
        prop_assert!(!tree.is_empty());
    }

    /// Non-empty parent rows always produce non-empty handoff tree output.
    #[test]
    fn non_empty_parents_produce_non_empty_handoff(
        paths in prop::collection::vec(path_strategy(), 1..16),
        depth in 0usize..8,
    ) {
        let rows: Vec<FileRow> = paths
            .into_iter()
            .enumerate()
            .map(|(i, p)| parent_row(p, i + 1, (i + 1) * 2))
            .collect();
        let tree = render_handoff_tree(&export(rows), depth);
        prop_assert!(!tree.is_empty());
    }

    /// Every non-empty analysis tree ends with a newline.
    #[test]
    fn analysis_tree_trailing_newline(
        paths in prop::collection::vec(path_strategy(), 1..16),
    ) {
        let rows: Vec<FileRow> = paths
            .into_iter()
            .enumerate()
            .map(|(i, p)| parent_row(p, i + 1, (i + 1) * 2))
            .collect();
        let tree = render_analysis_tree(&export(rows));
        prop_assert!(tree.ends_with('\n'), "expected trailing newline");
    }

    /// Handoff tree `.rs` file leaves are never rendered.
    #[test]
    fn handoff_never_contains_rs_extension(
        paths in prop::collection::vec(path_strategy(), 1..16),
        depth in 0usize..8,
    ) {
        let rows: Vec<FileRow> = paths
            .into_iter()
            .enumerate()
            .map(|(i, p)| parent_row(p, i + 1, (i + 1) * 2))
            .collect();
        let tree = render_handoff_tree(&export(rows), depth);
        prop_assert!(!tree.contains(".rs"), "handoff should not show file leaves");
    }
}
