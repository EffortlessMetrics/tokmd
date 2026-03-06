//! Deep tests for tokmd-export-tree — W76
//!
//! Covers: analysis tree construction, aggregation arithmetic, child-row
//! filtering, handoff tree depth limiting, root sentinel, deterministic
//! ordering, and empty / single-row / wide-tree edge cases.

use tokmd_export_tree::{render_analysis_tree, render_handoff_tree};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── helpers ─────────────────────────────────────────────────────────

fn row(path: &str, kind: FileKind, lines: usize, tokens: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: path.split('/').next().unwrap_or("root").to_string(),
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
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::ParentsOnly,
    }
}

// ═══════════════════════════════════════════════════════════════════
// 1. Analysis tree — aggregation and structure
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w76_analysis_empty_export() {
    assert!(render_analysis_tree(&export(vec![])).is_empty());
}

#[test]
fn w76_analysis_single_file_at_root() {
    let tree = render_analysis_tree(&export(vec![row("main.rs", FileKind::Parent, 50, 100)]));
    assert!(tree.contains("main.rs (lines: 50, tokens: 100)"));
}

#[test]
fn w76_analysis_aggregates_sibling_files() {
    let tree = render_analysis_tree(&export(vec![
        row("src/a.rs", FileKind::Parent, 10, 20),
        row("src/b.rs", FileKind::Parent, 30, 60),
    ]));
    assert!(tree.contains("src (lines: 40, tokens: 80)"));
    assert!(tree.contains("a.rs (lines: 10, tokens: 20)"));
    assert!(tree.contains("b.rs (lines: 30, tokens: 60)"));
}

#[test]
fn w76_analysis_ignores_child_kind_rows() {
    let tree = render_analysis_tree(&export(vec![
        row("src/main.rs", FileKind::Parent, 10, 20),
        row("src/main.rs::html", FileKind::Child, 100, 200),
    ]));
    assert!(tree.contains("src (lines: 10, tokens: 20)"));
    assert!(!tree.contains("200"), "child tokens must not appear");
}

#[test]
fn w76_analysis_deep_nesting_aggregates_upward() {
    let tree = render_analysis_tree(&export(vec![row(
        "a/b/c/d/leaf.rs",
        FileKind::Parent,
        7,
        14,
    )]));
    assert!(tree.contains("a (lines: 7, tokens: 14)"));
    assert!(tree.contains("b (lines: 7, tokens: 14)"));
    assert!(tree.contains("c (lines: 7, tokens: 14)"));
    assert!(tree.contains("d (lines: 7, tokens: 14)"));
    assert!(tree.contains("leaf.rs (lines: 7, tokens: 14)"));
}

#[test]
fn w76_analysis_lexicographic_ordering() {
    let tree = render_analysis_tree(&export(vec![
        row("z.rs", FileKind::Parent, 1, 1),
        row("a.rs", FileKind::Parent, 1, 1),
        row("m.rs", FileKind::Parent, 1, 1),
    ]));
    let a_pos = tree.find("a.rs").unwrap();
    let m_pos = tree.find("m.rs").unwrap();
    let z_pos = tree.find("z.rs").unwrap();
    assert!(a_pos < m_pos, "a.rs must precede m.rs");
    assert!(m_pos < z_pos, "m.rs must precede z.rs");
}

// ═══════════════════════════════════════════════════════════════════
// 2. Handoff tree — depth limiting and root sentinel
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w76_handoff_empty_export() {
    assert!(render_handoff_tree(&export(vec![]), 5).is_empty());
}

#[test]
fn w76_handoff_root_sentinel_always_present() {
    let tree = render_handoff_tree(
        &export(vec![row("src/lib.rs", FileKind::Parent, 10, 20)]),
        3,
    );
    assert!(tree.starts_with("(root)"));
}

#[test]
fn w76_handoff_depth_zero_shows_only_root() {
    let tree = render_handoff_tree(&export(vec![row("a/b/c.rs", FileKind::Parent, 5, 10)]), 0);
    assert!(tree.contains("(root)"));
    assert!(!tree.contains("a/"), "depth 0 must suppress children");
}

#[test]
fn w76_handoff_depth_one_shows_first_level_only() {
    let tree = render_handoff_tree(&export(vec![row("a/b/c.rs", FileKind::Parent, 5, 10)]), 1);
    assert!(tree.contains("a/"));
    assert!(!tree.contains("b/"), "depth 1 must stop at first dir");
}

#[test]
fn w76_handoff_files_count_aggregated() {
    let tree = render_handoff_tree(
        &export(vec![
            row("src/a.rs", FileKind::Parent, 10, 20),
            row("src/b.rs", FileKind::Parent, 20, 40),
            row("lib/c.rs", FileKind::Parent, 5, 10),
        ]),
        3,
    );
    assert!(tree.contains("(root) (files: 3, lines: 35, tokens: 70)"));
    assert!(tree.contains("src/ (files: 2, lines: 30, tokens: 60)"));
    assert!(tree.contains("lib/ (files: 1, lines: 5, tokens: 10)"));
}

#[test]
fn w76_handoff_no_file_leaves_rendered() {
    let tree = render_handoff_tree(
        &export(vec![row("src/main.rs", FileKind::Parent, 10, 20)]),
        10,
    );
    // File leaves are not rendered in handoff trees
    assert!(!tree.contains("main.rs"));
}

#[test]
fn w76_handoff_ignores_child_kind_rows() {
    let tree = render_handoff_tree(
        &export(vec![
            row("src/lib.rs", FileKind::Parent, 10, 20),
            row("src/lib.rs::css", FileKind::Child, 50, 100),
        ]),
        5,
    );
    assert!(tree.contains("(root) (files: 1, lines: 10, tokens: 20)"));
}
