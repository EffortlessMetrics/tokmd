use tokmd_export_tree::{render_analysis_tree, render_handoff_tree};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

fn row(path: &str, module: &str, lines: usize, tokens: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
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
        module_roots: vec!["crates".to_string(), "packages".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::ParentsOnly,
    }
}

#[test]
fn renders_workspace_like_layout_for_both_consumers() {
    let export = export(vec![
        row(
            "crates/tokmd-model/src/lib.rs",
            "crates/tokmd-model",
            50,
            120,
        ),
        row(
            "crates/tokmd-model/tests/model_tests.rs",
            "crates/tokmd-model",
            70,
            160,
        ),
        row("packages/web/src/main.ts", "packages/web", 20, 60),
        row("README.md", "(root)", 5, 10),
    ]);

    let analysis_tree = render_analysis_tree(&export);
    let handoff_tree = render_handoff_tree(&export, 3);

    assert!(analysis_tree.contains("README.md (lines: 5, tokens: 10)"));
    assert!(analysis_tree.contains("crates (lines: 120, tokens: 280)"));
    assert!(handoff_tree.contains("(root) (files: 4, lines: 145, tokens: 350)"));
    assert!(handoff_tree.contains("  crates/ (files: 2, lines: 120, tokens: 280)"));
}

#[test]
fn handoff_root_counts_match_parent_file_inventory() {
    let export = export(vec![
        row("src/main.rs", "src", 10, 20),
        row("src/lib.rs", "src", 20, 30),
    ]);

    let tree = render_handoff_tree(&export, 2);
    let first_line = tree.lines().next().unwrap_or_default();
    assert_eq!(
        first_line, "(root) (files: 2, lines: 30, tokens: 50)",
        "root counts should match parent rows"
    );
}

#[test]
fn analysis_tree_deep_workspace_aggregates_correctly() {
    let export = export(vec![
        row("crates/a/src/lib.rs", "crates/a", 100, 200),
        row("crates/a/src/utils.rs", "crates/a", 50, 100),
        row("crates/b/src/lib.rs", "crates/b", 30, 60),
        row("packages/web/src/index.ts", "packages/web", 40, 80),
    ]);

    let tree = render_analysis_tree(&export);

    // "crates" should aggregate all files under it
    assert!(tree.contains("crates (lines: 180, tokens: 360)"));
    // "a" under crates should aggregate its two files
    assert!(tree.contains("a (lines: 150, tokens: 300)"));
    assert!(tree.contains("b (lines: 30, tokens: 60)"));
    assert!(tree.contains("packages (lines: 40, tokens: 80)"));
}

#[test]
fn handoff_tree_depth_two_shows_intermediate_dirs() {
    let export = export(vec![
        row("crates/a/src/lib.rs", "crates/a", 100, 200),
        row("crates/b/src/lib.rs", "crates/b", 30, 60),
    ]);

    let tree = render_handoff_tree(&export, 2);

    assert!(tree.contains("(root)"));
    assert!(tree.contains("crates/ (files: 2, lines: 130, tokens: 260)"));
    assert!(tree.contains("a/ (files: 1, lines: 100, tokens: 200)"));
    assert!(tree.contains("b/ (files: 1, lines: 30, tokens: 60)"));
    // depth=2 should NOT show "src/"
    assert!(!tree.contains("src/"));
}

#[test]
fn analysis_tree_indentation_increases_with_depth() {
    let export = export(vec![row(
        "a/b/c.rs",
        "a",
        10,
        20,
    )]);

    let tree = render_analysis_tree(&export);
    let lines: Vec<&str> = tree.lines().collect();

    // "a" at level 0, "b" at level 1, "c.rs" at level 2
    assert!(lines[0].starts_with("a "));
    assert!(lines[1].starts_with("  b "));
    assert!(lines[2].starts_with("    c.rs "));
}

#[test]
fn handoff_tree_indentation_increases_with_depth() {
    let export = export(vec![row(
        "a/b/c/d.rs",
        "a",
        10,
        20,
    )]);

    let tree = render_handoff_tree(&export, 5);
    let lines: Vec<&str> = tree.lines().collect();

    assert!(lines[0].starts_with("(root)"));
    assert!(lines[1].starts_with("  a/"));
    assert!(lines[2].starts_with("    b/"));
    assert!(lines[3].starts_with("      c/"));
}

#[test]
fn single_file_at_root_produces_minimal_trees() {
    let export = export(vec![row("Cargo.toml", "(root)", 12, 24)]);

    let analysis = render_analysis_tree(&export);
    assert_eq!(analysis.lines().count(), 1);
    assert!(analysis.contains("Cargo.toml (lines: 12, tokens: 24)"));

    let handoff = render_handoff_tree(&export, 5);
    // only the root line, no directory children
    assert_eq!(handoff.lines().count(), 1);
    assert!(handoff.contains("(root) (files: 1, lines: 12, tokens: 24)"));
}

#[test]
fn zero_line_zero_token_files_are_rendered() {
    let export = export(vec![
        row("src/empty.rs", "src", 0, 0),
        row("src/real.rs", "src", 10, 20),
    ]);

    let analysis = render_analysis_tree(&export);
    assert!(analysis.contains("empty.rs (lines: 0, tokens: 0)"));
    assert!(analysis.contains("src (lines: 10, tokens: 20)"));

    let handoff = render_handoff_tree(&export, 3);
    assert!(handoff.contains("(root) (files: 2, lines: 10, tokens: 20)"));
}
