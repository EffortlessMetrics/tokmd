//! Wave 61: Depth tests for `tokmd-export-tree`.
//!
//! Covers: tree building with shared prefixes, depth-limit precision,
//! aggregation arithmetic, sorting invariants, path edge cases,
//! cross-tree validation, and stress scenarios.

use tokmd_export_tree::{render_analysis_tree, render_handoff_tree};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── helpers ─────────────────────────────────────────────────────

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
// 1. Analysis tree — shared prefix aggregation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_analysis_shared_prefix_two_files_same_dir() {
    let tree = render_analysis_tree(&export(vec![
        row("src/a.rs", FileKind::Parent, 10, 20),
        row("src/b.rs", FileKind::Parent, 20, 40),
    ]));
    assert!(tree.contains("src (lines: 30, tokens: 60)"));
}

#[test]
fn w61_analysis_shared_prefix_three_levels() {
    let tree = render_analysis_tree(&export(vec![
        row("a/b/c/x.rs", FileKind::Parent, 5, 10),
        row("a/b/c/y.rs", FileKind::Parent, 15, 30),
    ]));
    assert!(tree.contains("a (lines: 20, tokens: 40)"));
    assert!(tree.contains("b (lines: 20, tokens: 40)"));
    assert!(tree.contains("c (lines: 20, tokens: 40)"));
}

#[test]
fn w61_analysis_diverging_paths_from_shared_root() {
    let tree = render_analysis_tree(&export(vec![
        row("crates/alpha/src/lib.rs", FileKind::Parent, 10, 20),
        row("crates/beta/src/lib.rs", FileKind::Parent, 30, 60),
    ]));
    assert!(tree.contains("crates (lines: 40, tokens: 80)"));
    assert!(tree.contains("alpha (lines: 10, tokens: 20)"));
    assert!(tree.contains("beta (lines: 30, tokens: 60)"));
}

#[test]
fn w61_analysis_sibling_dirs_isolated_aggregation() {
    let tree = render_analysis_tree(&export(vec![
        row("a/file.rs", FileKind::Parent, 10, 20),
        row("b/file.rs", FileKind::Parent, 30, 60),
    ]));
    assert!(tree.contains("a (lines: 10, tokens: 20)"));
    assert!(tree.contains("b (lines: 30, tokens: 60)"));
}

// ═══════════════════════════════════════════════════════════════════
// 2. Handoff tree — depth-limit precision
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_handoff_depth_0_only_root() {
    let tree = render_handoff_tree(
        &export(vec![row("a/b/c/d/e.rs", FileKind::Parent, 1, 2)]),
        0,
    );
    assert_eq!(tree.lines().count(), 1);
    assert!(tree.contains("(root)"));
}

#[test]
fn w61_handoff_depth_1_root_plus_first_level() {
    let tree = render_handoff_tree(&export(vec![row("a/b/c.rs", FileKind::Parent, 5, 10)]), 1);
    assert!(tree.contains("(root)"));
    assert!(tree.contains("a/"));
    assert!(!tree.contains("b/"));
}

#[test]
fn w61_handoff_depth_2_shows_two_levels() {
    let tree = render_handoff_tree(&export(vec![row("a/b/c/d.rs", FileKind::Parent, 1, 2)]), 2);
    assert!(tree.contains("a/"));
    assert!(tree.contains("b/"));
    assert!(!tree.contains("c/"));
}

#[test]
fn w61_handoff_depth_3_shows_three_levels() {
    let tree = render_handoff_tree(
        &export(vec![row("a/b/c/d/e.rs", FileKind::Parent, 1, 2)]),
        3,
    );
    assert!(tree.contains("a/"));
    assert!(tree.contains("b/"));
    assert!(tree.contains("c/"));
    assert!(!tree.contains("d/"));
}

#[test]
fn w61_handoff_very_large_depth_shows_all_dirs() {
    let tree = render_handoff_tree(
        &export(vec![row("a/b/c/d/e.rs", FileKind::Parent, 1, 2)]),
        1000,
    );
    assert!(tree.contains("a/"));
    assert!(tree.contains("b/"));
    assert!(tree.contains("c/"));
    assert!(tree.contains("d/"));
    assert!(!tree.contains("e.rs"));
}

// ═══════════════════════════════════════════════════════════════════
// 3. Analysis tree — sorting invariants
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_analysis_sorting_case_sensitive() {
    let tree = render_analysis_tree(&export(vec![
        row("Zebra.rs", FileKind::Parent, 1, 2),
        row("alpha.rs", FileKind::Parent, 1, 2),
    ]));
    let lines: Vec<&str> = tree.lines().collect();
    // Uppercase comes before lowercase in ASCII order
    assert!(lines[0].starts_with("Zebra.rs"));
    assert!(lines[1].starts_with("alpha.rs"));
}

