use tokmd_export_tree::{render_analysis_tree, render_handoff_tree};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Tree construction preserves hierarchy ──

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
    let export = export(vec![row("a/b/c.rs", "a", 10, 20)]);

    let tree = render_analysis_tree(&export);
    let lines: Vec<&str> = tree.lines().collect();

    // "a" at level 0, "b" at level 1, "c.rs" at level 2
    assert!(lines[0].starts_with("a "));
    assert!(lines[1].starts_with("  b "));
    assert!(lines[2].starts_with("    c.rs "));
}

#[test]
fn handoff_tree_indentation_increases_with_depth() {
    let export = export(vec![row("a/b/c/d.rs", "a", 10, 20)]);

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

// ── Tree construction from file rows preserves hierarchy ──

#[test]
fn analysis_tree_preserves_directory_hierarchy() {
    let export = export(vec![
        row(
            "crates/tokmd-model/src/lib.rs",
            "crates/tokmd-model",
            100,
            200,
        ),
        row(
            "crates/tokmd-model/src/utils.rs",
            "crates/tokmd-model",
            50,
            100,
        ),
        row("crates/tokmd-scan/src/lib.rs", "crates/tokmd-scan", 30, 60),
    ]);

    let tree = render_analysis_tree(&export);
    let lines: Vec<&str> = tree.lines().collect();

    // Top-level: crates
    assert!(lines[0].starts_with("crates "));
    // Second level: tokmd-model, tokmd-scan (sorted)
    assert!(lines[1].starts_with("  tokmd-model "));
    assert!(lines[2].starts_with("    src "));
    // File leaves under src
    assert!(lines[3].starts_with("      lib.rs "));
    assert!(lines[4].starts_with("      utils.rs "));
    // tokmd-scan after tokmd-model
    assert!(lines[5].starts_with("  tokmd-scan "));
}

#[test]
fn handoff_tree_preserves_directory_hierarchy() {
    let export = export(vec![
        row("crates/a/src/lib.rs", "crates/a", 10, 20),
        row("crates/b/src/lib.rs", "crates/b", 5, 10),
    ]);

    let tree = render_handoff_tree(&export, 5);
    let lines: Vec<&str> = tree.lines().collect();

    assert!(lines[0].starts_with("(root)"));
    assert!(lines[1].starts_with("  crates/"));
    assert!(lines[2].starts_with("    a/"));
    assert!(lines[3].starts_with("      src/"));
    assert!(lines[4].starts_with("    b/"));
    assert!(lines[5].starts_with("      src/"));
}

// ── Empty input produces empty tree ──

#[test]
fn empty_rows_produce_empty_analysis_tree() {
    let export = export(vec![]);
    let tree = render_analysis_tree(&export);
    assert!(
        tree.is_empty(),
        "analysis tree from empty rows should be empty"
    );
}

#[test]
fn empty_rows_produce_empty_handoff_tree() {
    let export = export(vec![]);
    let tree = render_handoff_tree(&export, 10);
    assert!(
        tree.is_empty(),
        "handoff tree from empty rows should be empty"
    );
}

// ── Tree is deterministic ──

#[test]
fn analysis_tree_is_deterministic_across_calls() {
    let export = export(vec![
        row("src/b.rs", "src", 20, 40),
        row("src/a.rs", "src", 10, 20),
        row("lib/c.rs", "lib", 5, 10),
    ]);

    let tree1 = render_analysis_tree(&export);
    let tree2 = render_analysis_tree(&export);
    assert_eq!(tree1, tree2, "analysis tree must be deterministic");
}

#[test]
fn handoff_tree_is_deterministic_across_calls() {
    let export = export(vec![
        row("src/b.rs", "src", 20, 40),
        row("src/a.rs", "src", 10, 20),
        row("lib/c.rs", "lib", 5, 10),
    ]);

    let tree1 = render_handoff_tree(&export, 5);
    let tree2 = render_handoff_tree(&export, 5);
    assert_eq!(tree1, tree2, "handoff tree must be deterministic");
}

#[test]
fn trees_are_deterministic_regardless_of_row_order() {
    let rows_forward = vec![
        row("src/a.rs", "src", 10, 20),
        row("src/b.rs", "src", 30, 60),
        row("lib/c.rs", "lib", 5, 10),
    ];
    let mut rows_reversed = rows_forward.clone();
    rows_reversed.reverse();

    let forward_analysis = render_analysis_tree(&export(rows_forward.clone()));
    let reversed_analysis = render_analysis_tree(&export(rows_reversed.clone()));
    assert_eq!(forward_analysis, reversed_analysis);

    let forward_handoff = render_handoff_tree(&export(rows_forward), 5);
    let reversed_handoff = render_handoff_tree(&export(rows_reversed), 5);
    assert_eq!(forward_handoff, reversed_handoff);
}

// ── Round-trip serialization ──

#[test]
fn round_trip_serialization_produces_identical_analysis_tree() {
    let original = export(vec![
        row("crates/a/src/lib.rs", "crates/a", 100, 200),
        row("crates/b/src/lib.rs", "crates/b", 50, 100),
        row("README.md", "(root)", 10, 20),
    ]);

    let tree_before = render_analysis_tree(&original);

    let json = serde_json::to_string(&original).expect("serialize ExportData");
    let deserialized: ExportData = serde_json::from_str(&json).expect("deserialize ExportData");

    let tree_after = render_analysis_tree(&deserialized);
    assert_eq!(
        tree_before, tree_after,
        "analysis tree must survive JSON round-trip"
    );
}

#[test]
fn round_trip_serialization_produces_identical_handoff_tree() {
    let original = export(vec![
        row("crates/a/src/lib.rs", "crates/a", 100, 200),
        row("crates/b/src/lib.rs", "crates/b", 50, 100),
        row("README.md", "(root)", 10, 20),
    ]);

    let tree_before = render_handoff_tree(&original, 5);

    let json = serde_json::to_string(&original).expect("serialize ExportData");
    let deserialized: ExportData = serde_json::from_str(&json).expect("deserialize ExportData");

    let tree_after = render_handoff_tree(&deserialized, 5);
    assert_eq!(
        tree_before, tree_after,
        "handoff tree must survive JSON round-trip"
    );
}

// ── Deep nesting works correctly ──

#[test]
fn analysis_tree_handles_deeply_nested_paths() {
    let export = export(vec![row("a/b/c/d/e/f/g/h/i/j/leaf.rs", "a", 42, 84)]);

    let tree = render_analysis_tree(&export);

    // All 10 directory segments plus the file leaf must appear
    for seg in ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "leaf.rs"] {
        assert!(tree.contains(seg), "missing segment: {seg}");
    }
    // Leaf has correct stats
    assert!(tree.contains("leaf.rs (lines: 42, tokens: 84)"));
    // Each level increases indentation by 2 spaces
    let lines: Vec<&str> = tree.lines().collect();
    assert_eq!(lines.len(), 11, "10 dirs + 1 file = 11 lines");
    for (i, line) in lines.iter().enumerate() {
        let expected_indent = "  ".repeat(i);
        assert!(
            line.starts_with(&expected_indent),
            "line {i} should start with {}-space indent",
            i * 2
        );
    }
}

#[test]
fn handoff_tree_handles_deeply_nested_paths_with_large_depth() {
    let export = export(vec![row("a/b/c/d/e/f/g/h/i/j/leaf.rs", "a", 42, 84)]);

    let tree = render_handoff_tree(&export, 20);

    // Handoff tree shows directories but not file leaves
    for seg in ["a/", "b/", "c/", "d/", "e/", "f/", "g/", "h/", "i/", "j/"] {
        assert!(tree.contains(seg), "missing dir: {seg}");
    }
    assert!(
        !tree.contains("leaf.rs"),
        "file leaf should not appear in handoff tree"
    );
}

#[test]
fn handoff_tree_deep_nesting_respects_depth_limit() {
    let export = export(vec![row("a/b/c/d/e/f/g/h/leaf.rs", "a", 10, 20)]);

    let tree_depth3 = render_handoff_tree(&export, 3);
    // depth=3 means root + 3 levels of children: root, a/, b/, c/
    assert!(tree_depth3.contains("(root)"));
    assert!(tree_depth3.contains("a/"));
    assert!(tree_depth3.contains("b/"));
    assert!(tree_depth3.contains("c/"));
    assert!(!tree_depth3.contains("d/"), "d/ should be beyond depth 3");
}
