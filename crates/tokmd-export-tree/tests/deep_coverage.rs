//! Deep coverage tests for `tokmd-export-tree`.
//!
//! Exercises tree construction from file rows, tree traversal ordering,
//! empty tree, deeply nested structures, and single-file trees.

use tokmd_export_tree::{render_analysis_tree, render_handoff_tree};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── helpers ─────────────────────────────────────────────────────

fn row(path: &str, kind: FileKind, lines: usize, tokens: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: path.split('/').next().unwrap_or("(root)").to_string(),
        lang: "Rust".to_string(),
        kind,
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

// ===========================================================================
// Tree construction from file rows
// ===========================================================================

#[test]
fn analysis_tree_constructs_from_single_file_row() {
    let tree = render_analysis_tree(&export(vec![row("main.rs", FileKind::Parent, 10, 20)]));
    assert!(tree.contains("main.rs (lines: 10, tokens: 20)"));
    assert_eq!(tree.lines().count(), 1);
}

#[test]
fn analysis_tree_constructs_directory_nodes_from_paths() {
    let tree = render_analysis_tree(&export(vec![
        row("src/lib.rs", FileKind::Parent, 50, 100),
        row("src/utils.rs", FileKind::Parent, 30, 60),
    ]));
    assert!(tree.contains("src (lines: 80, tokens: 160)"));
    assert!(tree.contains("lib.rs (lines: 50, tokens: 100)"));
    assert!(tree.contains("utils.rs (lines: 30, tokens: 60)"));
}

#[test]
fn handoff_tree_constructs_root_from_file_rows() {
    let tree = render_handoff_tree(
        &export(vec![row("src/lib.rs", FileKind::Parent, 50, 100)]),
        5,
    );
    assert!(tree.contains("(root) (files: 1, lines: 50, tokens: 100)"));
    assert!(tree.contains("src/ (files: 1, lines: 50, tokens: 100)"));
}

#[test]
fn analysis_tree_aggregates_across_dirs() {
    let tree = render_analysis_tree(&export(vec![
        row("crates/a/src/lib.rs", FileKind::Parent, 10, 20),
        row("crates/b/src/lib.rs", FileKind::Parent, 30, 60),
    ]));
    // crates directory should aggregate both
    assert!(tree.contains("crates (lines: 40, tokens: 80)"));
}

// ===========================================================================
// Tree traversal ordering (lexicographic)
// ===========================================================================

#[test]
fn analysis_tree_siblings_sorted_lexicographically() {
    let tree = render_analysis_tree(&export(vec![
        row("z.rs", FileKind::Parent, 1, 2),
        row("m.rs", FileKind::Parent, 3, 6),
        row("a.rs", FileKind::Parent, 5, 10),
    ]));
    let lines: Vec<&str> = tree.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("a.rs"));
    assert!(lines[1].starts_with("m.rs"));
    assert!(lines[2].starts_with("z.rs"));
}

#[test]
fn analysis_tree_dirs_sorted_among_dirs() {
    let tree = render_analysis_tree(&export(vec![
        row("zzz/file.rs", FileKind::Parent, 1, 2),
        row("aaa/file.rs", FileKind::Parent, 3, 6),
        row("mmm/file.rs", FileKind::Parent, 5, 10),
    ]));
    let top_lines: Vec<&str> = tree.lines().filter(|l| !l.starts_with(' ')).collect();
    assert!(top_lines[0].starts_with("aaa"));
    assert!(top_lines[1].starts_with("mmm"));
    assert!(top_lines[2].starts_with("zzz"));
}

#[test]
fn handoff_tree_dirs_sorted_lexicographically() {
    let tree = render_handoff_tree(
        &export(vec![
            row("z/file.rs", FileKind::Parent, 1, 2),
            row("a/file.rs", FileKind::Parent, 3, 6),
        ]),
        2,
    );
    let lines: Vec<&str> = tree.lines().collect();
    // First line is (root), then a/, then z/
    assert!(lines[1].contains("a/"));
    assert!(lines[2].contains("z/"));
}

// ===========================================================================
// Empty tree
// ===========================================================================

#[test]
fn analysis_tree_empty_rows_returns_empty_string() {
    let tree = render_analysis_tree(&export(vec![]));
    assert!(tree.is_empty());
}

#[test]
fn handoff_tree_empty_rows_returns_empty_string() {
    let tree = render_handoff_tree(&export(vec![]), 10);
    assert!(tree.is_empty());
}

#[test]
fn analysis_tree_only_child_rows_returns_empty() {
    let tree = render_analysis_tree(&export(vec![row("a.rs::css", FileKind::Child, 10, 20)]));
    assert!(tree.is_empty());
}

#[test]
fn handoff_tree_only_child_rows_returns_empty() {
    let tree = render_handoff_tree(&export(vec![row("a.rs::css", FileKind::Child, 10, 20)]), 3);
    assert!(tree.is_empty());
}

// ===========================================================================
// Deeply nested tree structures
// ===========================================================================

#[test]
fn analysis_tree_8_levels_deep() {
    let tree = render_analysis_tree(&export(vec![row(
        "a/b/c/d/e/f/g/h.rs",
        FileKind::Parent,
        1,
        2,
    )]));
    let lines: Vec<&str> = tree.lines().collect();
    assert_eq!(lines.len(), 8);
    // Each level should have increasing indentation
    for (i, line) in lines.iter().enumerate() {
        let indent = line.len() - line.trim_start().len();
        assert_eq!(indent, i * 2);
    }
}

