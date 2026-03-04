//! Wave-40 cross-crate integration tests.
//!
//! These tests verify the full scan → model → format pipeline and
//! scan → analysis → analysis-format pipeline through the tokmd-core
//! library façade, using controlled temp-dir fixtures.

use std::fs;

use tempfile::TempDir;
use tokmd_core::{
    diff_workflow, export_workflow, lang_workflow, module_workflow,
    settings::{DiffSettings, ExportSettings, LangSettings, ModuleSettings, ScanSettings},
};

// ============================================================================
// Helpers
// ============================================================================

/// Create a temp dir with Rust, Python, and nested sub-module files.
fn fixture() -> TempDir {
    let dir = TempDir::new().expect("create temp dir");

    fs::write(
        dir.path().join("main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();

    fs::write(
        dir.path().join("util.py"),
        "def add(a, b):\n    return a + b\n\ndef sub(a, b):\n    return a - b\n",
    )
    .unwrap();

    fs::create_dir_all(dir.path().join("sub")).unwrap();
    fs::write(
        dir.path().join("sub").join("lib.rs"),
        "pub fn greet() -> &'static str {\n    \"hi\"\n}\n",
    )
    .unwrap();

    fs::create_dir_all(dir.path().join("deep").join("nested")).unwrap();
    fs::write(
        dir.path().join("deep").join("nested").join("mod.rs"),
        "pub const X: i32 = 42;\n",
    )
    .unwrap();

    dir
}

fn fixture_v2() -> TempDir {
    let dir = TempDir::new().expect("create temp dir");
    fs::write(
        dir.path().join("main.rs"),
        concat!(
            "fn main() {\n",
            "    let x = 1;\n",
            "    let y = 2;\n",
            "    println!(\"{}\", x + y);\n",
            "}\n",
            "\n",
            "fn helper() -> i32 { 42 }\n",
        ),
    )
    .unwrap();
    dir
}

fn scan_for(dir: &TempDir) -> ScanSettings {
    ScanSettings::for_paths(vec![dir.path().to_string_lossy().into_owned()])
}

fn strip_volatile(v: &mut serde_json::Value) {
    if let Some(obj) = v.as_object_mut() {
        obj.remove("generated_at_ms");
        obj.remove("scan_duration_ms");
        obj.remove("export_generated_at_ms");
        for (_, child) in obj.iter_mut() {
            strip_volatile(child);
        }
    }
    if let Some(arr) = v.as_array_mut() {
        for child in arr.iter_mut() {
            strip_volatile(child);
        }
    }
}

// ============================================================================
// 1. Scan → Model → Format: valid output
// ============================================================================

#[test]
fn pipeline_lang_produces_valid_receipt() {
    let dir = fixture();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default())
        .expect("lang_workflow should succeed");

    assert_eq!(receipt.mode, "lang");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(!receipt.report.rows.is_empty());
    assert!(receipt.report.total.code > 0);
    assert!(receipt.report.total.files > 0);
}

#[test]
fn pipeline_lang_finds_known_languages() {
    let dir = fixture();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();

    let langs: Vec<&str> = receipt
        .report
        .rows
        .iter()
        .map(|r| r.lang.as_str())
        .collect();
    assert!(langs.contains(&"Rust"), "expected Rust, got {langs:?}");
    assert!(langs.contains(&"Python"), "expected Python, got {langs:?}");
}

// ============================================================================
// 2. Scan → Model → Format → JSON: valid JSON
// ============================================================================

#[test]
fn pipeline_lang_json_round_trips() {
    let dir = fixture();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();

    let json_str = serde_json::to_string_pretty(&receipt).expect("serialize");
    let back: tokmd_types::LangReceipt =
        serde_json::from_str(&json_str).expect("deserialize round-trip");

    assert_eq!(receipt.mode, back.mode);
    assert_eq!(receipt.schema_version, back.schema_version);
    assert_eq!(receipt.report.rows.len(), back.report.rows.len());
    assert_eq!(receipt.report.total.code, back.report.total.code);
}

#[test]
fn pipeline_module_json_round_trips() {
    let dir = fixture();
    let receipt = module_workflow(&scan_for(&dir), &ModuleSettings::default()).unwrap();

    let json_str = serde_json::to_string(&receipt).expect("serialize");
    let back: tokmd_types::ModuleReceipt =
        serde_json::from_str(&json_str).expect("deserialize round-trip");

    assert_eq!(receipt.mode, back.mode);
    assert_eq!(receipt.report.rows.len(), back.report.rows.len());
}

#[test]
fn pipeline_export_json_round_trips() {
    let dir = fixture();
    let receipt = export_workflow(&scan_for(&dir), &ExportSettings::default()).unwrap();

    let json_str = serde_json::to_string(&receipt).expect("serialize");
    let back: tokmd_types::ExportReceipt =
        serde_json::from_str(&json_str).expect("deserialize round-trip");

    assert_eq!(receipt.mode, back.mode);
    assert_eq!(receipt.data.rows.len(), back.data.rows.len());
}

// ============================================================================
// 3. Scan → Analysis → Analysis-format pipeline (feature-gated)
// ============================================================================

#[cfg(feature = "analysis")]
#[test]
fn pipeline_analysis_receipt_preset() {
    let dir = fixture();
    let analyze = tokmd_core::settings::AnalyzeSettings::default();
    let receipt = tokmd_core::analyze_workflow(&scan_for(&dir), &analyze)
        .expect("analyze_workflow should succeed");

    assert_eq!(receipt.mode, "analysis");
    assert_eq!(
        receipt.schema_version,
        tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION
    );
    assert!(receipt.derived.is_some(), "receipt preset includes derived");
}

#[cfg(feature = "analysis")]
#[test]
fn pipeline_analysis_derived_serializes_to_json() {
    let dir = fixture();
    let analyze = tokmd_core::settings::AnalyzeSettings::default();
    let receipt = tokmd_core::analyze_workflow(&scan_for(&dir), &analyze).unwrap();

    let json = serde_json::to_value(receipt).expect("serialize");
    assert!(json["derived"].is_object());
    assert!(json["schema_version"].is_number());
}

#[cfg(feature = "analysis")]
#[test]
fn pipeline_analysis_deterministic() {
    let dir = fixture();
    let analyze = tokmd_core::settings::AnalyzeSettings::default();

    let r1 = tokmd_core::analyze_workflow(&scan_for(&dir), &analyze).unwrap();
    let r2 = tokmd_core::analyze_workflow(&scan_for(&dir), &analyze).unwrap();

    let mut j1 = serde_json::to_value(r1).unwrap();
    let mut j2 = serde_json::to_value(r2).unwrap();
    strip_volatile(&mut j1);
    strip_volatile(&mut j2);
    assert_eq!(j1, j2, "analysis output should be deterministic");
}

// ============================================================================
// 4. lang_workflow: receipt with all required fields
// ============================================================================

#[test]
fn lang_receipt_has_all_envelope_fields() {
    let dir = fixture();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();
    let json = serde_json::to_value(receipt).expect("serialize");

    for field in [
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "status",
        "scan",
        "args",
        "rows",
        "total",
    ] {
        assert!(json.get(field).is_some(), "missing field: {field}");
    }
}

#[test]
fn lang_receipt_tool_info_populated() {
    let dir = fixture();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();

    assert!(!receipt.tool.name.is_empty());
    assert!(!receipt.tool.version.is_empty());
    assert!(receipt.tool.version.contains('.'));
}

#[test]
fn lang_receipt_timestamp_reasonable() {
    let dir = fixture();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();
    // After 2024-01-01
    assert!(receipt.generated_at_ms > 1_704_067_200_000);
}

#[test]
fn lang_receipt_status_complete() {
    let dir = fixture();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();
    assert_eq!(format!("{:?}", receipt.status), "Complete");
}

#[test]
fn lang_receipt_rows_have_positive_metrics() {
    let dir = fixture();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();

    for row in &receipt.report.rows {
        assert!(row.code > 0, "{} should have code > 0", row.lang);
        assert!(row.files > 0, "{} should have files > 0", row.lang);
        assert!(row.lines > 0, "{} should have lines > 0", row.lang);
    }
}

// ============================================================================
// 5. module_workflow: receipt with correct depth
// ============================================================================

#[test]
fn module_receipt_respects_depth_1() {
    let dir = fixture();
    let module = ModuleSettings {
        module_depth: 1,
        ..Default::default()
    };
    let receipt = module_workflow(&scan_for(&dir), &module).unwrap();

    assert_eq!(receipt.args.module_depth, 1);
    assert_eq!(receipt.report.module_depth, 1);
}

#[test]
fn module_receipt_respects_depth_3() {
    let dir = fixture();
    let module = ModuleSettings {
        module_depth: 3,
        ..Default::default()
    };
    let receipt = module_workflow(&scan_for(&dir), &module).unwrap();

    assert_eq!(receipt.args.module_depth, 3);
    assert_eq!(receipt.report.module_depth, 3);
}

#[test]
fn module_receipt_has_valid_envelope() {
    let dir = fixture();
    let receipt = module_workflow(&scan_for(&dir), &ModuleSettings::default()).unwrap();

    assert_eq!(receipt.mode, "module");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert_eq!(format!("{:?}", receipt.status), "Complete");
    assert!(!receipt.report.rows.is_empty());
}

// ============================================================================
// 6. export_workflow: all requested formats
// ============================================================================

#[test]
fn export_receipt_contains_all_fixture_files() {
    let dir = fixture();
    let receipt = export_workflow(&scan_for(&dir), &ExportSettings::default()).unwrap();

    let paths: Vec<&str> = receipt.data.rows.iter().map(|r| r.path.as_str()).collect();
    assert!(
        paths.iter().any(|p| p.ends_with("main.rs")),
        "missing main.rs in {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.ends_with("util.py")),
        "missing util.py in {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.ends_with("lib.rs")),
        "missing lib.rs in {paths:?}"
    );
}

#[test]
fn export_forward_slash_paths() {
    let dir = fixture();
    let receipt = export_workflow(&scan_for(&dir), &ExportSettings::default()).unwrap();

    for row in &receipt.data.rows {
        assert!(!row.path.contains('\\'), "backslash in path: {}", row.path);
    }
}

#[test]
fn export_min_code_filter_reduces_rows() {
    let dir = fixture();
    let scan = scan_for(&dir);

    let all = export_workflow(&scan, &ExportSettings::default()).unwrap();
    let filtered = export_workflow(
        &scan,
        &ExportSettings {
            min_code: 99999,
            ..Default::default()
        },
    )
    .unwrap();

    assert!(filtered.data.rows.len() < all.data.rows.len());
}

#[test]
fn export_max_rows_limits_output() {
    let dir = fixture();
    let receipt = export_workflow(
        &scan_for(&dir),
        &ExportSettings {
            max_rows: 1,
            ..Default::default()
        },
    )
    .unwrap();

    assert!(receipt.data.rows.len() <= 1);
}

#[test]
fn export_rows_serializable_as_jsonl() {
    let dir = fixture();
    let receipt = export_workflow(&scan_for(&dir), &ExportSettings::default()).unwrap();

    for row in &receipt.data.rows {
        let line = serde_json::to_string(row).expect("row should serialize");
        let parsed: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert!(parsed.get("path").is_some());
        assert!(parsed.get("code").is_some());
    }
}

// ============================================================================
// 7. Multiple workflows on same scan produce consistent totals
// ============================================================================

#[test]
fn lang_and_export_totals_consistent() {
    let dir = fixture();
    let scan = scan_for(&dir);

    let lang = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let export = export_workflow(&scan, &ExportSettings::default()).unwrap();

    // Total code from lang receipt should match sum of export file rows
    let export_code_sum: usize = export.data.rows.iter().map(|r| r.code).sum();
    assert_eq!(
        lang.report.total.code, export_code_sum,
        "lang total.code ({}) should match sum of export rows ({})",
        lang.report.total.code, export_code_sum
    );
}

#[test]
fn lang_and_module_totals_consistent() {
    let dir = fixture();
    let scan = scan_for(&dir);

    let lang = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let module = module_workflow(&scan, &ModuleSettings::default()).unwrap();

    assert_eq!(
        lang.report.total.code, module.report.total.code,
        "lang total.code ({}) should match module total.code ({})",
        lang.report.total.code, module.report.total.code
    );

    assert_eq!(
        lang.report.total.files, module.report.total.files,
        "lang total.files ({}) should match module total.files ({})",
        lang.report.total.files, module.report.total.files
    );
}

#[test]
fn two_lang_runs_identical() {
    let dir = fixture();
    let scan = scan_for(&dir);
    let settings = LangSettings::default();

    let r1 = lang_workflow(&scan, &settings).unwrap();
    let r2 = lang_workflow(&scan, &settings).unwrap();

    let mut j1 = serde_json::to_value(r1).unwrap();
    let mut j2 = serde_json::to_value(r2).unwrap();
    strip_volatile(&mut j1);
    strip_volatile(&mut j2);
    assert_eq!(j1, j2);
}

// ============================================================================
// 8. Diff workflow integration
// ============================================================================

#[test]
fn diff_detects_changes() {
    let d1 = fixture();
    let d2 = fixture_v2();
    let settings = DiffSettings {
        from: d1.path().to_string_lossy().into_owned(),
        to: d2.path().to_string_lossy().into_owned(),
    };

    let receipt = diff_workflow(&settings).unwrap();
    assert_eq!(receipt.mode, "diff");
    assert!(!receipt.diff_rows.is_empty());
    assert!(
        receipt.diff_rows.iter().any(|r| r.delta_code != 0),
        "should detect code changes"
    );
}

#[test]
fn diff_self_has_zero_deltas() {
    let dir = fixture();
    let path = dir.path().to_string_lossy().into_owned();
    let settings = DiffSettings {
        from: path.clone(),
        to: path,
    };

    let receipt = diff_workflow(&settings).unwrap();
    for row in &receipt.diff_rows {
        assert_eq!(row.delta_code, 0, "self-diff delta != 0 for {}", row.lang);
    }
    assert_eq!(receipt.totals.delta_code, 0);
}

// ============================================================================
// 9. FFI layer
// ============================================================================

#[test]
fn ffi_lang_returns_ok_envelope() {
    let dir = fixture();
    let path = dir.path().to_string_lossy().into_owned();
    let args = format!(r#"{{"paths": ["{}"]}}"#, path.replace('\\', "\\\\"));

    let result = tokmd_core::ffi::run_json("lang", &args);
    let v: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["mode"], "lang");
}

#[test]
fn ffi_version_returns_semver() {
    let result = tokmd_core::ffi::run_json("version", "{}");
    let v: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], true);
    let ver = v["data"]["version"].as_str().unwrap();
    assert!(ver.contains('.'));
}
