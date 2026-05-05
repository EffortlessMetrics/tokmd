//! Deterministic property tests extracted from `fuzz_json_types`.
//!
//! Tests JSON deserialization invariants for model types without requiring a fuzzer.

use proptest::prelude::*;
use tokmd_types::{
    ExportData, FileRow, LangReport, LangRow, ModuleReport, ModuleRow, RunReceipt, Totals,
};

proptest! {
    #[test]
    fn file_row_does_not_crash_on_random_utf8(s in "\\PC*") {
        if let Ok(file_row) = serde_json::from_str::<FileRow>(&s) {
            let _ = file_row.path.len();
            let _ = file_row.module.len();
            let _ = file_row.lang.len();
            let _ = file_row.code;
            let _ = file_row.comments;
            let _ = file_row.blanks;
            let _ = file_row.lines;
            let _ = file_row.bytes;
            let _ = file_row.tokens;
        }
    }

    #[test]
    fn totals_does_not_crash_on_random_utf8(s in "\\PC*") {
        if let Ok(totals) = serde_json::from_str::<Totals>(&s) {
            let _ = totals.code;
            let _ = totals.lines;
            let _ = totals.files;
            let _ = totals.bytes;
            let _ = totals.tokens;
            let _ = totals.avg_lines;
        }
    }

    #[test]
    fn lang_row_does_not_crash_on_random_utf8(s in "\\PC*") {
        if let Ok(lang_row) = serde_json::from_str::<LangRow>(&s) {
            let _ = lang_row.lang.len();
            let _ = lang_row.code;
            let _ = lang_row.files;
        }
    }

    #[test]
    fn module_row_does_not_crash_on_random_utf8(s in "\\PC*") {
        if let Ok(module_row) = serde_json::from_str::<ModuleRow>(&s) {
            let _ = module_row.module.len();
            let _ = module_row.code;
            let _ = module_row.files;
        }
    }

    #[test]
    fn run_receipt_does_not_crash_on_random_utf8(s in "\\PC*") {
        if let Ok(receipt) = serde_json::from_str::<RunReceipt>(&s) {
            let _ = receipt.schema_version;
            let _ = receipt.lang_file.len();
            let _ = receipt.module_file.len();
            let _ = receipt.export_file.len();
        }
    }

    #[test]
    fn export_data_does_not_crash_on_random_utf8(s in "\\PC*") {
        if let Ok(export) = serde_json::from_str::<ExportData>(&s) {
            for row in &export.rows {
                let _ = row.path.len();
                let _ = row.module.len();
                let _ = row.lang.len();
                let _ = row.code;
                let _ = row.lines;
            }

            for root in &export.module_roots {
                let _ = root.len();
            }
            let _ = export.module_depth;
        }
    }

    #[test]
    fn lang_report_does_not_crash_on_random_utf8(s in "\\PC*") {
        if let Ok(report) = serde_json::from_str::<LangReport>(&s) {
            for row in &report.rows {
                let _ = row.lang.len();
                let _ = row.code;
            }
            let _ = report.total.code;
            let _ = report.total.files;
        }
    }

    #[test]
    fn module_report_does_not_crash_on_random_utf8(s in "\\PC*") {
        if let Ok(report) = serde_json::from_str::<ModuleReport>(&s) {
            for row in &report.rows {
                let _ = row.module.len();
                let _ = row.code;
            }
            let _ = report.total.code;
            let _ = report.module_depth;
        }
    }
}
