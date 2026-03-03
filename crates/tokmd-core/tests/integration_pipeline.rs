//! Full cross-crate integration tests that verify the complete
//! scan → model → format and scan → analysis pipelines at the library level.
//!
//! Each test exercises multiple crate boundaries in a single invocation,
//! ensuring the entire pipeline composes correctly end-to-end.

use std::fs;
use std::io::Cursor;

use tempfile::TempDir;
use tokmd_core::ffi::run_json;
use tokmd_core::{
    diff_workflow, export_workflow, lang_workflow, module_workflow,
    settings::{DiffSettings, ExportSettings, LangSettings, ModuleSettings, ScanSettings},
};
use tokmd_types::SCHEMA_VERSION;

// ============================================================================
// Helpers
// ============================================================================

/// Create a temp directory with known source files across multiple languages.
fn create_fixture() -> TempDir {
    let dir = TempDir::new().expect("should create temp dir");

    // Rust file
    fs::write(
        dir.path().join("main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();

    // Python file
    fs::write(
        dir.path().join("helper.py"),
        "def greet():\n    print(\"hello\")\n",
    )
    .unwrap();

    // Nested module directory
    fs::create_dir_all(dir.path().join("sub")).unwrap();
    fs::write(
        dir.path().join("sub").join("lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .unwrap();

    dir
}

/// Create an empty temp directory with no source files.
fn create_empty_fixture() -> TempDir {
    TempDir::new().expect("should create temp dir")
}

fn scan_for(dir: &TempDir) -> ScanSettings {
    ScanSettings::for_paths(vec![dir.path().to_string_lossy().into_owned()])
}

// ============================================================================
// 1. Scan → Model → Format: Lang → JSON pipeline
// ============================================================================

#[test]
fn lang_json_pipeline_scan_model_format_parse_verify() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let lang = LangSettings::default();

    // Scan → Model → Receipt
    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow");

    // Format as JSON
    let json_str = serde_json::to_string_pretty(&receipt).expect("serialize");

    // Parse JSON back
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("parse");

    // Verify structure
    assert!(parsed.is_object());
    assert_eq!(
        parsed["schema_version"].as_u64(),
        Some(u64::from(SCHEMA_VERSION))
    );
    assert_eq!(parsed["mode"].as_str(), Some("lang"));
    assert!(parsed["rows"].is_array());
    assert!(parsed["total"].is_object());
    assert!(parsed["tool"].is_object());
    assert!(parsed["scan"].is_object());
    assert!(parsed["args"].is_object());

    // Verify rows have expected shape
    let rows = parsed["rows"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        assert!(row["lang"].is_string());
        assert!(row["code"].is_number());
        assert!(row["lines"].is_number());
        assert!(row["files"].is_number());
        assert!(row["bytes"].is_number());
        assert!(row["tokens"].is_number());
    }
}

// ============================================================================
// 2. Scan → Model → Format: Module → Markdown pipeline
// ============================================================================

#[test]
fn module_markdown_pipeline_produces_valid_table() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let module = ModuleSettings::default();

    // Scan → Model → Receipt
    let receipt = module_workflow(&scan, &module).expect("module_workflow");

    // Format as Markdown via the format crate
    let module_args = tokmd_types::ModuleArgs {
        paths: vec![dir.path().to_path_buf()],
        format: tokmd_types::TableFormat::Md,
        top: 0,
        module_roots: vec![],
        module_depth: 2,
        children: tokmd_types::ChildIncludeMode::Separate,
    };
    let global = tokmd_settings::ScanOptions::default();

    let mut buf = Cursor::new(Vec::new());
    tokmd_format::write_module_report_to(&mut buf, &receipt.report, &global, &module_args)
        .expect("markdown write");

    let md = String::from_utf8(buf.into_inner()).expect("valid UTF-8");

    // Markdown tables must have pipe-delimited rows and a separator
    assert!(md.contains('|'), "markdown should contain table pipes");
    assert!(md.contains("---"), "markdown should contain separator row");

    // Should have at least header + separator + one data row
    let lines: Vec<&str> = md.lines().filter(|l| l.contains('|')).collect();
    assert!(
        lines.len() >= 3,
        "markdown table needs header + separator + data, got {} lines",
        lines.len()
    );
}

// ============================================================================
// 3. Scan → Model → Format: Export → CSV pipeline
// ============================================================================

#[test]
fn export_csv_pipeline_produces_valid_csv() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let export_settings = ExportSettings::default();

    let receipt = export_workflow(&scan, &export_settings).expect("export_workflow");

    let export_args = tokmd_types::ExportArgs {
        paths: vec![dir.path().to_path_buf()],
        format: tokmd_types::ExportFormat::Csv,
        output: None,
        module_roots: vec![],
        module_depth: 2,
        children: tokmd_types::ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: tokmd_types::RedactMode::None,
        meta: false,
        strip_prefix: None,
    };

    let mut buf = Cursor::new(Vec::new());
    tokmd_format::write_export_csv_to(&mut buf, &receipt.data, &export_args).expect("CSV write");

    let csv_output = String::from_utf8(buf.into_inner()).expect("valid UTF-8");
    let lines: Vec<&str> = csv_output.lines().collect();

    // Header + at least 3 data rows (main.rs, helper.py, sub/lib.rs)
    assert!(
        lines.len() >= 4,
        "CSV should have header + 3+ data rows, got {}",
        lines.len()
    );

    // Verify header columns
    let header = lines[0];
    for col in &[
        "path", "module", "lang", "kind", "code", "comments", "blanks", "lines", "bytes", "tokens",
    ] {
        assert!(header.contains(col), "CSV header missing column '{col}'");
    }

    // Verify consistent column count
    let header_cols = header.split(',').count();
    for (i, line) in lines.iter().enumerate().skip(1) {
        assert_eq!(
            line.split(',').count(),
            header_cols,
            "row {i} column count mismatch"
        );
    }
}

// ============================================================================
// 4. Scan → Model → Format: Export → JSONL pipeline
// ============================================================================

#[test]
fn export_jsonl_pipeline_each_line_is_valid_json() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let export_settings = ExportSettings::default();

    let receipt = export_workflow(&scan, &export_settings).expect("export_workflow");

    let export_args = tokmd_types::ExportArgs {
        paths: vec![dir.path().to_path_buf()],
        format: tokmd_types::ExportFormat::Jsonl,
        output: None,
        module_roots: vec![],
        module_depth: 2,
        children: tokmd_types::ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: tokmd_types::RedactMode::None,
        meta: false,
        strip_prefix: None,
    };
    let global = tokmd_settings::ScanOptions::default();

    let mut buf = Cursor::new(Vec::new());
    tokmd_format::write_export_jsonl_to(&mut buf, &receipt.data, &global, &export_args)
        .expect("JSONL write");

    let output = String::from_utf8(buf.into_inner()).expect("valid UTF-8");
    let lines: Vec<&str> = output.lines().collect();

    assert!(
        lines.len() >= 3,
        "JSONL should have 3+ lines for known files"
    );

    for (i, line) in lines.iter().enumerate() {
        let parsed: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("JSONL line {i} is invalid JSON: {e}"));
        assert!(parsed.is_object(), "JSONL line {i} should be a JSON object");
        assert!(
            parsed.get("path").is_some(),
            "JSONL line {i} missing 'path'"
        );
        assert!(
            parsed.get("lang").is_some(),
            "JSONL line {i} missing 'lang'"
        );
        assert!(
            parsed.get("code").is_some(),
            "JSONL line {i} missing 'code'"
        );
    }
}

