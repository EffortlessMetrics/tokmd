//! Deep tests for `tokmd-export-tree`.
//!
//! Covers: analysis tree rendering, handoff tree rendering, edge cases,
//! aggregation correctness, path handling, determinism, and boundary conditions.

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
// 1. Analysis tree – basic rendering
// ===========================================================================

#[test]
fn analysis_tree_single_top_level_file() {
    let tree = render_analysis_tree(&export(vec![row("Cargo.toml", FileKind::Parent, 20, 40)]));
    assert_eq!(tree.trim(), "Cargo.toml (lines: 20, tokens: 40)");
}

#[test]
fn analysis_tree_two_siblings_sorted_lexicographically() {
    let tree = render_analysis_tree(&export(vec![
        row("b.rs", FileKind::Parent, 5, 10),
        row("a.rs", FileKind::Parent, 3, 6),
    ]));
    let lines: Vec<&str> = tree.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("a.rs"));
    assert!(lines[1].starts_with("b.rs"));
}

#[test]
fn analysis_tree_nested_dir_aggregates_children() {
    let tree = render_analysis_tree(&export(vec![
        row("src/a.rs", FileKind::Parent, 10, 20),
        row("src/b.rs", FileKind::Parent, 30, 60),
    ]));
    assert!(tree.contains("src (lines: 40, tokens: 80)"));
    assert!(tree.contains("a.rs (lines: 10, tokens: 20)"));
    assert!(tree.contains("b.rs (lines: 30, tokens: 60)"));
}

#[test]
fn analysis_tree_multi_level_aggregation() {
    // a/b/c.rs (10,20) and a/b/d.rs (5,15)
    // a should aggregate to (15,35), a/b should also be (15,35)
    let tree = render_analysis_tree(&export(vec![
        row("a/b/c.rs", FileKind::Parent, 10, 20),
        row("a/b/d.rs", FileKind::Parent, 5, 15),
    ]));
    assert!(tree.contains("a (lines: 15, tokens: 35)"));
    assert!(tree.contains("b (lines: 15, tokens: 35)"));
}

#[test]
fn analysis_tree_multiple_top_level_dirs() {
    let tree = render_analysis_tree(&export(vec![
        row("src/main.rs", FileKind::Parent, 50, 100),
        row("tests/test.rs", FileKind::Parent, 20, 40),
        row("benches/bench.rs", FileKind::Parent, 10, 20),
    ]));
    // All three dirs should appear in sorted order
    let benches_pos = tree.find("benches").unwrap();
    let src_pos = tree.find("src").unwrap();
    let tests_pos = tree.find("tests").unwrap();
    assert!(benches_pos < src_pos);
    assert!(src_pos < tests_pos);
}

#[test]
fn analysis_tree_indentation_per_depth() {
    let tree = render_analysis_tree(&export(vec![row("a/b/c/d.rs", FileKind::Parent, 1, 2)]));
    let lines: Vec<&str> = tree.lines().collect();
    assert_eq!(lines.len(), 4);
    assert!(lines[0].starts_with("a "));
    assert!(lines[1].starts_with("  b "));
    assert!(lines[2].starts_with("    c "));
    assert!(lines[3].starts_with("      d.rs "));
}

#[test]
fn analysis_tree_empty_export() {
    let tree = render_analysis_tree(&export(vec![]));
    assert!(tree.is_empty());
}

#[test]
fn analysis_tree_only_child_rows_produces_empty() {
    let tree = render_analysis_tree(&export(vec![
        row("src/a.rs::css", FileKind::Child, 10, 20),
        row("src/b.rs::js", FileKind::Child, 5, 10),
    ]));
    assert!(tree.is_empty());
}

// ===========================================================================
// 2. Analysis tree – file leaf presence
// ===========================================================================

#[test]
fn analysis_tree_shows_file_leaf_with_exact_counts() {
    let tree = render_analysis_tree(&export(vec![row("src/lib.rs", FileKind::Parent, 42, 84)]));
    assert!(tree.contains("lib.rs (lines: 42, tokens: 84)"));
}

#[test]
fn analysis_tree_mixed_parent_child_only_shows_parent() {
    let tree = render_analysis_tree(&export(vec![
        row("src/lib.rs", FileKind::Parent, 10, 20),
        row("src/lib.rs::html", FileKind::Child, 50, 100),
    ]));
    assert!(tree.contains("lib.rs (lines: 10, tokens: 20)"));
    assert!(!tree.contains("50"));
    assert!(!tree.contains("html"));
}

