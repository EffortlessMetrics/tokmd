//! Deep export-tree tests (w48): tree construction, traversal, sorting,
//! property-based verification, and edge cases.

use proptest::prelude::*;
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
// 1. Export tree construction from file rows
// ===========================================================================

#[test]
fn analysis_tree_single_file_at_root() {
    let tree = render_analysis_tree(&export(vec![row("main.rs", FileKind::Parent, 50, 100)]));
    assert_eq!(tree.trim(), "main.rs (lines: 50, tokens: 100)");
}

#[test]
fn analysis_tree_file_in_directory() {
    let tree = render_analysis_tree(&export(vec![row("src/lib.rs", FileKind::Parent, 30, 60)]));
    assert!(tree.contains("src (lines: 30, tokens: 60)"));
    assert!(tree.contains("lib.rs (lines: 30, tokens: 60)"));
}

#[test]
fn analysis_tree_multiple_files_same_dir() {
    let tree = render_analysis_tree(&export(vec![
        row("src/a.rs", FileKind::Parent, 10, 20),
        row("src/b.rs", FileKind::Parent, 30, 60),
    ]));
    assert!(tree.contains("src (lines: 40, tokens: 80)"));
    assert!(tree.contains("a.rs (lines: 10, tokens: 20)"));
    assert!(tree.contains("b.rs (lines: 30, tokens: 60)"));
}

#[test]
fn analysis_tree_deep_nesting() {
    let tree = render_analysis_tree(&export(vec![row("a/b/c/d/e.rs", FileKind::Parent, 5, 10)]));
    let lines: Vec<&str> = tree.lines().collect();
    assert_eq!(lines.len(), 5);
    assert!(lines[0].starts_with("a "));
    assert!(lines[1].starts_with("  b "));
    assert!(lines[2].starts_with("    c "));
    assert!(lines[3].starts_with("      d "));
    assert!(lines[4].starts_with("        e.rs "));
}

#[test]
fn analysis_tree_child_rows_excluded() {
    let tree = render_analysis_tree(&export(vec![
        row("src/main.rs", FileKind::Parent, 20, 40),
        row("src/main.rs::css", FileKind::Child, 100, 200),
    ]));
    assert!(tree.contains("main.rs (lines: 20, tokens: 40)"));
    assert!(!tree.contains("css"));
    assert!(!tree.contains("100"));
}

#[test]
fn handoff_tree_single_file_root_only() {
    let tree = render_handoff_tree(&export(vec![row("main.rs", FileKind::Parent, 10, 20)]), 5);
    assert_eq!(tree.lines().count(), 1);
    assert!(tree.contains("(root) (files: 1, lines: 10, tokens: 20)"));
}

#[test]
fn handoff_tree_file_in_dir() {
    let tree = render_handoff_tree(
        &export(vec![row("src/main.rs", FileKind::Parent, 10, 20)]),
        5,
    );
    assert!(tree.contains("(root) (files: 1"));
    assert!(tree.contains("src/ (files: 1"));
    assert!(!tree.contains("main.rs"));
}

#[test]
fn handoff_tree_no_file_leaves() {
    let tree = render_handoff_tree(
        &export(vec![
            row("a/x.rs", FileKind::Parent, 5, 10),
            row("b/y.rs", FileKind::Parent, 3, 6),
        ]),
        10,
    );
    assert!(!tree.contains("x.rs"));
    assert!(!tree.contains("y.rs"));
}

// ===========================================================================
// 2. Tree traversal and flattening
// ===========================================================================

#[test]
fn analysis_tree_aggregates_nested_dirs() {
    let tree = render_analysis_tree(&export(vec![
        row("a/b/c.rs", FileKind::Parent, 10, 20),
        row("a/b/d.rs", FileKind::Parent, 5, 15),
    ]));
    assert!(tree.contains("a (lines: 15, tokens: 35)"));
    assert!(tree.contains("b (lines: 15, tokens: 35)"));
}

#[test]
fn handoff_tree_aggregates_files_lines_tokens() {
    let tree = render_handoff_tree(
        &export(vec![
            row("src/a.rs", FileKind::Parent, 10, 20),
            row("src/b.rs", FileKind::Parent, 30, 60),
            row("lib/c.rs", FileKind::Parent, 5, 10),
        ]),
        3,
    );
    assert!(tree.contains("(root) (files: 3, lines: 45, tokens: 90)"));
    assert!(tree.contains("src/ (files: 2, lines: 40, tokens: 80)"));
    assert!(tree.contains("lib/ (files: 1, lines: 5, tokens: 10)"));
}

