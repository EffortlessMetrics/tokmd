//! Depth tests for tokmd-export-tree: tree construction, rendering, sorting, edge cases.

use tokmd_export_tree::{render_analysis_tree, render_handoff_tree};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ──────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────

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

// ──────────────────────────────────────────────────────────────────────
// 1. Analysis tree construction
// ──────────────────────────────────────────────────────────────────────

#[test]
fn analysis_tree_empty_returns_empty_string() {
    assert!(render_analysis_tree(&export(vec![])).is_empty());
}

#[test]
fn analysis_tree_single_file_at_root() {
    let out = render_analysis_tree(&export(vec![row("main.rs", FileKind::Parent, 10, 20)]));
    assert!(out.contains("main.rs (lines: 10, tokens: 20)"));
}

#[test]
fn analysis_tree_single_file_nested() {
    let out = render_analysis_tree(&export(vec![row("src/lib.rs", FileKind::Parent, 50, 100)]));
    assert!(out.contains("src (lines: 50, tokens: 100)"));
    assert!(out.contains("lib.rs (lines: 50, tokens: 100)"));
}

#[test]
fn analysis_tree_aggregates_directory_totals() {
    let out = render_analysis_tree(&export(vec![
        row("src/a.rs", FileKind::Parent, 10, 20),
        row("src/b.rs", FileKind::Parent, 30, 60),
    ]));
    assert!(out.contains("src (lines: 40, tokens: 80)"));
    assert!(out.contains("a.rs (lines: 10, tokens: 20)"));
    assert!(out.contains("b.rs (lines: 30, tokens: 60)"));
}

#[test]
fn analysis_tree_filters_child_rows() {
    let out = render_analysis_tree(&export(vec![
        row("src/main.rs", FileKind::Parent, 100, 200),
        row("src/main.rs::embedded", FileKind::Child, 50, 100),
    ]));
    assert!(out.contains("main.rs (lines: 100, tokens: 200)"));
    assert!(!out.contains("embedded"));
}

// ──────────────────────────────────────────────────────────────────────
// 2. Directory tree rendering
// ──────────────────────────────────────────────────────────────────────

#[test]
fn analysis_tree_deeply_nested_path() {
    let out = render_analysis_tree(&export(vec![row("a/b/c/d/e.rs", FileKind::Parent, 5, 10)]));
    assert!(out.contains("a (lines: 5, tokens: 10)"));
    assert!(out.contains("b (lines: 5, tokens: 10)"));
    assert!(out.contains("c (lines: 5, tokens: 10)"));
    assert!(out.contains("d (lines: 5, tokens: 10)"));
    assert!(out.contains("e.rs (lines: 5, tokens: 10)"));
}

#[test]
fn analysis_tree_indentation_increases_with_depth() {
    let out = render_analysis_tree(&export(vec![row("a/b/c.rs", FileKind::Parent, 5, 10)]));
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 3);
    // Each level should have more leading whitespace
    let indent0 = lines[0].len() - lines[0].trim_start().len();
    let indent1 = lines[1].len() - lines[1].trim_start().len();
    let indent2 = lines[2].len() - lines[2].trim_start().len();
    assert!(indent1 > indent0);
    assert!(indent2 > indent1);
}

#[test]
fn analysis_tree_multiple_directories() {
    let out = render_analysis_tree(&export(vec![
        row("src/lib.rs", FileKind::Parent, 100, 200),
        row("tests/test.rs", FileKind::Parent, 50, 100),
    ]));
    assert!(out.contains("src (lines: 100, tokens: 200)"));
    assert!(out.contains("tests (lines: 50, tokens: 100)"));
}

// ──────────────────────────────────────────────────────────────────────
// 3. Sorting and ordering (deterministic output)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn analysis_tree_siblings_sorted_lexicographically() {
    let out = render_analysis_tree(&export(vec![
        row("src/zebra.rs", FileKind::Parent, 10, 20),
        row("src/alpha.rs", FileKind::Parent, 10, 20),
        row("src/middle.rs", FileKind::Parent, 10, 20),
    ]));
    let lines: Vec<&str> = out.lines().collect();
    // First line: src, then children in alphabetical order
    let names: Vec<&str> = lines
        .iter()
        .filter_map(|l| {
            let trimmed = l.trim();
            trimmed.split(' ').next()
        })
        .collect();
    // Should contain: src, alpha.rs, middle.rs, zebra.rs (sorted)
    let alpha_pos = names.iter().position(|n| *n == "alpha.rs").unwrap();
    let middle_pos = names.iter().position(|n| *n == "middle.rs").unwrap();
    let zebra_pos = names.iter().position(|n| *n == "zebra.rs").unwrap();
    assert!(alpha_pos < middle_pos);
    assert!(middle_pos < zebra_pos);
}

#[test]
fn analysis_tree_deterministic_across_runs() {
    let rows = vec![
        row("b/x.rs", FileKind::Parent, 10, 20),
        row("a/y.rs", FileKind::Parent, 30, 60),
    ];
    let out1 = render_analysis_tree(&export(rows.clone()));
    let out2 = render_analysis_tree(&export(rows));
    assert_eq!(out1, out2);
}

#[test]
fn analysis_tree_dir_ordering_is_lexicographic() {
    let out = render_analysis_tree(&export(vec![
        row("zzz/a.rs", FileKind::Parent, 1, 2),
        row("aaa/b.rs", FileKind::Parent, 3, 4),
    ]));
    let lines: Vec<&str> = out.lines().collect();
    // aaa should come before zzz
    let aaa_pos = lines.iter().position(|l| l.contains("aaa")).unwrap();
    let zzz_pos = lines.iter().position(|l| l.contains("zzz")).unwrap();
    assert!(aaa_pos < zzz_pos);
}