// ===========================================================================
// 3. Analysis tree – zero values
// ===========================================================================

#[test]
fn analysis_tree_zero_lines_zero_tokens() {
    let tree = render_analysis_tree(&export(vec![row("empty.rs", FileKind::Parent, 0, 0)]));
    assert!(tree.contains("empty.rs (lines: 0, tokens: 0)"));
}

#[test]
fn analysis_tree_zero_tokens_nonzero_lines() {
    let tree = render_analysis_tree(&export(vec![row("blank.rs", FileKind::Parent, 10, 0)]));
    assert!(tree.contains("blank.rs (lines: 10, tokens: 0)"));
}

// ===========================================================================
// 4. Handoff tree – basic rendering
// ===========================================================================

#[test]
fn handoff_tree_single_file_shows_root_only() {
    let tree = render_handoff_tree(&export(vec![row("main.rs", FileKind::Parent, 5, 10)]), 5);
    assert_eq!(tree.lines().count(), 1);
    assert!(tree.contains("(root) (files: 1, lines: 5, tokens: 10)"));
}

#[test]
fn handoff_tree_file_in_dir_shows_root_and_dir() {
    let tree = render_handoff_tree(
        &export(vec![row("src/main.rs", FileKind::Parent, 10, 20)]),
        5,
    );
    assert!(tree.contains("(root) (files: 1, lines: 10, tokens: 20)"));
    assert!(tree.contains("src/ (files: 1, lines: 10, tokens: 20)"));
    // file leaf should NOT appear
    assert!(!tree.contains("main.rs"));
}

#[test]
fn handoff_tree_no_file_leaves() {
    let tree = render_handoff_tree(
        &export(vec![
            row("a/b/file.rs", FileKind::Parent, 5, 10),
            row("a/c/other.rs", FileKind::Parent, 3, 6),
        ]),
        10,
    );
    assert!(!tree.contains("file.rs"));
    assert!(!tree.contains("other.rs"));
}

#[test]
fn handoff_tree_empty_export() {
    let tree = render_handoff_tree(&export(vec![]), 3);
    assert!(tree.is_empty());
}

#[test]
fn handoff_tree_only_child_rows_produces_empty() {
    let tree = render_handoff_tree(
        &export(vec![row("src/a.rs::css", FileKind::Child, 10, 20)]),
        3,
    );
    assert!(tree.is_empty());
}

// ===========================================================================
// 5. Handoff tree – depth limiting
// ===========================================================================

#[test]
fn handoff_tree_depth_zero_shows_only_root() {
    let tree = render_handoff_tree(
        &export(vec![
            row("a/b/c/d.rs", FileKind::Parent, 10, 20),
            row("x/y.rs", FileKind::Parent, 5, 10),
        ]),
        0,
    );
    assert_eq!(tree.lines().count(), 1);
    assert!(tree.contains("(root)"));
}

#[test]
fn handoff_tree_depth_one_shows_root_and_first_level() {
    let tree = render_handoff_tree(
        &export(vec![
            row("a/b/c.rs", FileKind::Parent, 10, 20),
            row("x/y.rs", FileKind::Parent, 5, 10),
        ]),
        1,
    );
    assert!(tree.contains("(root)"));
    assert!(tree.contains("a/"));
    assert!(tree.contains("x/"));
    assert!(!tree.contains("b/"));
    assert!(!tree.contains("y/"));
}

#[test]
fn handoff_tree_depth_boundary_exact() {
    // depth=2 should show root + level 1 + level 2, not level 3
    let tree = render_handoff_tree(&export(vec![row("a/b/c/d.rs", FileKind::Parent, 1, 2)]), 2);
    assert!(tree.contains("a/"));
    assert!(tree.contains("b/"));
    assert!(!tree.contains("c/"));
}

// ===========================================================================
// 6. Handoff tree – aggregation
// ===========================================================================

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
fn handoff_tree_nested_aggregation() {
    let tree = render_handoff_tree(
        &export(vec![
            row("a/x/file1.rs", FileKind::Parent, 10, 20),
            row("a/y/file2.rs", FileKind::Parent, 5, 10),
        ]),
        3,
    );
    assert!(tree.contains("a/ (files: 2, lines: 15, tokens: 30)"));
    assert!(tree.contains("x/ (files: 1, lines: 10, tokens: 20)"));
    assert!(tree.contains("y/ (files: 1, lines: 5, tokens: 10)"));
}