#[test]
fn export_jsonl_lines_match_export_receipt_rows() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let export_settings = ExportSettings::default();

    let receipt = export_workflow(&scan, &export_settings).expect("export_workflow");

    // Each row serialized individually should be valid JSON
    for (i, row) in receipt.data.rows.iter().enumerate() {
        let json_line =
            serde_json::to_string(row).unwrap_or_else(|e| panic!("row {i} serialize failed: {e}"));
        let parsed: serde_json::Value = serde_json::from_str(&json_line)
            .unwrap_or_else(|e| panic!("row {i} round-trip failed: {e}"));
        assert_eq!(
            parsed["path"].as_str().unwrap(),
            row.path.as_str(),
            "row {i} path mismatch"
        );
    }
}

// ============================================================================
// 5. Scan → Analysis pipeline (feature-gated)
// ============================================================================

#[cfg(feature = "analysis")]
#[test]
fn analysis_receipt_has_density_distribution_cocomo() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let analyze = tokmd_core::settings::AnalyzeSettings::default();

    let receipt = tokmd_core::analyze_workflow(&scan, &analyze).expect("analyze_workflow");

    let derived = receipt
        .derived
        .as_ref()
        .expect("should have derived metrics");

    // Verify density metrics
    let derived_json = serde_json::to_value(derived).expect("serialize derived");
    assert!(
        derived_json.get("doc_density").is_some(),
        "derived should have doc_density"
    );
    assert!(
        derived_json.get("distribution").is_some(),
        "derived should have distribution"
    );
    assert!(
        derived_json.get("cocomo").is_some(),
        "derived should have cocomo"
    );
}

