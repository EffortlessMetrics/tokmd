//! Deep tests for export-tree rendering (Wave 43).

use tokmd_export_tree::{render_analysis_tree, render_handoff_tree};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn row(path: &str, kind: FileKind, lines: usize, tokens: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: "src".to_string(),
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
// Analysis tree tests
// ===========================================================================

#[test]
fn analysis_tree_single_file() {
    let out = render_analysis_tree(&export(vec![row("src/lib.rs", FileKind::Parent, 100, 200)]));
    assert!(out.contains("src (lines: 100, tokens: 200)"));
    assert!(out.contains("lib.rs (lines: 100, tokens: 200)"));
}

#[test]
fn analysis_tree_multiple_files_same_dir() {
    let data = export(vec![
        row("src/a.rs", FileKind::Parent, 10, 20),
        row("src/b.rs", FileKind::Parent, 30, 60),
    ]);
    let out = render_analysis_tree(&data);
    // Directory aggregates both files
    assert!(out.contains("src (lines: 40, tokens: 80)"));
    assert!(out.contains("a.rs (lines: 10, tokens: 20)"));
    assert!(out.contains("b.rs (lines: 30, tokens: 60)"));
}

#[test]
fn analysis_tree_nested_dirs() {
    let data = export(vec![row("src/foo/bar/baz.rs", FileKind::Parent, 5, 10)]);
    let out = render_analysis_tree(&data);
    assert!(out.contains("src"));
    assert!(out.contains("foo"));
    assert!(out.contains("bar"));
    assert!(out.contains("baz.rs (lines: 5, tokens: 10)"));
}

#[test]
fn analysis_tree_lexicographic_order() {
    let data = export(vec![
        row("src/z.rs", FileKind::Parent, 1, 1),
        row("src/a.rs", FileKind::Parent, 1, 1),
        row("src/m.rs", FileKind::Parent, 1, 1),
    ]);
    let out = render_analysis_tree(&data);
    let a_pos = out.find("a.rs").unwrap();
    let m_pos = out.find("m.rs").unwrap();
    let z_pos = out.find("z.rs").unwrap();
    assert!(a_pos < m_pos, "a.rs should appear before m.rs");
    assert!(m_pos < z_pos, "m.rs should appear before z.rs");
}

#[test]
fn analysis_tree_filters_child_rows() {
    let data = export(vec![
        row("src/main.rs", FileKind::Parent, 50, 100),
        row("src/main.rs::css", FileKind::Child, 20, 40),
    ]);
    let out = render_analysis_tree(&data);
    assert!(out.contains("main.rs (lines: 50, tokens: 100)"));
    assert!(!out.contains("css"), "Child rows should be excluded");
}

#[test]
fn analysis_tree_aggregates_across_subtrees() {
    let data = export(vec![
        row("crates/a/src/lib.rs", FileKind::Parent, 10, 20),
        row("crates/b/src/lib.rs", FileKind::Parent, 30, 60),
    ]);
    let out = render_analysis_tree(&data);
    assert!(out.contains("crates (lines: 40, tokens: 80)"));
}

// ===========================================================================
// Handoff tree tests
// ===========================================================================

#[test]
fn handoff_tree_single_file_depth_1() {
    let out = render_handoff_tree(
        &export(vec![row("src/lib.rs", FileKind::Parent, 100, 200)]),
        1,
    );
    assert!(out.contains("(root) (files: 1, lines: 100, tokens: 200)"));
    assert!(out.contains("src/ (files: 1, lines: 100, tokens: 200)"));
    // File leaf should NOT appear
    assert!(!out.contains("lib.rs"));
}

#[test]
fn handoff_tree_depth_limit_truncates() {
    let data = export(vec![row("a/b/c/d/e.rs", FileKind::Parent, 10, 20)]);
    let out = render_handoff_tree(&data, 2);
    assert!(out.contains("a/"));
    assert!(out.contains("b/"));
    // depth 2 means root(0) + a(1) + b(2) → c should not appear
    assert!(!out.contains("c/"));
}

#[test]
fn handoff_tree_depth_zero_shows_root_only() {
    let data = export(vec![row("src/lib.rs", FileKind::Parent, 10, 20)]);
    let out = render_handoff_tree(&data, 0);
    // root is at depth 0, so it should appear but nothing else
    assert!(out.contains("(root)"));
    assert!(!out.contains("src/"));
}

#[test]
fn handoff_tree_multiple_dirs_aggregated() {
    let data = export(vec![
        row("src/a.rs", FileKind::Parent, 10, 20),
        row("src/b.rs", FileKind::Parent, 30, 60),
        row("tests/t.rs", FileKind::Parent, 5, 10),
    ]);
    let out = render_handoff_tree(&data, 2);
    assert!(out.contains("(root) (files: 3, lines: 45, tokens: 90)"));
    assert!(out.contains("src/ (files: 2, lines: 40, tokens: 80)"));
    assert!(out.contains("tests/ (files: 1, lines: 5, tokens: 10)"));
}

#[test]
fn handoff_tree_filters_child_rows() {
    let data = export(vec![
        row("src/main.rs", FileKind::Parent, 50, 100),
        row("src/main.rs::embedded", FileKind::Child, 20, 40),
    ]);
    let out = render_handoff_tree(&data, 3);
    // Only 1 file (parent), child excluded
    assert!(out.contains("(root) (files: 1, lines: 50, tokens: 100)"));
}

#[test]
fn handoff_tree_lexicographic_order() {
    let data = export(vec![
        row("z/file.rs", FileKind::Parent, 1, 1),
        row("a/file.rs", FileKind::Parent, 1, 1),
        row("m/file.rs", FileKind::Parent, 1, 1),
    ]);
    let out = render_handoff_tree(&data, 2);
    let a_pos = out.find("a/").unwrap();
    let m_pos = out.find("m/").unwrap();
    let z_pos = out.find("z/").unwrap();
    assert!(a_pos < m_pos);
    assert!(m_pos < z_pos);
}

#[test]
fn handoff_tree_large_depth_shows_everything() {
    let data = export(vec![row("a/b/c/d.rs", FileKind::Parent, 10, 20)]);
    let out = render_handoff_tree(&data, 100);
    assert!(out.contains("a/"));
    assert!(out.contains("b/"));
    assert!(out.contains("c/"));
    // d.rs is a file leaf — handoff tree excludes file leaves
    assert!(!out.contains("d.rs"));
}