// ===========================================================================
// 7. Handoff tree – indentation
// ===========================================================================

#[test]
fn handoff_tree_indentation_increases() {
    let tree = render_handoff_tree(&export(vec![row("a/b/c/d.rs", FileKind::Parent, 1, 2)]), 5);
    let lines: Vec<&str> = tree.lines().collect();
    assert!(lines[0].starts_with("(root)"));
    assert!(lines[1].starts_with("  a/"));
    assert!(lines[2].starts_with("    b/"));
    assert!(lines[3].starts_with("      c/"));
}

// ===========================================================================
// 8. Determinism
// ===========================================================================

#[test]
fn analysis_tree_deterministic_across_calls() {
    let data = export(vec![
        row("z/z.rs", FileKind::Parent, 100, 200),
        row("a/a.rs", FileKind::Parent, 50, 100),
        row("m/m.rs", FileKind::Parent, 75, 150),
    ]);
    let t1 = render_analysis_tree(&data);
    let t2 = render_analysis_tree(&data);
    assert_eq!(t1, t2);
}

#[test]
fn handoff_tree_deterministic_across_calls() {
    let data = export(vec![
        row("z/z.rs", FileKind::Parent, 100, 200),
        row("a/a.rs", FileKind::Parent, 50, 100),
    ]);
    let t1 = render_handoff_tree(&data, 3);
    let t2 = render_handoff_tree(&data, 3);
    assert_eq!(t1, t2);
}

#[test]
fn analysis_tree_order_independent() {
    let forward = export(vec![
        row("a.rs", FileKind::Parent, 1, 2),
        row("b.rs", FileKind::Parent, 3, 6),
        row("c.rs", FileKind::Parent, 5, 10),
    ]);
    let reversed = export(vec![
        row("c.rs", FileKind::Parent, 5, 10),
        row("b.rs", FileKind::Parent, 3, 6),
        row("a.rs", FileKind::Parent, 1, 2),
    ]);
    assert_eq!(
        render_analysis_tree(&forward),
        render_analysis_tree(&reversed)
    );
}

#[test]
fn handoff_tree_order_independent() {
    let forward = export(vec![
        row("x/a.rs", FileKind::Parent, 10, 20),
        row("y/b.rs", FileKind::Parent, 5, 10),
    ]);
    let reversed = export(vec![
        row("y/b.rs", FileKind::Parent, 5, 10),
        row("x/a.rs", FileKind::Parent, 10, 20),
    ]);
    assert_eq!(
        render_handoff_tree(&forward, 3),
        render_handoff_tree(&reversed, 3)
    );
}

// ===========================================================================
// 9. Path edge cases
// ===========================================================================

#[test]
fn analysis_tree_leading_slash_stripped() {
    // Path segments are filtered for empty, so leading slash yields empty first segment
    let tree = render_analysis_tree(&export(vec![row("/src/main.rs", FileKind::Parent, 10, 20)]));
    // The empty segment from leading "/" is filtered out
    assert!(tree.contains("src"));
    assert!(tree.contains("main.rs"));
}

#[test]
fn analysis_tree_trailing_slash_handled() {
    // A path like "src/" would produce an empty last segment, filtered out
    let tree = render_analysis_tree(&export(vec![row("src/", FileKind::Parent, 5, 10)]));
    assert!(tree.contains("src (lines: 5, tokens: 10)"));
}

#[test]
fn analysis_tree_deeply_nested_6_levels() {
    let tree = render_analysis_tree(&export(vec![row("a/b/c/d/e/f.rs", FileKind::Parent, 1, 2)]));
    for seg in ["a", "b", "c", "d", "e", "f.rs"] {
        assert!(tree.contains(seg), "missing segment: {seg}");
    }
    assert_eq!(tree.lines().count(), 6);
}

// ===========================================================================
// 10. Large tree structure
// ===========================================================================

#[test]
fn analysis_tree_many_files_same_directory() {
    let rows: Vec<FileRow> = (0..20)
        .map(|i| {
            row(
                &format!("src/file_{i:02}.rs"),
                FileKind::Parent,
                i + 1,
                (i + 1) * 2,
            )
        })
        .collect();
    let expected_lines: usize = (1..=20).sum();
    let expected_tokens: usize = (1..=20).map(|i| i * 2).sum();
    let tree = render_analysis_tree(&export(rows));
    assert!(tree.contains(&format!(
        "src (lines: {expected_lines}, tokens: {expected_tokens})"
    )));
    assert_eq!(tree.lines().count(), 21); // 1 dir + 20 files
}