#[cfg(feature = "analysis")]
#[test]
fn analysis_json_has_schema_version_field() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let analyze = tokmd_core::settings::AnalyzeSettings::default();

    let receipt = tokmd_core::analyze_workflow(&scan, &analyze).expect("analyze_workflow");

    let json = serde_json::to_value(&receipt).expect("serialize");
    assert_eq!(
        json["schema_version"].as_u64(),
        Some(u64::from(tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION)),
        "analysis schema_version should match constant"
    );
    assert!(json.get("mode").is_some());
    assert!(json.get("source").is_some());
    assert!(json.get("args").is_some());
}

#[cfg(feature = "analysis")]
#[test]
fn analysis_cocomo_fields_are_plausible() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let analyze = tokmd_core::settings::AnalyzeSettings::default();

    let receipt = tokmd_core::analyze_workflow(&scan, &analyze).expect("analyze_workflow");

    let derived = receipt.derived.as_ref().expect("derived");
    let cocomo_json = serde_json::to_value(&derived.cocomo).expect("serialize cocomo");
    if let Some(cocomo) = cocomo_json.as_object() {
        // COCOMO should have positive effort and duration for non-empty code
        let kloc = cocomo.get("kloc").and_then(|v| v.as_f64()).unwrap_or(0.0);
        assert!(kloc > 0.0, "kloc should be positive for fixture with code");
    }
}

#[cfg(feature = "analysis")]
#[test]
fn analysis_distribution_stats_present() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let analyze = tokmd_core::settings::AnalyzeSettings::default();

    let receipt = tokmd_core::analyze_workflow(&scan, &analyze).expect("analyze_workflow");

    let derived = receipt.derived.as_ref().expect("derived");
    let dist_json = serde_json::to_value(&derived.distribution).expect("serialize distribution");
    let dist = dist_json.as_object().expect("distribution is object");

    // Distribution should have statistical summary fields
    for field in &["count", "min", "max", "mean", "median"] {
        assert!(
            dist.get(*field).is_some(),
            "distribution missing field '{field}'"
        );
    }
}

// ============================================================================
// 6. Receipt round-trip: JSON → parse → re-serialize → compare (idempotency)
// ============================================================================

#[test]
fn lang_receipt_json_roundtrip_is_idempotent() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow");

    // First serialization
    let json1 = serde_json::to_string(&receipt).expect("serialize 1");

    // Parse back
    let parsed: tokmd_types::LangReceipt = serde_json::from_str(&json1).expect("parse");

    // Re-serialize
    let json2 = serde_json::to_string(&parsed).expect("serialize 2");

    // Must be identical
    assert_eq!(json1, json2, "JSON round-trip should be idempotent");
}