#[test]
fn handoff_tree_deep_nesting_respects_depth_limit() {
    let tree = render_handoff_tree(
        &export(vec![row("a/b/c/d/e/f/g/h.rs", FileKind::Parent, 1, 2)]),
        3,
    );
    // depth 3: root (depth 0) + a (depth 1) + b (depth 2) + c (depth 3)
    let line_count = tree.lines().count();
    assert!(
        line_count <= 4,
        "expected at most 4 lines at depth 3, got {line_count}"
    );
}

#[test]
fn analysis_tree_aggregation_propagates_upward() {
    let tree = render_analysis_tree(&export(vec![
        row("a/b/c/leaf1.rs", FileKind::Parent, 10, 20),
        row("a/b/c/leaf2.rs", FileKind::Parent, 5, 15),
    ]));
    // Each ancestor should aggregate
    assert!(tree.contains("a (lines: 15, tokens: 35)"));
    assert!(tree.contains("b (lines: 15, tokens: 35)"));
    assert!(tree.contains("c (lines: 15, tokens: 35)"));
}

// ===========================================================================
// Tree with single file
// ===========================================================================

#[test]
fn analysis_tree_single_top_level_file() {
    let tree = render_analysis_tree(&export(vec![row("README.md", FileKind::Parent, 5, 10)]));
    assert_eq!(tree.trim(), "README.md (lines: 5, tokens: 10)");
    assert_eq!(tree.lines().count(), 1);
}

#[test]
fn handoff_tree_single_top_level_file_shows_root_only() {
    let tree = render_handoff_tree(&export(vec![row("README.md", FileKind::Parent, 5, 10)]), 5);
    assert_eq!(tree.lines().count(), 1);
    assert!(tree.contains("(root) (files: 1, lines: 5, tokens: 10)"));
}

#[test]
fn analysis_tree_single_nested_file() {
    let tree = render_analysis_tree(&export(vec![row(
        "deep/path/file.rs",
        FileKind::Parent,
        7,
        14,
    )]));
    let lines: Vec<&str> = tree.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains("deep"));
    assert!(lines[1].contains("path"));
    assert!(lines[2].contains("file.rs"));
}

#[test]
fn handoff_tree_single_nested_file() {
    let tree = render_handoff_tree(
        &export(vec![row("deep/path/file.rs", FileKind::Parent, 7, 14)]),
        5,
    );
    assert!(tree.contains("(root) (files: 1, lines: 7, tokens: 14)"));
    assert!(tree.contains("deep/ (files: 1, lines: 7, tokens: 14)"));
    assert!(tree.contains("path/ (files: 1, lines: 7, tokens: 14)"));
    // file.rs should not appear (handoff doesn't show leaves)
    assert!(!tree.contains("file.rs"));
}

// ===========================================================================
// Zero-valued rows
// ===========================================================================

#[test]
fn analysis_tree_zero_lines_and_tokens() {
    let tree = render_analysis_tree(&export(vec![row("empty.rs", FileKind::Parent, 0, 0)]));
    assert!(tree.contains("empty.rs (lines: 0, tokens: 0)"));
}

#[test]
fn handoff_tree_zero_lines_and_tokens() {
    let tree = render_handoff_tree(
        &export(vec![row("src/empty.rs", FileKind::Parent, 0, 0)]),
        3,
    );
    assert!(tree.contains("(root) (files: 1, lines: 0, tokens: 0)"));
}

// ===========================================================================
// Input order independence
// ===========================================================================

#[test]
fn analysis_tree_input_order_does_not_affect_output() {
    let data1 = export(vec![
        row("z/z.rs", FileKind::Parent, 10, 20),
        row("a/a.rs", FileKind::Parent, 5, 10),
    ]);
    let data2 = export(vec![
        row("a/a.rs", FileKind::Parent, 5, 10),
        row("z/z.rs", FileKind::Parent, 10, 20),
    ]);
    assert_eq!(render_analysis_tree(&data1), render_analysis_tree(&data2));
}

#[test]
fn handoff_tree_input_order_does_not_affect_output() {
    let data1 = export(vec![
        row("z/z.rs", FileKind::Parent, 10, 20),
        row("a/a.rs", FileKind::Parent, 5, 10),
    ]);
    let data2 = export(vec![
        row("a/a.rs", FileKind::Parent, 5, 10),
        row("z/z.rs", FileKind::Parent, 10, 20),
    ]);
    assert_eq!(
        render_handoff_tree(&data1, 3),
        render_handoff_tree(&data2, 3)
    );
}

// ===========================================================================
// Handoff tree large max_depth (no truncation)
// ===========================================================================

#[test]
fn handoff_tree_large_depth_shows_all_dirs() {
    let tree = render_handoff_tree(
        &export(vec![row("a/b/c/d.rs", FileKind::Parent, 1, 2)]),
        100,
    );
    assert!(tree.contains("a/"));
    assert!(tree.contains("b/"));
    assert!(tree.contains("c/"));
    // d.rs is a file leaf, not shown
    assert!(!tree.contains("d.rs"));
}

// ===========================================================================
// Output ends with newline
// ===========================================================================

#[test]
fn analysis_tree_nonempty_ends_with_newline() {
    let tree = render_analysis_tree(&export(vec![row("a.rs", FileKind::Parent, 1, 2)]));
    assert!(!tree.is_empty());
    assert!(tree.ends_with('\n'));
}

#[test]
fn handoff_tree_nonempty_ends_with_newline() {
    let tree = render_handoff_tree(&export(vec![row("a/b.rs", FileKind::Parent, 1, 2)]), 3);
    assert!(!tree.is_empty());
    assert!(tree.ends_with('\n'));
}
