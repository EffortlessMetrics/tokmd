use tokmd_export_tree::{render_analysis_tree, render_handoff_tree};
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

#[test]
fn given_parent_and_child_rows_when_rendering_analysis_tree_then_only_parent_data_is_counted() {
    let export = export(vec![
        row("src/lib.rs", FileKind::Parent, 10, 20),
        row("src/lib.rs::markdown", FileKind::Child, 99, 999),
    ]);

    let tree = render_analysis_tree(&export);

    assert!(tree.contains("lib.rs (lines: 10, tokens: 20)"));
    assert!(!tree.contains("99"));
    assert!(!tree.contains("markdown"));
}

#[test]
fn given_nested_paths_when_rendering_handoff_tree_then_depth_limit_hides_deeper_nodes() {
    let export = export(vec![row("a/b/c/file.rs", FileKind::Parent, 10, 20)]);

    let tree = render_handoff_tree(&export, 1);

    assert!(tree.contains("(root)"));
    assert!(tree.contains("a/"));
    assert!(!tree.contains("b/"));
    assert!(!tree.contains("file.rs"));
}

#[test]
fn given_unsorted_paths_when_rendering_analysis_tree_then_output_order_is_deterministic() {
    let export = export(vec![
        row("src/z.rs", FileKind::Parent, 1, 1),
        row("src/a.rs", FileKind::Parent, 1, 1),
    ]);

    let tree = render_analysis_tree(&export);
    let a_idx = tree.find("a.rs").expect("a.rs present");
    let z_idx = tree.find("z.rs").expect("z.rs present");

    assert!(a_idx < z_idx);
}
