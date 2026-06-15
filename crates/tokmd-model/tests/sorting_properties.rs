//! Dedicated mutation-coverage tests for deterministic sorting using public APIs.
//! Since sorting functions are private, we test them through the public reports.

use proptest::prelude::*;
use tokmd_model::{create_lang_report_from_rows, create_module_report_from_rows, create_export_data_from_rows};
use tokmd_types::{FileKind, FileRow, ChildrenMode, ChildIncludeMode};

fn arb_file_row() -> impl Strategy<Value = FileRow> {
    (
        "[a-zA-Z0-9_/.]+",
        "[a-zA-Z0-9_/]+",
        "[a-zA-Z]+",
        prop_oneof![Just(FileKind::Parent)], // Keep it simple for reports
        0usize..1000,
        0usize..500,
        0usize..500,
        0usize..2000,
        0usize..10000,
        0usize..1000,
    )
        .prop_map(
            |(path, module, lang, kind, code, comments, blanks, lines, bytes, tokens)| FileRow {
                path,
                module,
                lang,
                kind,
                code,
                comments,
                blanks,
                lines,
                bytes,
                tokens,
            },
        )
}

proptest! {
    #[test]
    fn lang_report_sorting_is_descending_by_code_then_name(rows in prop::collection::vec(arb_file_row(), 0..20)) {
        let report = create_lang_report_from_rows(&rows, 0, false, ChildrenMode::Collapse);
        for i in 1..report.rows.len() {
            let a = &report.rows[i - 1];
            let b = &report.rows[i];
            prop_assert!(a.code >= b.code);
            if a.code == b.code {
                prop_assert!(a.lang <= b.lang);
            }
        }
    }

    #[test]
    fn module_report_sorting_is_descending_by_code_then_module(rows in prop::collection::vec(arb_file_row(), 0..20)) {
        let report = create_module_report_from_rows(&rows, &[], 1, ChildIncludeMode::ParentsOnly, 0);
        for i in 1..report.rows.len() {
            let a = &report.rows[i - 1];
            let b = &report.rows[i];
            prop_assert!(a.code >= b.code);
            if a.code == b.code {
                prop_assert!(a.module <= b.module);
            }
        }
    }

    #[test]
    fn export_data_sorting_is_descending_by_code_then_path(rows in prop::collection::vec(arb_file_row(), 0..20)) {
        let data = create_export_data_from_rows(rows, &[], 1, ChildIncludeMode::ParentsOnly, 0, 0);
        for i in 1..data.rows.len() {
            let a = &data.rows[i - 1];
            let b = &data.rows[i];
            prop_assert!(a.code >= b.code);
            if a.code == b.code {
                prop_assert!(a.path <= b.path);
            }
        }
    }
}