#[test]
fn module_receipt_json_roundtrip_is_idempotent() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let module = ModuleSettings::default();

    let receipt = module_workflow(&scan, &module).expect("module_workflow");

    let json1 = serde_json::to_string(&receipt).expect("serialize 1");
    let parsed: tokmd_types::ModuleReceipt = serde_json::from_str(&json1).expect("parse");
    let json2 = serde_json::to_string(&parsed).expect("serialize 2");

    assert_eq!(
        json1, json2,
        "module receipt round-trip should be idempotent"
    );
}

#[test]
fn export_receipt_json_roundtrip_is_idempotent() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).expect("export_workflow");

    let json1 = serde_json::to_string(&receipt).expect("serialize 1");
    let parsed: tokmd_types::ExportReceipt = serde_json::from_str(&json1).expect("parse");
    let json2 = serde_json::to_string(&parsed).expect("serialize 2");

    assert_eq!(
        json1, json2,
        "export receipt round-trip should be idempotent"
    );
}

#[test]
fn diff_receipt_json_roundtrip_is_idempotent() {
    let dir = create_fixture();
    let path = dir.path().to_string_lossy().into_owned();

    let settings = DiffSettings {
        from: path.clone(),
        to: path,
    };
    let receipt = diff_workflow(&settings).expect("diff_workflow");

    let json1 = serde_json::to_string(&receipt).expect("serialize 1");
    let parsed: tokmd_types::DiffReceipt = serde_json::from_str(&json1).expect("parse");
    let json2 = serde_json::to_string(&parsed).expect("serialize 2");

    assert_eq!(json1, json2, "diff receipt round-trip should be idempotent");
}

// ============================================================================
// 7. Receipt required fields per schema
// ============================================================================

#[test]
fn lang_receipt_has_all_required_fields() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow");
    let json = serde_json::to_value(&receipt).expect("serialize");

    let required = [
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "status",
        "warnings",
        "scan",
        "args",
        "rows",
        "total",
    ];
    for field in &required {
        assert!(
            json.get(*field).is_some(),
            "LangReceipt missing required field '{field}'"
        );
    }
}

#[test]
fn module_receipt_has_all_required_fields() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let module = ModuleSettings::default();

    let receipt = module_workflow(&scan, &module).expect("module_workflow");
    let json = serde_json::to_value(&receipt).expect("serialize");

    let required = [
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "status",
        "warnings",
        "scan",
        "args",
        "rows",
        "total",
    ];
    for field in &required {
        assert!(
            json.get(*field).is_some(),
            "ModuleReceipt missing required field '{field}'"
        );
    }
}

#[test]
fn export_receipt_has_all_required_fields() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).expect("export_workflow");
    let json = serde_json::to_value(&receipt).expect("serialize");

    let required = [
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "status",
        "warnings",
        "scan",
        "args",
        "rows",
        "module_roots",
        "module_depth",
        "children",
    ];
    for field in &required {
        assert!(
            json.get(*field).is_some(),
            "ExportReceipt missing required field '{field}'"
        );
    }
}

#[test]
fn diff_receipt_has_all_required_fields() {
    let dir = create_fixture();
    let path = dir.path().to_string_lossy().into_owned();

    let settings = DiffSettings {
        from: path.clone(),
        to: path,
    };
    let receipt = diff_workflow(&settings).expect("diff_workflow");
    let json = serde_json::to_value(&receipt).expect("serialize");

    let required = [
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "from_source",
        "to_source",
        "diff_rows",
        "totals",
    ];
    for field in &required {
        assert!(
            json.get(*field).is_some(),
            "DiffReceipt missing required field '{field}'"
        );
    }
}

// ============================================================================
// 8. Envelope compliance: schema_version matches constant
// ============================================================================

#[test]
fn lang_receipt_schema_version_matches_constant() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow");
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
}

#[test]
fn module_receipt_schema_version_matches_constant() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let module = ModuleSettings::default();

    let receipt = module_workflow(&scan, &module).expect("module_workflow");
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
}

#[test]
fn export_receipt_schema_version_matches_constant() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).expect("export_workflow");
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
}