// ──────────────────────────────────────────────────────────────────────
// 4. Edge cases: flat repo, deeply nested, single file
// ──────────────────────────────────────────────────────────────────────

#[test]
fn analysis_tree_flat_repo_no_directories() {
    let out = render_analysis_tree(&export(vec![
        row("a.rs", FileKind::Parent, 10, 20),
        row("b.rs", FileKind::Parent, 20, 40),
    ]));
    assert!(out.contains("a.rs (lines: 10, tokens: 20)"));
    assert!(out.contains("b.rs (lines: 20, tokens: 40)"));
    // No directory grouping
    assert_eq!(out.lines().count(), 2);
}

#[test]
fn analysis_tree_deeply_nested_10_levels() {
    let out = render_analysis_tree(&export(vec![row(
        "a/b/c/d/e/f/g/h/i/j.rs",
        FileKind::Parent,
        1,
        2,
    )]));
    // 9 directory levels + 1 file = 10 lines
    assert_eq!(out.lines().count(), 10);
}

#[test]
fn analysis_tree_zero_lines_and_tokens() {
    let out = render_analysis_tree(&export(vec![row("empty.rs", FileKind::Parent, 0, 0)]));
    assert!(out.contains("empty.rs (lines: 0, tokens: 0)"));
}

#[test]
fn analysis_tree_all_child_rows_produces_empty() {
    let out = render_analysis_tree(&export(vec![
        row("src/a.rs", FileKind::Child, 10, 20),
        row("src/b.rs", FileKind::Child, 30, 60),
    ]));
    assert!(out.is_empty());
}

// ──────────────────────────────────────────────────────────────────────
// 5. Handoff tree tests
// ──────────────────────────────────────────────────────────────────────

#[test]
fn handoff_tree_empty_export_returns_empty() {
    assert!(render_handoff_tree(&export(vec![]), 3).is_empty());
}

#[test]
fn handoff_tree_shows_root_line() {
    let out = render_handoff_tree(
        &export(vec![row("src/lib.rs", FileKind::Parent, 10, 20)]),
        3,
    );
    assert!(out.contains("(root)"));
}

#[test]
fn handoff_tree_depth_0_shows_only_root() {
    let out = render_handoff_tree(
        &export(vec![row("src/lib.rs", FileKind::Parent, 10, 20)]),
        0,
    );
    assert!(out.contains("(root)"));
    assert!(!out.contains("src/"));
}

#[test]
fn handoff_tree_depth_limit_respected() {
    let out = render_handoff_tree(
        &export(vec![row("a/b/c/d.rs", FileKind::Parent, 10, 20)]),
        1,
    );
    assert!(out.contains("(root)"));
    assert!(out.contains("a/"));
    assert!(!out.contains("b/"));
}

#[test]
fn handoff_tree_no_file_leaves() {
    let out = render_handoff_tree(
        &export(vec![row("src/main.rs", FileKind::Parent, 10, 20)]),
        10,
    );
    // Should have (root) and src/ but not main.rs
    assert!(out.contains("(root)"));
    assert!(out.contains("src/"));
    assert!(!out.contains("main.rs"));
}

#[test]
fn handoff_tree_aggregates_file_counts() {
    let out = render_handoff_tree(
        &export(vec![
            row("src/a.rs", FileKind::Parent, 10, 20),
            row("src/b.rs", FileKind::Parent, 30, 60),
        ]),
        3,
    );
    assert!(out.contains("(root) (files: 2, lines: 40, tokens: 80)"));
    assert!(out.contains("src/ (files: 2, lines: 40, tokens: 80)"));
}

#[test]
fn handoff_tree_filters_child_rows() {
    let out = render_handoff_tree(
        &export(vec![
            row("src/a.rs", FileKind::Parent, 10, 20),
            row("src/a.rs::css", FileKind::Child, 5, 10),
        ]),
        3,
    );
    assert!(out.contains("(root) (files: 1, lines: 10, tokens: 20)"));
    assert!(!out.contains("css"));
}

#[test]
fn handoff_tree_deterministic() {
    let rows = vec![
        row("z/x.rs", FileKind::Parent, 10, 20),
        row("a/y.rs", FileKind::Parent, 30, 60),
    ];
    let out1 = render_handoff_tree(&export(rows.clone()), 5);
    let out2 = render_handoff_tree(&export(rows), 5);
    assert_eq!(out1, out2);
}

#[test]
fn handoff_tree_lexicographic_directory_order() {
    let out = render_handoff_tree(
        &export(vec![
            row("zzz/a.rs", FileKind::Parent, 1, 2),
            row("aaa/b.rs", FileKind::Parent, 3, 4),
        ]),
        3,
    );
    let lines: Vec<&str> = out.lines().collect();
    let aaa_pos = lines.iter().position(|l| l.contains("aaa/")).unwrap();
    let zzz_pos = lines.iter().position(|l| l.contains("zzz/")).unwrap();
    assert!(aaa_pos < zzz_pos);
}

#[test]
fn handoff_tree_flat_repo() {
    let out = render_handoff_tree(
        &export(vec![
            row("a.rs", FileKind::Parent, 10, 20),
            row("b.rs", FileKind::Parent, 20, 40),
        ]),
        3,
    );
    // Root only since files are at top level (no directory children)
    assert!(out.contains("(root) (files: 2, lines: 30, tokens: 60)"));
}