#[test]
fn handoff_tree_many_directories() {
    let rows: Vec<FileRow> = (0..10)
        .map(|i| row(&format!("dir_{i:02}/file.rs"), FileKind::Parent, 10, 20))
        .collect();
    let tree = render_handoff_tree(&export(rows), 2);
    assert!(tree.contains("(root) (files: 10, lines: 100, tokens: 200)"));
    for i in 0..10 {
        assert!(tree.contains(&format!("dir_{i:02}/")));
    }
}

// ===========================================================================
// 11. Output format
// ===========================================================================

#[test]
fn analysis_tree_nonempty_output_ends_with_newline() {
    let tree = render_analysis_tree(&export(vec![row("a.rs", FileKind::Parent, 1, 2)]));
    assert!(tree.ends_with('\n'));
}

#[test]
fn handoff_tree_nonempty_output_ends_with_newline() {
    let tree = render_handoff_tree(&export(vec![row("a/b.rs", FileKind::Parent, 1, 2)]), 3);
    assert!(tree.ends_with('\n'));
}

#[test]
fn analysis_tree_format_matches_expected_pattern() {
    let tree = render_analysis_tree(&export(vec![row(
        "src/main.rs",
        FileKind::Parent,
        100,
        250,
    )]));
    // Each line must match "INDENT NAME (lines: N, tokens: N)"
    for line in tree.lines() {
        let trimmed = line.trim();
        assert!(
            trimmed.contains("(lines:") && trimmed.contains("tokens:"),
            "line does not match expected format: {line:?}"
        );
    }
}

#[test]
fn handoff_tree_format_dirs_have_trailing_slash() {
    let tree = render_handoff_tree(
        &export(vec![row("src/lib/mod.rs", FileKind::Parent, 5, 10)]),
        5,
    );
    // All non-root lines should have trailing slash (they're directories)
    for line in tree.lines().skip(1) {
        let trimmed = line.trim();
        let name_part = trimmed.split(" (").next().unwrap();
        assert!(
            name_part.ends_with('/'),
            "directory should have trailing slash: {name_part:?}"
        );
    }
}

// ===========================================================================
// 12. Handoff tree – root line format
// ===========================================================================

#[test]
fn handoff_tree_root_always_says_root() {
    let tree = render_handoff_tree(&export(vec![row("a.rs", FileKind::Parent, 1, 2)]), 1);
    let first = tree.lines().next().unwrap();
    assert!(first.starts_with("(root)"));
}

#[test]
fn handoff_tree_root_files_count_matches_parent_rows() {
    let rows = vec![
        row("a.rs", FileKind::Parent, 1, 2),
        row("b.rs", FileKind::Parent, 3, 6),
        row("c.rs", FileKind::Parent, 5, 10),
        row("d.rs::css", FileKind::Child, 100, 200), // not counted
    ];
    let tree = render_handoff_tree(&export(rows), 1);
    assert!(tree.contains("(root) (files: 3, lines: 9, tokens: 18)"));
}

// ===========================================================================
// 13. Cross-validation between analysis and handoff trees
// ===========================================================================

#[test]
fn both_trees_agree_on_total_lines_and_tokens() {
    let data = export(vec![
        row("a/x.rs", FileKind::Parent, 10, 20),
        row("b/y.rs", FileKind::Parent, 30, 60),
    ]);
    let analysis = render_analysis_tree(&data);
    let handoff = render_handoff_tree(&data, 1);

    // Analysis tree: top-level dirs a and b individually sum to total
    assert!(analysis.contains("a (lines: 10, tokens: 20)"));
    assert!(analysis.contains("b (lines: 30, tokens: 60)"));

    // Handoff tree root should reflect the total
    assert!(handoff.contains("(root) (files: 2, lines: 40, tokens: 80)"));
}

#[test]
fn analysis_tree_shows_file_leaves_handoff_does_not() {
    let data = export(vec![row("src/main.rs", FileKind::Parent, 10, 20)]);
    let analysis = render_analysis_tree(&data);
    let handoff = render_handoff_tree(&data, 5);

    assert!(
        analysis.contains("main.rs"),
        "analysis should show file leaf"
    );
    assert!(
        !handoff.contains("main.rs"),
        "handoff should NOT show file leaf"
    );
}
