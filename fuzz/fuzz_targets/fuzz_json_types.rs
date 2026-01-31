//! Fuzz target for JSON deserialization of receipt types.
//!
//! Tests deserialization of `LangReceipt`, `ModuleReceipt`, `ExportReceipt`,
//! `RunReceipt`, `FileRow`, and other types from arbitrary JSON input.

#![no_main]
use libfuzzer_sys::fuzz_target;
use tokmd_types::{
    ExportData, FileRow, LangReport, LangRow, ModuleReport, ModuleRow, RunReceipt, Totals,
};

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };

    // Try deserializing as various receipt types - should never panic
    let _ = serde_json::from_str::<RunReceipt>(s);
    let _ = serde_json::from_str::<LangReport>(s);
    let _ = serde_json::from_str::<ModuleReport>(s);
    let _ = serde_json::from_str::<ExportData>(s);
    let _ = serde_json::from_str::<FileRow>(s);
    let _ = serde_json::from_str::<LangRow>(s);
    let _ = serde_json::from_str::<ModuleRow>(s);
    let _ = serde_json::from_str::<Totals>(s);

    // Also try as generic JSON Value and back
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(s) {
        // Round-trip through Value
        let _ = serde_json::from_value::<RunReceipt>(value.clone());
        let _ = serde_json::from_value::<LangReport>(value.clone());
        let _ = serde_json::from_value::<ExportData>(value.clone());
        let _ = serde_json::from_value::<FileRow>(value);
    }
});
