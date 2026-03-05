//! W59 – Tree construction, depth limiting, sorting, determinism.

use tokmd_export_tree::{render_analysis_tree, render_handoff_tree};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ──────────────────────────────────────────────────────────

fn row(path: &str, kind: FileKind, lines: usize, tokens: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: path
            .split('/')
            .next()
            .map_or_else(|| "(root)".to_string(), String::from),
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

fn parent(path: &str, lines: usize, tokens: usize) -> FileRow {
    row(path, FileKind::Parent, lines, tokens)
}

fn child(path: &str, lines: usize, tokens: usize) -> FileRow {
    row(path, FileKind::Child, lines, tokens)
}

fn export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::ParentsOnly,
    }
}

// ── Empty input ──────────────────────────────────────────────────────

#[test]
fn analysis_tree_empty_rows_returns_empty() {
    assert!(render_analysis_tree(&export(vec![])).is_empty());
}

#[test]
fn handoff_tree_empty_rows_returns_empty() {
    assert!(render_handoff_tree(&export(vec![]), 5).is_empty());
}

// ── Single file ──────────────────────────────────────────────────────

#[test]
fn analysis_tree_single_root_file() {
    let tree = render_analysis_tree(&export(vec![parent("README.md", 10, 20)]));
    assert_eq!(tree.trim(), "README.md (lines: 10, tokens: 20)");
    assert_eq!(tree.lines().count(), 1);
}

#[test]
fn handoff_tree_single_root_file() {
    let tree = render_handoff_tree(&export(vec![parent("README.md", 10, 20)]), 5);
    assert_eq!(tree.trim(), "(root) (files: 1, lines: 10, tokens: 20)");
    assert_eq!(tree.lines().count(), 1);
}

// ── Child filtering ──────────────────────────────────────────────────

#[test]
fn analysis_tree_ignores_child_kind_rows() {
    let data = export(vec![
        parent("src/main.rs", 50, 100),
        child("src/main.rs::inline", 20, 40),
    ]);
    let tree = render_analysis_tree(&data);
    assert!(tree.contains("main.rs (lines: 50, tokens: 100)"));
    assert!(!tree.contains("inline"));
}

#[test]
fn handoff_tree_ignores_child_kind_rows() {
    let data = export(vec![
        parent("src/main.rs", 50, 100),
        child("src/main.rs::css", 5, 10),
    ]);
    let tree = render_handoff_tree(&data, 5);
    assert!(tree.contains("(root) (files: 1, lines: 50, tokens: 100)"));
}

#[test]
fn only_child_rows_produce_empty_trees() {
    let data = export(vec![
        child("src/a.rs::embedded", 10, 20),
        child("src/b.rs::embedded", 30, 60),
    ]);
    assert!(render_analysis_tree(&data).is_empty());
    assert!(render_handoff_tree(&data, 5).is_empty());
}

// ── Aggregation ──────────────────────────────────────────────────────

#[test]
fn analysis_tree_aggregates_parent_lines_and_tokens() {
    let data = export(vec![parent("src/a.rs", 10, 20), parent("src/b.rs", 30, 60)]);
    let tree = render_analysis_tree(&data);
    assert!(tree.contains("src (lines: 40, tokens: 80)"));
}

#[test]
fn handoff_tree_aggregates_file_count_lines_tokens() {
    let data = export(vec![
        parent("src/a.rs", 10, 20),
        parent("src/b.rs", 30, 60),
        parent("src/c.rs", 5, 10),
    ]);
    let tree = render_handoff_tree(&data, 5);
    let root_line = tree.lines().next().unwrap();
    assert_eq!(root_line, "(root) (files: 3, lines: 45, tokens: 90)");
}

#[test]
fn analysis_tree_nested_aggregation() {
    let data = export(vec![
        parent("crates/a/src/lib.rs", 100, 200),
        parent("crates/a/src/util.rs", 50, 100),
        parent("crates/b/src/lib.rs", 30, 60),
    ]);
    let tree = render_analysis_tree(&data);
    assert!(tree.contains("crates (lines: 180, tokens: 360)"));
    assert!(tree.contains("a (lines: 150, tokens: 300)"));
    assert!(tree.contains("b (lines: 30, tokens: 60)"));
}

// ── Depth limiting (handoff) ─────────────────────────────────────────

#[test]
fn handoff_depth_zero_emits_only_root() {
    let data = export(vec![parent("a/b/c/d.rs", 10, 20)]);
    let tree = render_handoff_tree(&data, 0);
    assert_eq!(tree.lines().count(), 1);
    assert!(tree.starts_with("(root)"));
}

#[test]
fn handoff_depth_one_shows_first_level_dir() {
    let data = export(vec![parent("a/b/c/d.rs", 10, 20)]);
    let tree = render_handoff_tree(&data, 1);
    assert!(tree.contains("a/"));
    assert!(!tree.contains("b/"));
}

#[test]
fn handoff_depth_two_shows_two_levels() {
    let data = export(vec![parent("a/b/c/d.rs", 10, 20)]);
    let tree = render_handoff_tree(&data, 2);
    assert!(tree.contains("a/"));
    assert!(tree.contains("b/"));
    assert!(!tree.contains("c/"));
}

#[test]
fn handoff_large_depth_shows_all_dirs() {
    let data = export(vec![parent("a/b/c/d.rs", 10, 20)]);
    let tree = render_handoff_tree(&data, 100);
    assert!(tree.contains("a/"));
    assert!(tree.contains("b/"));
    assert!(tree.contains("c/"));
    // File leaf never appears in handoff tree
    assert!(!tree.contains("d.rs"));
}