#[test]
fn w61_analysis_sorting_numeric_names() {
    let tree = render_analysis_tree(&export(vec![
        row("file_10.rs", FileKind::Parent, 1, 2),
        row("file_2.rs", FileKind::Parent, 1, 2),
        row("file_1.rs", FileKind::Parent, 1, 2),
    ]));
    let lines: Vec<&str> = tree.lines().collect();
    // Lexicographic: file_1 < file_10 < file_2
    assert!(lines[0].starts_with("file_1.rs"));
    assert!(lines[1].starts_with("file_10.rs"));
    assert!(lines[2].starts_with("file_2.rs"));
}

#[test]
fn w61_analysis_sorting_mixed_files_and_dirs() {
    let tree = render_analysis_tree(&export(vec![
        row("z_dir/f.rs", FileKind::Parent, 1, 2),
        row("a_file.rs", FileKind::Parent, 1, 2),
    ]));
    let lines: Vec<&str> = tree.lines().collect();
    // a_file.rs is a top-level entry, z_dir is a top-level entry
    // "a_file.rs" < "z_dir" lexicographically
    assert!(lines[0].starts_with("a_file.rs"));
    assert!(lines[1].starts_with("z_dir"));
}

#[test]
fn w61_handoff_sorting_dirs_lexicographic() {
    let tree = render_handoff_tree(
        &export(vec![
            row("m/f.rs", FileKind::Parent, 1, 2),
            row("a/f.rs", FileKind::Parent, 1, 2),
            row("z/f.rs", FileKind::Parent, 1, 2),
        ]),
        2,
    );
    let dir_lines: Vec<&str> = tree.lines().skip(1).collect();
    assert!(dir_lines[0].contains("a/"));
    assert!(dir_lines[1].contains("m/"));
    assert!(dir_lines[2].contains("z/"));
}

// ═══════════════════════════════════════════════════════════════════
// 4. Analysis tree — child row filtering
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_analysis_many_child_rows_all_filtered() {
    let rows: Vec<FileRow> = (0..20)
        .map(|i| {
            row(
                &format!("src/file_{i}.rs::embedded"),
                FileKind::Child,
                100,
                200,
            )
        })
        .collect();
    assert!(render_analysis_tree(&export(rows)).is_empty());
}

#[test]
fn w61_analysis_mixed_parent_child_only_counts_parent() {
    let tree = render_analysis_tree(&export(vec![
        row("src/a.rs", FileKind::Parent, 10, 20),
        row("src/a.rs::css", FileKind::Child, 100, 200),
        row("src/a.rs::js", FileKind::Child, 50, 100),
    ]));
    assert!(tree.contains("src (lines: 10, tokens: 20)"));
    assert!(tree.contains("a.rs (lines: 10, tokens: 20)"));
}

#[test]
fn w61_handoff_child_rows_not_counted_in_root() {
    let tree = render_handoff_tree(
        &export(vec![
            row("src/a.rs", FileKind::Parent, 10, 20),
            row("src/a.rs::css", FileKind::Child, 100, 200),
        ]),
        3,
    );
    assert!(tree.contains("(root) (files: 1, lines: 10, tokens: 20)"));
}

// ═══════════════════════════════════════════════════════════════════
// 5. Path edge cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_analysis_double_slash_in_path() {
    let tree = render_analysis_tree(&export(vec![row("src//main.rs", FileKind::Parent, 10, 20)]));
    // Empty segments filtered out, should still work
    assert!(tree.contains("src"));
    assert!(tree.contains("main.rs"));
}

#[test]
fn w61_analysis_trailing_slash_no_phantom_leaf() {
    let tree = render_analysis_tree(&export(vec![row("dir/", FileKind::Parent, 5, 10)]));
    assert!(tree.contains("dir (lines: 5, tokens: 10)"));
    assert_eq!(tree.lines().count(), 1);
}

#[test]
fn w61_analysis_leading_slash_stripped() {
    let tree = render_analysis_tree(&export(vec![row(
        "/root/file.rs",
        FileKind::Parent,
        10,
        20,
    )]));
    assert!(tree.contains("root"));
    assert!(tree.contains("file.rs"));
    // No empty-name node from leading slash
    let lines: Vec<&str> = tree.lines().collect();
    assert_eq!(lines.len(), 2);
}

#[test]
fn w61_analysis_single_segment_path() {
    let tree = render_analysis_tree(&export(vec![row("Cargo.toml", FileKind::Parent, 20, 40)]));
    assert_eq!(tree.trim(), "Cargo.toml (lines: 20, tokens: 40)");
}

