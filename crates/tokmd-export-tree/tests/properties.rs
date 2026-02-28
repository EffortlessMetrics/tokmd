use proptest::prelude::*;
use tokmd_export_tree::{render_analysis_tree, render_handoff_tree};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

fn path_segment() -> impl Strategy<Value = String> {
    "[a-z]{1,8}".prop_map(String::from)
}

fn filename() -> impl Strategy<Value = String> {
    "[a-z]{1,8}\\.rs".prop_map(String::from)
}

fn path_strategy() -> impl Strategy<Value = String> {
    (prop::collection::vec(path_segment(), 0..4), filename()).prop_map(|(dirs, file)| {
        if dirs.is_empty() {
            file
        } else {
            format!("{}/{}", dirs.join("/"), file)
        }
    })
}

fn row_strategy() -> impl Strategy<Value = FileRow> {
    (
        path_strategy(),
        prop::bool::ANY,
        0usize..4000,
        0usize..4000,
        0usize..4000,
    )
        .prop_map(|(path, is_child, code, comments, tokens)| {
            let kind = if is_child {
                FileKind::Child
            } else {
                FileKind::Parent
            };
            let module = path
                .split('/')
                .next()
                .map_or_else(|| "(root)".to_string(), String::from);
            let blanks = code % 13;
            let lines = code.saturating_add(comments).saturating_add(blanks);
            FileRow {
                path,
                module,
                lang: "Rust".to_string(),
                kind,
                code,
                comments,
                blanks,
                lines,
                bytes: lines.saturating_mul(4),
                tokens,
            }
        })
}

fn export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::ParentsOnly,
    }
}

proptest! {
    #[test]
    fn renderers_are_deterministic(rows in prop::collection::vec(row_strategy(), 0..64), depth in 0usize..8) {
        let export = export(rows);
        let analysis_1 = render_analysis_tree(&export);
        let analysis_2 = render_analysis_tree(&export);
        let handoff_1 = render_handoff_tree(&export, depth);
        let handoff_2 = render_handoff_tree(&export, depth);

        prop_assert_eq!(analysis_1, analysis_2);
        prop_assert_eq!(handoff_1, handoff_2);
    }

    #[test]
    fn renderers_are_order_independent(rows in prop::collection::vec(row_strategy(), 0..64), depth in 0usize..8) {
        let forward = export(rows.clone());
        let mut reversed_rows = rows;
        reversed_rows.reverse();
        let reversed = export(reversed_rows);

        prop_assert_eq!(
            render_analysis_tree(&forward),
            render_analysis_tree(&reversed)
        );
        prop_assert_eq!(
            render_handoff_tree(&forward, depth),
            render_handoff_tree(&reversed, depth)
        );
    }

    #[test]
    fn handoff_tree_omits_file_leaf_names(
        rows in prop::collection::vec(path_strategy(), 1..32),
        depth in 0usize..8
    ) {
        let parent_rows: Vec<FileRow> = rows
            .iter()
            .enumerate()
            .map(|(idx, path)| FileRow {
                path: path.clone(),
                module: path.split('/').next().unwrap_or("(root)").to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: idx + 1,
                comments: 0,
                blanks: 0,
                lines: idx + 1,
                bytes: idx + 10,
                tokens: idx + 20,
            })
            .collect();

        let tree = render_handoff_tree(&export(parent_rows), depth);
        prop_assert!(!tree.contains(".rs"));
    }

    #[test]
    fn analysis_tree_never_empty_when_parents_exist(
        rows in prop::collection::vec(row_strategy(), 1..64)
    ) {
        let has_parent = rows.iter().any(|r| r.kind == FileKind::Parent);
        let tree = render_analysis_tree(&export(rows));
        if has_parent {
            prop_assert!(!tree.is_empty());
        }
    }

    #[test]
    fn handoff_root_file_count_matches_parent_rows(
        rows in prop::collection::vec(row_strategy(), 1..64)
    ) {
        let parent_count = rows.iter().filter(|r| r.kind == FileKind::Parent).count();
        let tree = render_handoff_tree(&export(rows), 10);
        if parent_count > 0 {
            let expected_prefix = format!("(root) (files: {parent_count},");
            prop_assert!(
                tree.starts_with(&expected_prefix),
                "expected root to start with '{}', got '{}'",
                expected_prefix,
                tree.lines().next().unwrap_or("")
            );
        }
    }

    #[test]
    fn handoff_depth_zero_emits_single_line(
        rows in prop::collection::vec(row_strategy(), 1..64)
    ) {
        let has_parent = rows.iter().any(|r| r.kind == FileKind::Parent);
        let tree = render_handoff_tree(&export(rows), 0);
        if has_parent {
            prop_assert_eq!(tree.lines().count(), 1, "depth=0 should emit only the root line");
        }
    }

    #[test]
    fn analysis_tree_output_ends_with_newline(
        rows in prop::collection::vec(row_strategy(), 1..64)
    ) {
        let has_parent = rows.iter().any(|r| r.kind == FileKind::Parent);
        let tree = render_analysis_tree(&export(rows));
        if has_parent {
            prop_assert!(tree.ends_with('\n'), "non-empty tree should end with newline");
        }
    }

    #[test]
    fn handoff_tree_output_ends_with_newline(
        rows in prop::collection::vec(row_strategy(), 1..64),
        depth in 0usize..8
    ) {
        let has_parent = rows.iter().any(|r| r.kind == FileKind::Parent);
        let tree = render_handoff_tree(&export(rows), depth);
        if has_parent {
            prop_assert!(tree.ends_with('\n'), "non-empty tree should end with newline");
        }
    }
}
