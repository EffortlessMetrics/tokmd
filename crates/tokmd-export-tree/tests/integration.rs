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