#[test]
fn analysis_tree_multiple_top_dirs() {
    let tree = render_analysis_tree(&export(vec![
        row("src/a.rs", FileKind::Parent, 10, 20),
        row("tests/b.rs", FileKind::Parent, 5, 10),
        row("benches/c.rs", FileKind::Parent, 3, 6),
    ]));
    let src_pos = tree.find("src").unwrap();
    let tests_pos = tree.find("tests").unwrap();
    let benches_pos = tree.find("benches").unwrap();
    assert!(benches_pos < src_pos);
    assert!(src_pos < tests_pos);
}

// ===========================================================================
// 3. Sorting by code/name (lexicographic sibling order)
// ===========================================================================

#[test]
fn analysis_tree_siblings_sorted_lexicographically() {
    let tree = render_analysis_tree(&export(vec![
        row("z.rs", FileKind::Parent, 1, 2),
        row("a.rs", FileKind::Parent, 3, 6),
        row("m.rs", FileKind::Parent, 5, 10),
    ]));
    let lines: Vec<&str> = tree.lines().collect();
    assert!(lines[0].starts_with("a.rs"));
    assert!(lines[1].starts_with("m.rs"));
    assert!(lines[2].starts_with("z.rs"));
}

#[test]
fn analysis_tree_deterministic_across_calls() {
    let data = export(vec![
        row("z/z.rs", FileKind::Parent, 100, 200),
        row("a/a.rs", FileKind::Parent, 50, 100),
    ]);
    assert_eq!(render_analysis_tree(&data), render_analysis_tree(&data));
}

#[test]
fn analysis_tree_input_order_independent() {
    let fwd = export(vec![
        row("a.rs", FileKind::Parent, 1, 2),
        row("b.rs", FileKind::Parent, 3, 6),
    ]);
    let rev = export(vec![
        row("b.rs", FileKind::Parent, 3, 6),
        row("a.rs", FileKind::Parent, 1, 2),
    ]);
    assert_eq!(render_analysis_tree(&fwd), render_analysis_tree(&rev));
}

#[test]
fn handoff_tree_input_order_independent() {
    let fwd = export(vec![
        row("x/a.rs", FileKind::Parent, 10, 20),
        row("y/b.rs", FileKind::Parent, 5, 10),
    ]);
    let rev = export(vec![
        row("y/b.rs", FileKind::Parent, 5, 10),
        row("x/a.rs", FileKind::Parent, 10, 20),
    ]);
    assert_eq!(render_handoff_tree(&fwd, 3), render_handoff_tree(&rev, 3));
}

// ===========================================================================
// 4. Property test: tree contains all input rows
// ===========================================================================

fn path_segment() -> impl Strategy<Value = String> {
    "[a-z]{1,6}".prop_map(String::from)
}

fn filename() -> impl Strategy<Value = String> {
    "[a-z]{1,6}\\.rs".prop_map(String::from)
}

fn path_strategy() -> impl Strategy<Value = String> {
    (prop::collection::vec(path_segment(), 0..3), filename()).prop_map(|(dirs, file)| {
        if dirs.is_empty() {
            file
        } else {
            format!("{}/{}", dirs.join("/"), file)
        }
    })
}