#[test]
fn diff_receipt_schema_version_matches_constant() {
    let dir = create_fixture();
    let path = dir.path().to_string_lossy().into_owned();

    let settings = DiffSettings {
        from: path.clone(),
        to: path,
    };
    let receipt = diff_workflow(&settings).expect("diff_workflow");
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
}

#[test]
fn ffi_envelope_has_ok_and_data_keys() {
    let dir = create_fixture();
    let path = dir.path().to_string_lossy().into_owned();
    let args = format!(r#"{{"paths": ["{}"]}}"#, path.replace('\\', "\\\\"));

    let result = run_json("lang", &args);
    let v: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

    assert_eq!(v["ok"], true);
    assert!(v.get("data").is_some(), "success envelope must have 'data'");
    assert!(
        v.get("error").is_none(),
        "success envelope must not have 'error'"
    );
}

#[test]
fn ffi_error_envelope_has_ok_false_and_error_key() {
    let result = run_json("bad_mode", "{}");
    let v: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

    assert_eq!(v["ok"], false);
    assert!(v.get("error").is_some(), "error envelope must have 'error'");
    assert!(v["error"]["code"].is_string(), "error must have code");
    assert!(v["error"]["message"].is_string(), "error must have message");
}

// ============================================================================
// 9. Error propagation: non-existent path
// ============================================================================

#[test]
fn lang_workflow_nonexistent_path_returns_error() {
    let scan = ScanSettings::for_paths(vec![
        "/definitely/does/not/exist/tokmd_test_9999".to_string(),
    ]);
    let lang = LangSettings::default();

    let result = lang_workflow(&scan, &lang);
    assert!(result.is_err(), "non-existent path should produce an error");
}

#[test]
fn module_workflow_nonexistent_path_returns_error() {
    let scan = ScanSettings::for_paths(vec![
        "/definitely/does/not/exist/tokmd_test_9999".to_string(),
    ]);
    let module = ModuleSettings::default();

    let result = module_workflow(&scan, &module);
    assert!(result.is_err(), "non-existent path should produce an error");
}

#[test]
fn export_workflow_nonexistent_path_returns_error() {
    let scan = ScanSettings::for_paths(vec![
        "/definitely/does/not/exist/tokmd_test_9999".to_string(),
    ]);
    let export = ExportSettings::default();

    let result = export_workflow(&scan, &export);
    assert!(result.is_err(), "non-existent path should produce an error");
}

#[test]
fn ffi_nonexistent_path_returns_error_envelope() {
    let args = r#"{"paths": ["/definitely/does/not/exist/tokmd_test_9999"]}"#;
    let result = run_json("lang", args);
    let v: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

    assert_eq!(v["ok"], false, "non-existent path should fail");
    assert!(v.get("error").is_some());
}

// ============================================================================
// 10. Error propagation: empty scan results
// ============================================================================

#[test]
fn lang_workflow_empty_dir_produces_empty_rows() {
    let dir = create_empty_fixture();
    let scan = scan_for(&dir);
    let lang = LangSettings::default();

    // An empty directory should succeed but produce zero rows
    let result = lang_workflow(&scan, &lang);
    match result {
        Ok(receipt) => {
            assert!(
                receipt.report.rows.is_empty(),
                "empty dir should produce no language rows"
            );
            assert_eq!(receipt.report.total.code, 0);
        }
        Err(_) => {
            // Some implementations may error on empty — that's also acceptable
        }
    }
}

#[test]
fn export_workflow_empty_dir_produces_empty_rows() {
    let dir = create_empty_fixture();
    let scan = scan_for(&dir);
    let export = ExportSettings::default();

    let result = export_workflow(&scan, &export);
    match result {
        Ok(receipt) => {
            assert!(
                receipt.data.rows.is_empty(),
                "empty dir should produce no export rows"
            );
        }
        Err(_) => {
            // Also acceptable
        }
    }
}

#[test]
fn module_workflow_empty_dir_graceful() {
    let dir = create_empty_fixture();
    let scan = scan_for(&dir);
    let module = ModuleSettings::default();

    let result = module_workflow(&scan, &module);
    match result {
        Ok(receipt) => {
            assert!(
                receipt.report.rows.is_empty(),
                "empty dir should produce no module rows"
            );
        }
        Err(_) => {
            // Also acceptable
        }
    }
}

// ============================================================================
// 11. Cross-pipeline consistency
// ============================================================================

#[test]
fn lang_and_export_total_code_consistent() {
    let dir = create_fixture();
    let scan = scan_for(&dir);

    let lang_receipt = lang_workflow(&scan, &LangSettings::default()).expect("lang");
    let export_receipt = export_workflow(&scan, &ExportSettings::default()).expect("export");

    let lang_total: usize = lang_receipt.report.rows.iter().map(|r| r.code).sum();
    let export_total: usize = export_receipt.data.rows.iter().map(|r| r.code).sum();

    assert_eq!(
        lang_total, export_total,
        "lang total code ({lang_total}) should match export total code ({export_total})"
    );
}

#[test]
fn all_receipt_modes_have_correct_mode_string() {
    let dir = create_fixture();
    let scan = scan_for(&dir);

    let lang = lang_workflow(&scan, &LangSettings::default()).expect("lang");
    assert_eq!(lang.mode, "lang");

    let module = module_workflow(&scan, &ModuleSettings::default()).expect("module");
    assert_eq!(module.mode, "module");

    let export = export_workflow(&scan, &ExportSettings::default()).expect("export");
    assert_eq!(export.mode, "export");

    let path = dir.path().to_string_lossy().into_owned();
    let diff = diff_workflow(&DiffSettings {
        from: path.clone(),
        to: path,
    })
    .expect("diff");
    assert_eq!(diff.mode, "diff");
}

#[test]
fn tool_info_consistent_across_receipts() {
    let dir = create_fixture();
    let scan = scan_for(&dir);

    let lang = lang_workflow(&scan, &LangSettings::default()).expect("lang");
    let module = module_workflow(&scan, &ModuleSettings::default()).expect("module");
    let export = export_workflow(&scan, &ExportSettings::default()).expect("export");

    // All receipts should report the same tool name and version
    assert_eq!(lang.tool.name, module.tool.name);
    assert_eq!(lang.tool.name, export.tool.name);
    assert_eq!(lang.tool.version, module.tool.version);
    assert_eq!(lang.tool.version, export.tool.version);
    assert_eq!(lang.tool.name, "tokmd");
}

// ============================================================================
// 12. FFI round-trip consistency
// ============================================================================

#[test]
fn ffi_export_envelope_matches_workflow_row_count() {
    let dir = create_fixture();
    let path = dir.path().to_string_lossy().into_owned();
    let scan = scan_for(&dir);

    // Direct workflow
    let direct = export_workflow(&scan, &ExportSettings::default()).expect("direct");

    // FFI call
    let args = format!(r#"{{"paths": ["{}"]}}"#, path.replace('\\', "\\\\"));
    let result = run_json("export", &args);
    let v: serde_json::Value = serde_json::from_str(&result).expect("parse");
    assert_eq!(v["ok"], true);

    let ffi_rows = v["data"]["rows"].as_array().expect("ffi rows");
    assert_eq!(
        direct.data.rows.len(),
        ffi_rows.len(),
        "workflow and FFI should produce the same number of export rows"
    );
}

#[test]
fn ffi_module_envelope_has_correct_schema_version() {
    let dir = create_fixture();
    let path = dir.path().to_string_lossy().into_owned();
    let args = format!(r#"{{"paths": ["{}"]}}"#, path.replace('\\', "\\\\"));

    let result = run_json("module", &args);
    let v: serde_json::Value = serde_json::from_str(&result).expect("parse");
    assert_eq!(v["ok"], true);
    assert_eq!(
        v["data"]["schema_version"].as_u64(),
        Some(u64::from(SCHEMA_VERSION))
    );
}
