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

#[test]
fn given_deeply_nested_path_when_rendering_analysis_tree_then_all_segments_appear() {
    let export = export(vec![row("a/b/c/d/e/f.rs", FileKind::Parent, 5, 10)]);

    let tree = render_analysis_tree(&export);

    for seg in ["a", "b", "c", "d", "e", "f.rs"] {
        assert!(tree.contains(seg), "missing segment: {seg}");
    }
    assert!(tree.contains("f.rs (lines: 5, tokens: 10)"));
}

#[test]
fn given_deeply_nested_path_when_rendering_handoff_tree_with_large_depth_then_all_dirs_appear() {
    let export = export(vec![row("a/b/c/d/e/f.rs", FileKind::Parent, 5, 10)]);

    let tree = render_handoff_tree(&export, 10);

    for seg in ["a/", "b/", "c/", "d/", "e/"] {
        assert!(tree.contains(seg), "missing dir: {seg}");
    }
    // file leaf must NOT appear
    assert!(!tree.contains("f.rs"));
}

#[test]
fn given_single_root_file_when_rendering_analysis_tree_then_file_appears_at_top_level() {
    let export = export(vec![row("README.md", FileKind::Parent, 3, 6)]);

    let tree = render_analysis_tree(&export);

    assert!(tree.contains("README.md (lines: 3, tokens: 6)"));
    // no indentation expected for root-level file
    assert!(tree.starts_with("README.md"));
}

#[test]
fn given_single_root_file_when_rendering_handoff_tree_then_root_counts_are_correct() {
    let export = export(vec![row("README.md", FileKind::Parent, 3, 6)]);

    let tree = render_handoff_tree(&export, 3);

    assert_eq!(
        tree.lines().next().unwrap(),
        "(root) (files: 1, lines: 3, tokens: 6)"
    );
    // single root file has no directory children
    assert_eq!(tree.lines().count(), 1);
}

#[test]
fn given_special_characters_in_paths_when_rendering_analysis_tree_then_names_are_preserved() {
    let export = export(vec![
        row("my-crate/src/lib.rs", FileKind::Parent, 10, 20),
        row("my_other.crate/src/main.rs", FileKind::Parent, 5, 8),
    ]);

    let tree = render_analysis_tree(&export);

    assert!(tree.contains("my-crate"));
    assert!(tree.contains("my_other.crate"));
}

#[test]
fn given_special_characters_in_paths_when_rendering_handoff_tree_then_dirs_are_preserved() {
    let export = export(vec![
        row("my-crate/src/lib.rs", FileKind::Parent, 10, 20),
        row("my_other.crate/src/main.rs", FileKind::Parent, 5, 8),
    ]);

    let tree = render_handoff_tree(&export, 3);

    assert!(tree.contains("my-crate/"));
    assert!(tree.contains("my_other.crate/"));
}

#[test]
fn given_multiple_files_in_same_dir_when_rendering_analysis_tree_then_parent_aggregates() {
    let export = export(vec![
        row("src/a.rs", FileKind::Parent, 10, 20),
        row("src/b.rs", FileKind::Parent, 30, 40),
    ]);

    let tree = render_analysis_tree(&export);

    // parent "src" should aggregate lines=40, tokens=60
    assert!(tree.contains("src (lines: 40, tokens: 60)"));
    assert!(tree.contains("a.rs (lines: 10, tokens: 20)"));
    assert!(tree.contains("b.rs (lines: 30, tokens: 40)"));
}

#[test]
fn given_handoff_depth_zero_then_only_root_line_is_emitted() {
    let export = export(vec![
        row("src/a.rs", FileKind::Parent, 10, 20),
        row("lib/b.rs", FileKind::Parent, 5, 10),
    ]);

    let tree = render_handoff_tree(&export, 0);

    assert_eq!(tree.lines().count(), 1);
    assert!(tree.contains("(root) (files: 2, lines: 15, tokens: 30)"));
}

#[test]
fn given_only_child_rows_when_rendering_then_both_trees_are_empty() {
    let export = export(vec![
        row("src/lib.rs::html", FileKind::Child, 10, 20),
        row("src/main.rs::css", FileKind::Child, 5, 8),
    ]);

    assert!(render_analysis_tree(&export).is_empty());
    assert!(render_handoff_tree(&export, 5).is_empty());
}

#[test]
fn given_mixed_depth_files_when_rendering_handoff_tree_then_file_counts_are_correct() {
    let export = export(vec![
        row("README.md", FileKind::Parent, 5, 10),
        row("src/lib.rs", FileKind::Parent, 20, 40),
        row("src/utils/helpers.rs", FileKind::Parent, 15, 30),
    ]);

    let tree = render_handoff_tree(&export, 3);

    // root has all 3 files
    assert!(tree.contains("(root) (files: 3, lines: 40, tokens: 80)"));
    // src/ has 2 files (lib.rs + helpers.rs)
    assert!(tree.contains("src/ (files: 2, lines: 35, tokens: 70)"));
    // utils/ has 1 file
    assert!(tree.contains("utils/ (files: 1, lines: 15, tokens: 30)"));
}