proptest! {
    #[test]
    fn prop_analysis_tree_contains_all_parent_filenames(
        paths in prop::collection::vec(path_strategy(), 1..20),
    ) {
        let rows: Vec<FileRow> = paths
            .iter()
            .enumerate()
            .map(|(i, p)| row(p, FileKind::Parent, i + 1, (i + 1) * 2))
            .collect();
        let tree = render_analysis_tree(&export(rows));
        for p in &paths {
            let filename = p.rsplit('/').next().unwrap();
            prop_assert!(
                tree.contains(filename),
                "tree missing filename {filename} from path {p}"
            );
        }
    }

    #[test]
    fn prop_handoff_root_file_count_matches(
        n in 1usize..30,
    ) {
        let rows: Vec<FileRow> = (0..n)
            .map(|i| row(&format!("d/f{i}.rs"), FileKind::Parent, 10, 20))
            .collect();
        let tree = render_handoff_tree(&export(rows), 10);
        let expected_prefix = format!("(root) (files: {n},");
        prop_assert!(
            tree.starts_with(&expected_prefix),
            "expected root to start with '{expected_prefix}', got '{}'",
            tree.lines().next().unwrap_or("")
        );
    }

    #[test]
    fn prop_analysis_deterministic(
        rows in prop::collection::vec(path_strategy(), 0..30),
    ) {
        let file_rows: Vec<FileRow> = rows
            .iter()
            .enumerate()
            .map(|(i, p)| row(p, FileKind::Parent, i + 1, i + 2))
            .collect();
        let data = export(file_rows);
        prop_assert_eq!(render_analysis_tree(&data), render_analysis_tree(&data));
    }

    #[test]
    fn prop_handoff_depth_zero_single_line(
        n in 1usize..20,
    ) {
        let rows: Vec<FileRow> = (0..n)
            .map(|i| row(&format!("d{i}/f.rs"), FileKind::Parent, 1, 2))
            .collect();
        let tree = render_handoff_tree(&export(rows), 0);
        prop_assert_eq!(tree.lines().count(), 1);
    }
}

// ===========================================================================
// 5. Edge cases
// ===========================================================================

#[test]
fn analysis_tree_empty_export() {
    assert!(render_analysis_tree(&export(vec![])).is_empty());
}

#[test]
fn handoff_tree_empty_export() {
    assert!(render_handoff_tree(&export(vec![]), 3).is_empty());
}

#[test]
fn analysis_tree_only_child_rows_empty() {
    let tree = render_analysis_tree(&export(vec![row("a.rs::css", FileKind::Child, 10, 20)]));
    assert!(tree.is_empty());
}

#[test]
fn handoff_tree_only_child_rows_empty() {
    let tree = render_handoff_tree(&export(vec![row("a.rs::css", FileKind::Child, 10, 20)]), 3);
    assert!(tree.is_empty());
}

#[test]
fn analysis_tree_zero_lines_and_tokens() {
    let tree = render_analysis_tree(&export(vec![row("empty.rs", FileKind::Parent, 0, 0)]));
    assert!(tree.contains("empty.rs (lines: 0, tokens: 0)"));
}

#[test]
fn handoff_tree_depth_limit_respected() {
    let tree = render_handoff_tree(&export(vec![row("a/b/c/d.rs", FileKind::Parent, 1, 2)]), 1);
    assert!(tree.contains("a/"));
    assert!(!tree.contains("b/"));
}

#[test]
fn analysis_tree_nonempty_ends_with_newline() {
    let tree = render_analysis_tree(&export(vec![row("a.rs", FileKind::Parent, 1, 2)]));
    assert!(tree.ends_with('\n'));
}

#[test]
fn handoff_tree_nonempty_ends_with_newline() {
    let tree = render_handoff_tree(&export(vec![row("a/b.rs", FileKind::Parent, 1, 2)]), 3);
    assert!(tree.ends_with('\n'));
}

#[test]
fn analysis_tree_leading_slash_handled() {
    let tree = render_analysis_tree(&export(vec![row("/src/main.rs", FileKind::Parent, 10, 20)]));
    assert!(tree.contains("src"));
    assert!(tree.contains("main.rs"));
}

#[test]
fn handoff_tree_many_dirs_root_aggregates() {
    let rows: Vec<FileRow> = (0..10)
        .map(|i| row(&format!("dir_{i:02}/file.rs"), FileKind::Parent, 10, 20))
        .collect();
    let tree = render_handoff_tree(&export(rows), 2);
    assert!(tree.contains("(root) (files: 10, lines: 100, tokens: 200)"));
}

#[test]
fn both_trees_agree_on_totals() {
    let data = export(vec![
        row("a/x.rs", FileKind::Parent, 10, 20),
        row("b/y.rs", FileKind::Parent, 30, 60),
    ]);
    let analysis = render_analysis_tree(&data);
    let handoff = render_handoff_tree(&data, 1);
    assert!(analysis.contains("a (lines: 10, tokens: 20)"));
    assert!(analysis.contains("b (lines: 30, tokens: 60)"));
    assert!(handoff.contains("(root) (files: 2, lines: 40, tokens: 80)"));
}