#[test]
fn w61_handoff_single_segment_root_only() {
    let tree = render_handoff_tree(&export(vec![row("README.md", FileKind::Parent, 5, 10)]), 10);
    assert_eq!(tree.lines().count(), 1);
    assert!(tree.contains("(root) (files: 1, lines: 5, tokens: 10)"));
}

// ═══════════════════════════════════════════════════════════════════
// 6. Aggregation arithmetic
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_analysis_aggregation_sums_correctly() {
    let tree = render_analysis_tree(&export(vec![
        row("pkg/a.rs", FileKind::Parent, 7, 14),
        row("pkg/b.rs", FileKind::Parent, 13, 26),
        row("pkg/c.rs", FileKind::Parent, 80, 160),
    ]));
    assert!(tree.contains("pkg (lines: 100, tokens: 200)"));
}

#[test]
fn w61_handoff_aggregation_across_nested_dirs() {
    let tree = render_handoff_tree(
        &export(vec![
            row("a/x/f1.rs", FileKind::Parent, 10, 20),
            row("a/y/f2.rs", FileKind::Parent, 20, 40),
            row("b/z/f3.rs", FileKind::Parent, 30, 60),
        ]),
        3,
    );
    assert!(tree.contains("(root) (files: 3, lines: 60, tokens: 120)"));
    assert!(tree.contains("a/ (files: 2, lines: 30, tokens: 60)"));
    assert!(tree.contains("b/ (files: 1, lines: 30, tokens: 60)"));
}

#[test]
fn w61_analysis_zero_valued_rows_aggregate_to_zero() {
    let tree = render_analysis_tree(&export(vec![
        row("pkg/a.rs", FileKind::Parent, 0, 0),
        row("pkg/b.rs", FileKind::Parent, 0, 0),
    ]));
    assert!(tree.contains("pkg (lines: 0, tokens: 0)"));
}

#[test]
fn w61_handoff_files_count_per_dir() {
    let tree = render_handoff_tree(
        &export(vec![
            row("d/a.rs", FileKind::Parent, 1, 2),
            row("d/b.rs", FileKind::Parent, 3, 6),
            row("d/c.rs", FileKind::Parent, 5, 10),
        ]),
        3,
    );
    assert!(tree.contains("d/ (files: 3, lines: 9, tokens: 18)"));
}

// ═══════════════════════════════════════════════════════════════════
// 7. Determinism — input order independence
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_analysis_order_independent_three_dirs() {
    let make = |order: &[usize]| {
        let all = [
            row("a/f.rs", FileKind::Parent, 10, 20),
            row("b/f.rs", FileKind::Parent, 20, 40),
            row("c/f.rs", FileKind::Parent, 30, 60),
        ];
        export(order.iter().map(|&i| all[i].clone()).collect())
    };
    let baseline = render_analysis_tree(&make(&[0, 1, 2]));
    assert_eq!(render_analysis_tree(&make(&[2, 1, 0])), baseline);
    assert_eq!(render_analysis_tree(&make(&[1, 0, 2])), baseline);
}

#[test]
fn w61_handoff_order_independent_three_dirs() {
    let make = |order: &[usize]| {
        let all = [
            row("a/f.rs", FileKind::Parent, 10, 20),
            row("b/f.rs", FileKind::Parent, 20, 40),
            row("c/f.rs", FileKind::Parent, 30, 60),
        ];
        export(order.iter().map(|&i| all[i].clone()).collect())
    };
    let baseline = render_handoff_tree(&make(&[0, 1, 2]), 3);
    assert_eq!(render_handoff_tree(&make(&[2, 1, 0]), 3), baseline);
    assert_eq!(render_handoff_tree(&make(&[1, 0, 2]), 3), baseline);
}

#[test]
fn w61_determinism_100_iterations_analysis() {
    let data = export(vec![
        row("x/a.rs", FileKind::Parent, 10, 20),
        row("x/b.rs", FileKind::Parent, 30, 60),
        row("y/c.rs", FileKind::Parent, 50, 100),
    ]);
    let first = render_analysis_tree(&data);
    for _ in 0..100 {
        assert_eq!(render_analysis_tree(&data), first);
    }
}

#[test]
fn w61_determinism_100_iterations_handoff() {
    let data = export(vec![
        row("x/a.rs", FileKind::Parent, 10, 20),
        row("y/b.rs", FileKind::Parent, 30, 60),
    ]);
    let first = render_handoff_tree(&data, 3);
    for _ in 0..100 {
        assert_eq!(render_handoff_tree(&data, 3), first);
    }
}

