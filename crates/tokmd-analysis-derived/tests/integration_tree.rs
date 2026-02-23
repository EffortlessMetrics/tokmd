use tokmd_analysis_derived::build_tree;
use tokmd_export_tree::render_analysis_tree;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

fn row(path: &str, kind: FileKind, lines: usize, tokens: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind,
        code: lines,
        comments: 0,
        blanks: 0,
        lines,
        bytes: lines * 8,
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

#[test]
fn derived_build_tree_matches_export_tree_renderer() {
    let export = export(vec![
        row("src/main.rs", FileKind::Parent, 10, 20),
        row("src/lib.rs", FileKind::Parent, 20, 40),
    ]);

    assert_eq!(build_tree(&export), render_analysis_tree(&export));
}

#[test]
fn derived_build_tree_ignores_child_rows() {
    let export = export(vec![
        row("src/main.rs", FileKind::Parent, 10, 20),
        row("src/main.rs::embedded", FileKind::Child, 99, 199),
    ]);

    let tree = build_tree(&export);
    assert!(tree.contains("main.rs (lines: 10, tokens: 20)"));
    assert!(!tree.contains("embedded"));
}
