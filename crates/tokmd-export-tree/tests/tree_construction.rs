use tokmd_export_tree::{render_analysis_tree, render_handoff_tree};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

fn row(path: &str, kind: FileKind, lines: usize, tokens: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: "mod".to_string(),
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

// ── analysis tree: empty / single ──────────────────────────────────

#[test]
fn analysis_tree_empty_input_returns_empty_string() {
    assert!(render_analysis_tree(&export(vec![])).is_empty());
}

#[test]
fn analysis_tree_single_root_file() {
    let out = render_analysis_tree(&export(vec![row(
        "main.rs",
        FileKind::Parent,
        50,
        100,
    )]));
    assert!(out.contains("main.rs"));
    assert!(out.contains("lines: 50"));
    assert!(out.contains("tokens: 100"));
}

#[test]
fn analysis_tree_single_nested_file() {
    let out = render_analysis_tree(&export(vec![row(
        "src/lib.rs",
        FileKind::Parent,
        30,
        60,
    )]));
    assert!(out.contains("src"));
    assert!(out.contains("lib.rs"));
    assert!(out.contains("lines: 30"));
}

// ── analysis tree: nested structure ────────────────────────────────

#[test]
fn analysis_tree_aggregates_parent_directory_totals() {
    let out = render_analysis_tree(&export(vec![
        row("src/a.rs", FileKind::Parent, 10, 20),
        row("src/b.rs", FileKind::Parent, 20, 40),
    ]));
    // "src" aggregates both children
    assert!(out.contains("src (lines: 30, tokens: 60)"));
    assert!(out.contains("a.rs (lines: 10, tokens: 20)"));
    assert!(out.contains("b.rs (lines: 20, tokens: 40)"));
}

#[test]
fn analysis_tree_deep_nesting_accumulates() {
    let out = render_analysis_tree(&export(vec![row(
        "a/b/c/d.rs",
        FileKind::Parent,
        5,
        10,
    )]));
    assert!(out.contains("a (lines: 5, tokens: 10)"));
    assert!(out.contains("b (lines: 5, tokens: 10)"));
    assert!(out.contains("c (lines: 5, tokens: 10)"));
    assert!(out.contains("d.rs (lines: 5, tokens: 10)"));
}

// ── analysis tree: filtering ───────────────────────────────────────

#[test]
fn analysis_tree_filters_out_child_kind_rows() {
    let out = render_analysis_tree(&export(vec![
        row("src/main.rs", FileKind::Parent, 10, 20),
        row("src/main.rs::css", FileKind::Child, 5, 10),
    ]));
    assert!(out.contains("main.rs (lines: 10, tokens: 20)"));
    assert!(!out.contains("css"));
}

// ── analysis tree: determinism ─────────────────────────────────────

#[test]
fn analysis_tree_lexicographic_ordering() {
    let out = render_analysis_tree(&export(vec![
        row("src/z.rs", FileKind::Parent, 1, 2),
        row("src/a.rs", FileKind::Parent, 3, 4),
        row("src/m.rs", FileKind::Parent, 5, 6),
    ]));
    let a_pos = out.find("a.rs").unwrap();
    let m_pos = out.find("m.rs").unwrap();
    let z_pos = out.find("z.rs").unwrap();
    assert!(a_pos < m_pos);
    assert!(m_pos < z_pos);
}

#[test]
fn analysis_tree_deterministic_across_calls() {
    let data = export(vec![
        row("src/a.rs", FileKind::Parent, 10, 20),
        row("src/b.rs", FileKind::Parent, 5, 10),
        row("lib/c.rs", FileKind::Parent, 3, 6),
    ]);
    let first = render_analysis_tree(&data);
    let second = render_analysis_tree(&data);
    assert_eq!(first, second);
}

// ── analysis tree: multiple top-level directories ──────────────────

#[test]
fn analysis_tree_multiple_top_level_dirs() {
    let out = render_analysis_tree(&export(vec![
        row("src/lib.rs", FileKind::Parent, 10, 20),
        row("tests/test.rs", FileKind::Parent, 5, 10),
    ]));
    assert!(out.contains("src (lines: 10, tokens: 20)"));
    assert!(out.contains("tests (lines: 5, tokens: 10)"));
}

// ── handoff tree: empty ────────────────────────────────────────────

#[test]
fn handoff_tree_empty_input_returns_empty_string() {
    assert!(render_handoff_tree(&export(vec![]), 5).is_empty());
}

// ── handoff tree: root node ────────────────────────────────────────

#[test]
fn handoff_tree_shows_root_with_totals() {
    let out = render_handoff_tree(
        &export(vec![row("src/main.rs", FileKind::Parent, 10, 20)]),
        3,
    );
    assert!(out.contains("(root)"));
    assert!(out.contains("files: 1"));
    assert!(out.contains("lines: 10"));
    assert!(out.contains("tokens: 20"));
}

#[test]
fn handoff_tree_root_aggregates_all_files() {
    let out = render_handoff_tree(
        &export(vec![
            row("src/a.rs", FileKind::Parent, 10, 20),
            row("src/b.rs", FileKind::Parent, 5, 10),
            row("lib/c.rs", FileKind::Parent, 3, 6),
        ]),
        5,
    );
    assert!(out.contains("(root) (files: 3, lines: 18, tokens: 36)"));
}

// ── handoff tree: depth limiting ───────────────────────────────────

#[test]
fn handoff_tree_depth_zero_shows_only_root() {
    let out = render_handoff_tree(
        &export(vec![row("a/b/c/d.rs", FileKind::Parent, 10, 20)]),
        0,
    );
    assert!(out.contains("(root)"));
    assert!(!out.contains("a/"));
}

#[test]
fn handoff_tree_depth_one_shows_first_level_only() {
    let out = render_handoff_tree(
        &export(vec![row("a/b/c/d.rs", FileKind::Parent, 10, 20)]),
        1,
    );
    assert!(out.contains("(root)"));
    assert!(out.contains("a/"));
    assert!(!out.contains("b/"));
}

#[test]
fn handoff_tree_depth_two_shows_two_levels() {
    let out = render_handoff_tree(
        &export(vec![row("a/b/c/d.rs", FileKind::Parent, 10, 20)]),
        2,
    );
    assert!(out.contains("a/"));
    assert!(out.contains("b/"));
    assert!(!out.contains("c/"));
}

// ── handoff tree: no file leaves ───────────────────────────────────

#[test]
fn handoff_tree_excludes_file_leaves() {
    let out = render_handoff_tree(
        &export(vec![row("src/main.rs", FileKind::Parent, 10, 20)]),
        10,
    );
    assert!(out.contains("src/"));
    assert!(!out.contains("main.rs"));
}

// ── handoff tree: determinism ──────────────────────────────────────

#[test]
fn handoff_tree_lexicographic_ordering() {
    let out = render_handoff_tree(
        &export(vec![
            row("z/file.rs", FileKind::Parent, 1, 2),
            row("a/file.rs", FileKind::Parent, 3, 4),
            row("m/file.rs", FileKind::Parent, 5, 6),
        ]),
        2,
    );
    let a_pos = out.find("a/").unwrap();
    let m_pos = out.find("m/").unwrap();
    let z_pos = out.find("z/").unwrap();
    assert!(a_pos < m_pos);
    assert!(m_pos < z_pos);
}

#[test]
fn handoff_tree_deterministic_across_calls() {
    let data = export(vec![
        row("src/a.rs", FileKind::Parent, 10, 20),
        row("lib/b.rs", FileKind::Parent, 5, 10),
    ]);
    let first = render_handoff_tree(&data, 3);
    let second = render_handoff_tree(&data, 3);
    assert_eq!(first, second);
}

// ── handoff tree: filters child rows ───────────────────────────────

#[test]
fn handoff_tree_filters_out_child_kind_rows() {
    let out = render_handoff_tree(
        &export(vec![
            row("src/main.rs", FileKind::Parent, 10, 20),
            row("src/main.rs::css", FileKind::Child, 5, 10),
        ]),
        5,
    );
    // Only parent row counted
    assert!(out.contains("files: 1, lines: 10, tokens: 20"));
}

// ── handoff tree: single root-level file ───────────────────────────

#[test]
fn handoff_tree_single_root_level_file_returns_empty() {
    // A root-level file has no directory segments, so handoff inserts
    // into root but there are no directory children to render beyond root.
    let out = render_handoff_tree(
        &export(vec![row("Cargo.toml", FileKind::Parent, 10, 20)]),
        5,
    );
    // Root file has no slash, so insert_handoff's `!tail.is_empty()` check
    // means no children get created — only the root line appears.
    assert!(out.contains("(root) (files: 1, lines: 10, tokens: 20)"));
}