// ═══════════════════════════════════════════════════════════════════
// 8. Cross-tree validation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_analysis_shows_leaves_handoff_does_not() {
    let data = export(vec![
        row("src/main.rs", FileKind::Parent, 100, 200),
        row("src/lib.rs", FileKind::Parent, 50, 100),
    ]);
    let analysis = render_analysis_tree(&data);
    let handoff = render_handoff_tree(&data, 5);
    assert!(analysis.contains("main.rs"));
    assert!(analysis.contains("lib.rs"));
    assert!(!handoff.contains("main.rs"));
    assert!(!handoff.contains("lib.rs"));
}

#[test]
fn w61_handoff_root_totals_match_sum_of_analysis_roots() {
    let data = export(vec![
        row("a/f.rs", FileKind::Parent, 10, 20),
        row("b/f.rs", FileKind::Parent, 30, 60),
        row("c/f.rs", FileKind::Parent, 60, 120),
    ]);
    let handoff = render_handoff_tree(&data, 1);
    assert!(handoff.contains("(root) (files: 3, lines: 100, tokens: 200)"));
}

// ═══════════════════════════════════════════════════════════════════
// 9. Output formatting
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_analysis_output_ends_with_newline() {
    let tree = render_analysis_tree(&export(vec![row("f.rs", FileKind::Parent, 1, 2)]));
    assert!(tree.ends_with('\n'));
}

#[test]
fn w61_handoff_output_ends_with_newline() {
    let tree = render_handoff_tree(&export(vec![row("d/f.rs", FileKind::Parent, 1, 2)]), 3);
    assert!(tree.ends_with('\n'));
}

#[test]
fn w61_analysis_indentation_per_level() {
    let tree = render_analysis_tree(&export(vec![row("a/b/c/d.rs", FileKind::Parent, 1, 2)]));
    let lines: Vec<&str> = tree.lines().collect();
    assert_eq!(lines.len(), 4);
    for (i, line) in lines.iter().enumerate() {
        let indent = line.len() - line.trim_start().len();
        assert_eq!(indent, i * 2, "wrong indent at depth {i}");
    }
}

#[test]
fn w61_handoff_dirs_have_trailing_slash() {
    let tree = render_handoff_tree(
        &export(vec![row("src/sub/f.rs", FileKind::Parent, 5, 10)]),
        5,
    );
    for line in tree.lines().skip(1) {
        let name = line.trim().split(" (").next().unwrap();
        assert!(
            name.ends_with('/'),
            "dir should have trailing slash: {name}"
        );
    }
}

#[test]
fn w61_handoff_root_line_no_trailing_slash() {
    let tree = render_handoff_tree(&export(vec![row("d/f.rs", FileKind::Parent, 1, 2)]), 3);
    let first = tree.lines().next().unwrap();
    assert!(first.starts_with("(root)"));
    assert!(!first.starts_with("(root)/"));
}

// ═══════════════════════════════════════════════════════════════════
// 10. Stress — large trees
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_stress_50_files_in_one_dir() {
    let rows: Vec<FileRow> = (0..50)
        .map(|i| row(&format!("pkg/f_{i:03}.rs"), FileKind::Parent, 10, 20))
        .collect();
    let tree = render_analysis_tree(&export(rows));
    assert!(tree.contains("pkg (lines: 500, tokens: 1000)"));
    assert_eq!(tree.lines().count(), 51); // 1 dir + 50 files
}

#[test]
fn w61_stress_50_directories() {
    let rows: Vec<FileRow> = (0..50)
        .map(|i| row(&format!("dir_{i:03}/main.rs"), FileKind::Parent, 10, 20))
        .collect();
    let tree = render_handoff_tree(&export(rows), 2);
    assert!(tree.contains("(root) (files: 50, lines: 500, tokens: 1000)"));
    for i in 0..50 {
        assert!(tree.contains(&format!("dir_{i:03}/")));
    }
}

#[test]
fn w61_stress_deep_nesting_15_levels() {
    let path = (0..14)
        .map(|i| format!("d{i}"))
        .collect::<Vec<_>>()
        .join("/")
        + "/leaf.rs";
    let tree = render_analysis_tree(&export(vec![row(&path, FileKind::Parent, 1, 2)]));
    assert_eq!(tree.lines().count(), 15); // 14 dirs + 1 file
}

#[test]
fn w61_stress_handoff_deep_nesting_capped_by_depth() {
    let path = (0..20)
        .map(|i| format!("d{i}"))
        .collect::<Vec<_>>()
        .join("/")
        + "/leaf.rs";
    let tree = render_handoff_tree(&export(vec![row(&path, FileKind::Parent, 1, 2)]), 5);
    // depth=5: root(0) + d0(1) + d1(2) + d2(3) + d3(4) + d4(5) = 6 lines max
    let count = tree.lines().count();
    assert!(count <= 6, "expected at most 6 lines, got {count}");
}