// ── Deterministic ordering ───────────────────────────────────────────

#[test]
fn analysis_tree_lexicographic_sibling_order() {
    let data = export(vec![
        parent("src/z.rs", 1, 1),
        parent("src/a.rs", 1, 1),
        parent("src/m.rs", 1, 1),
    ]);
    let tree = render_analysis_tree(&data);
    let names: Vec<&str> = tree
        .lines()
        .filter(|l| l.contains(".rs"))
        .map(|l| l.trim().split(' ').next().unwrap())
        .collect();
    assert_eq!(names, vec!["a.rs", "m.rs", "z.rs"]);
}

#[test]
fn handoff_tree_lexicographic_sibling_order() {
    let data = export(vec![
        parent("z/file.rs", 1, 1),
        parent("a/file.rs", 1, 1),
        parent("m/file.rs", 1, 1),
    ]);
    let tree = render_handoff_tree(&data, 5);
    let dirs: Vec<&str> = tree
        .lines()
        .filter(|l| {
            l.trim().ends_with('/')
                || l.trim().starts_with("a/")
                || l.trim().starts_with("m/")
                || l.trim().starts_with("z/")
        })
        .map(|l| l.trim().split(' ').next().unwrap())
        .collect();
    assert_eq!(dirs, vec!["a/", "m/", "z/"]);
}

#[test]
fn reversed_input_order_produces_identical_output() {
    let rows = vec![parent("b/x.rs", 10, 20), parent("a/y.rs", 30, 60)];
    let mut rev = rows.clone();
    rev.reverse();
    assert_eq!(
        render_analysis_tree(&export(rows.clone())),
        render_analysis_tree(&export(rev.clone())),
    );
    assert_eq!(
        render_handoff_tree(&export(rows), 5),
        render_handoff_tree(&export(rev), 5),
    );
}

// ── Indentation ──────────────────────────────────────────────────────

#[test]
fn analysis_tree_indentation_per_depth() {
    let data = export(vec![parent("a/b/c.rs", 5, 10)]);
    let tree = render_analysis_tree(&data);
    let lines: Vec<&str> = tree.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("a "));
    assert!(lines[1].starts_with("  b "));
    assert!(lines[2].starts_with("    c.rs "));
}

#[test]
fn handoff_tree_indentation_per_depth() {
    let data = export(vec![parent("a/b/c/d.rs", 5, 10)]);
    let tree = render_handoff_tree(&data, 10);
    let lines: Vec<&str> = tree.lines().collect();
    assert!(lines[0].starts_with("(root)"));
    assert!(lines[1].starts_with("  a/"));
    assert!(lines[2].starts_with("    b/"));
    assert!(lines[3].starts_with("      c/"));
}

// ── Zero-value rows ─────────────────────────────────────────────────

#[test]
fn zero_lines_zero_tokens_still_rendered() {
    let data = export(vec![parent("src/empty.rs", 0, 0)]);
    let tree = render_analysis_tree(&data);
    assert!(tree.contains("empty.rs (lines: 0, tokens: 0)"));
}

#[test]
fn handoff_zero_values_in_root() {
    let data = export(vec![parent("src/empty.rs", 0, 0)]);
    let tree = render_handoff_tree(&data, 5);
    assert!(tree.contains("(root) (files: 1, lines: 0, tokens: 0)"));
}

// ── Multiple top-level dirs ──────────────────────────────────────────

#[test]
fn analysis_tree_multiple_top_dirs_aggregated_independently() {
    let data = export(vec![
        parent("crates/a/lib.rs", 100, 200),
        parent("packages/b/index.ts", 50, 100),
    ]);
    let tree = render_analysis_tree(&data);
    assert!(tree.contains("crates (lines: 100, tokens: 200)"));
    assert!(tree.contains("packages (lines: 50, tokens: 100)"));
}

#[test]
fn handoff_tree_multiple_top_dirs() {
    let data = export(vec![
        parent("crates/a/lib.rs", 100, 200),
        parent("packages/b/index.ts", 50, 100),
    ]);
    let tree = render_handoff_tree(&data, 3);
    assert!(tree.contains("crates/"));
    assert!(tree.contains("packages/"));
}

// ── Handoff never shows file leaves ──────────────────────────────────

#[test]
fn handoff_tree_never_shows_file_names() {
    let data = export(vec![
        parent("src/main.rs", 10, 20),
        parent("src/lib.rs", 20, 40),
        parent("tests/test_a.rs", 5, 10),
    ]);
    let tree = render_handoff_tree(&data, 10);
    assert!(!tree.contains("main.rs"));
    assert!(!tree.contains("lib.rs"));
    assert!(!tree.contains("test_a.rs"));
}

// ── Large tree ───────────────────────────────────────────────────────

#[test]
fn large_tree_aggregation_is_consistent() {
    let rows: Vec<FileRow> = (0..100)
        .map(|i| {
            parent(
                &format!("dir{}/sub{}/file{}.rs", i % 5, i % 10, i),
                i,
                i * 2,
            )
        })
        .collect();
    let total_lines: usize = rows.iter().map(|r| r.lines).sum();
    let total_tokens: usize = rows.iter().map(|r| r.tokens).sum();

    let handoff = render_handoff_tree(&export(rows), 10);
    let root_line = handoff.lines().next().unwrap();
    assert!(root_line.contains(&format!(
        "files: 100, lines: {total_lines}, tokens: {total_tokens}"
    )));
}
