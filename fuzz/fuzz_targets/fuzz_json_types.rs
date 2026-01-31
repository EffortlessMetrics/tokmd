//! Fuzz target for JSON deserialization of receipt types.
//!
//! Tests deserialization of `LangReceipt`, `ModuleReceipt`, `ExportReceipt`,
//! `RunReceipt`, `FileRow`, and other types from arbitrary JSON input.

#![no_main]
use libfuzzer_sys::fuzz_target;
use tokmd_types::{
    ExportData, FileRow, LangReport, LangRow, ModuleReport, ModuleRow, RunReceipt, Totals,
};

/// Max input size to prevent pathological parse times
const MAX_INPUT_SIZE: usize = 64 * 1024; // 64KB

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_SIZE {
        return;
    }
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };

    // Try deserializing as various receipt types and verify invariants

    // Totals: all numeric fields are usize (non-negative)
    if let Ok(totals) = serde_json::from_str::<Totals>(s) {
        // usize is always >= 0, but we verify the fields are sensible
        // lines should be >= code (lines = code + comments + blanks)
        // However, we can't enforce this on arbitrary input - just verify the fields exist
        // and are accessible without panic
        let _ = totals.code;
        let _ = totals.lines;
        let _ = totals.files;
        let _ = totals.bytes;
        let _ = totals.tokens;
        let _ = totals.avg_lines;
    }

    // FileRow: verify fields are accessible without panic
    if let Ok(file_row) = serde_json::from_str::<FileRow>(s) {
        // path must be a valid string (it is if we got here)
        let _ = file_row.path.len();
        let _ = file_row.module.len();
        let _ = file_row.lang.len();

        // All numeric fields are usize (non-negative by type)
        let _ = file_row.code;
        let _ = file_row.comments;
        let _ = file_row.blanks;
        let _ = file_row.lines;
        let _ = file_row.bytes;
        let _ = file_row.tokens;

        // Note: We don't assert lines == code + comments + blanks here because
        // arbitrary JSON input may not satisfy this invariant. The invariant is
        // only guaranteed by the creation functions in tokmd-model, not by
        // arbitrary deserialization.
    }

    // LangRow: verify lang is accessible
    if let Ok(lang_row) = serde_json::from_str::<LangRow>(s) {
        // lang must be a valid string
        let _ = lang_row.lang.len();
        // All numeric fields are usize (non-negative by type)
        let _ = lang_row.code;
        let _ = lang_row.files;
    }

    // ModuleRow: verify module is accessible
    if let Ok(module_row) = serde_json::from_str::<ModuleRow>(s) {
        // module must be a valid string
        let _ = module_row.module.len();
        // All numeric fields are usize (non-negative by type)
        let _ = module_row.code;
        let _ = module_row.files;
    }

    // LangReport: verify rows and totals
    if let Ok(report) = serde_json::from_str::<LangReport>(s) {
        // All rows should be valid LangRows
        for row in &report.rows {
            let _ = row.lang.len();
            let _ = row.code;
        }
        // totals should be accessible
        let _ = report.total.code;
        let _ = report.total.files;
    }

    // ModuleReport: verify rows and totals
    if let Ok(report) = serde_json::from_str::<ModuleReport>(s) {
        // All rows should be valid ModuleRows
        for row in &report.rows {
            let _ = row.module.len();
            let _ = row.code;
        }
        // totals should be accessible
        let _ = report.total.code;
        // module_depth should be reasonable (we don't enforce a max, but verify access)
        let _ = report.module_depth;
    }

    // ExportData: verify rows and structure
    if let Ok(export) = serde_json::from_str::<ExportData>(s) {
        // Verify all rows are accessible
        for row in &export.rows {
            let _ = row.path.len();
            let _ = row.module.len();
            let _ = row.lang.len();
            let _ = row.code;
            let _ = row.lines;
        }

        // Verify module_roots is accessible
        for root in &export.module_roots {
            let _ = root.len();
        }

        // module_depth should be accessible
        let _ = export.module_depth;
    }

    // RunReceipt: verify schema_version and file paths
    if let Ok(receipt) = serde_json::from_str::<RunReceipt>(s) {
        // schema_version should be accessible
        let _ = receipt.schema_version;
        // file paths should be non-empty strings (or at least accessible)
        let _ = receipt.lang_file.len();
        let _ = receipt.module_file.len();
        let _ = receipt.export_file.len();
    }

    // Also try as generic JSON Value and back
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(s) {
        // Round-trip through Value
        let _ = serde_json::from_value::<RunReceipt>(value.clone());
        let _ = serde_json::from_value::<LangReport>(value.clone());
        let _ = serde_json::from_value::<ExportData>(value.clone());
        let _ = serde_json::from_value::<FileRow>(value);
    }
});
