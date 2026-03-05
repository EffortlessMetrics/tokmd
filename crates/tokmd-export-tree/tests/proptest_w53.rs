//! W53: Property-based tests for `tokmd-export-tree`.
//!
//! Covers: tree path normalization, sorting stability, determinism,
//! structural invariants, and boundary conditions.

use proptest::prelude::*;
use tokmd_export_tree::{render_analysis_tree, render_handoff_tree};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Strategies ──────────────────────────────────────────────────────────

fn path_segment() -> impl Strategy<Value = String> {
    "[a-z]{1,8}".prop_map(String::from)
}

fn filename() -> impl Strategy<Value = String> {
    "[a-z]{1,8}\\.[a-z]{1,4}".prop_map(String::from)
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

fn row_strategy(kind: FileKind) -> impl Strategy<Value = FileRow> {
    (path_strategy(), 0usize..4000, 0usize..4000, 0usize..4000).prop_map(
        move |(path, code, comments, tokens)| {
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
        },
    )
}

fn parent_row_strategy() -> impl Strategy<Value = FileRow> {
    row_strategy(FileKind::Parent)
}

fn mixed_row_strategy() -> impl Strategy<Value = FileRow> {
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

// ── Tests ───────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(120))]

    // 1. Analysis tree is deterministic
    #[test]
    fn analysis_tree_deterministic(rows in proptest::collection::vec(mixed_row_strategy(), 0..30)) {
        let e = export(rows);
        let a = render_analysis_tree(&e);
        let b = render_analysis_tree(&e);
        prop_assert_eq!(a, b);
    }

    // 2. Handoff tree is deterministic
    #[test]
    fn handoff_tree_deterministic(
        rows in proptest::collection::vec(mixed_row_strategy(), 0..30),
        depth in 0usize..8,
    ) {
        let e = export(rows);
        let a = render_handoff_tree(&e, depth);
        let b = render_handoff_tree(&e, depth);
        prop_assert_eq!(a, b);
    }

    // 3. Input order independence (BTreeMap sorting)
    #[test]
    fn order_independent(
        rows in proptest::collection::vec(parent_row_strategy(), 2..20),
        depth in 1usize..6,
    ) {
        let forward = export(rows.clone());
        let mut rev = rows;
        rev.reverse();
        let reversed = export(rev);

        prop_assert_eq!(
            render_analysis_tree(&forward),
            render_analysis_tree(&reversed),
        );
        prop_assert_eq!(
            render_handoff_tree(&forward, depth),
            render_handoff_tree(&reversed, depth),
        );
    }

    // 4. Empty export produces empty output
    #[test]
    fn empty_export_empty_output(_dummy in 0..1u8) {
        let e = export(vec![]);
        prop_assert!(render_analysis_tree(&e).is_empty());
        prop_assert!(render_handoff_tree(&e, 5).is_empty());
    }

    // 5. Child rows are ignored by analysis tree
    #[test]
    fn child_rows_ignored(
        parent in parent_row_strategy(),
        child_path in path_strategy(),
    ) {
        let child = FileRow {
            path: format!("{}::embedded", child_path),
            module: "(root)".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 999,
            comments: 0,
            blanks: 0,
            lines: 999,
            bytes: 9999,
            tokens: 9999,
        };
        let tree = render_analysis_tree(&export(vec![parent, child]));
        prop_assert!(!tree.contains("embedded"));
    }

    // 6. Handoff depth=0 always emits single line when parents exist
    #[test]
    fn handoff_depth_zero_single_line(rows in proptest::collection::vec(parent_row_strategy(), 1..20)) {
        let tree = render_handoff_tree(&export(rows), 0);
        prop_assert_eq!(tree.lines().count(), 1, "depth=0 should produce 1 line");
    }

    // 7. Handoff root (files: N, ...) matches parent count
    #[test]
    fn handoff_root_count_matches(rows in proptest::collection::vec(parent_row_strategy(), 1..20)) {
        let count = rows.len();
        let tree = render_handoff_tree(&export(rows), 10);
        let expected = format!("(root) (files: {},", count);
        prop_assert!(
            tree.starts_with(&expected),
            "expected '{}', got '{}'",
            expected,
            tree.lines().next().unwrap_or(""),
        );
    }

    // 8. Non-empty parent rows → non-empty analysis tree
    #[test]
    fn parent_rows_nonempty_tree(rows in proptest::collection::vec(parent_row_strategy(), 1..20)) {
        let tree = render_analysis_tree(&export(rows));
        prop_assert!(!tree.is_empty());
    }

    // 9. Analysis tree output ends with newline
    #[test]
    fn analysis_tree_ends_newline(rows in proptest::collection::vec(parent_row_strategy(), 1..20)) {
        let tree = render_analysis_tree(&export(rows));
        prop_assert!(tree.ends_with('\n'));
    }

    // 10. Handoff tree output ends with newline
    #[test]
    fn handoff_tree_ends_newline(
        rows in proptest::collection::vec(parent_row_strategy(), 1..20),
        depth in 0usize..6,
    ) {
        let tree = render_handoff_tree(&export(rows), depth);
        prop_assert!(tree.ends_with('\n'));
    }

    // 11. Increasing depth never reduces output length
    #[test]
    fn deeper_handoff_not_shorter(rows in proptest::collection::vec(parent_row_strategy(), 1..15)) {
        let e = export(rows);
        let shallow = render_handoff_tree(&e, 1);
        let deep = render_handoff_tree(&e, 5);
        prop_assert!(
            deep.len() >= shallow.len(),
            "deep len {} < shallow len {}",
            deep.len(),
            shallow.len()
        );
    }

    // 12. Handoff tree never contains file extensions (omits leaf filenames)
    #[test]
    fn handoff_tree_no_file_leaves(
        rows in proptest::collection::vec(parent_row_strategy(), 1..15),
        depth in 0usize..6,
    ) {
        let tree = render_handoff_tree(&export(rows), depth);
        // Files have extensions like .rs, .toml; handoff only shows directories
        for line in tree.lines() {
            let trimmed = line.trim();
            // Each line is either "(root) ..." or "dirname/ ..."
            if !trimmed.is_empty() && !trimmed.starts_with("(root)") {
                prop_assert!(
                    trimmed.contains('/'),
                    "expected directory trailing slash in: {}",
                    trimmed
                );
            }
        }
    }
}
