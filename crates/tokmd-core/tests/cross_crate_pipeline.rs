//! Cross-crate integration tests that verify the full scan→model→format
//! and scan→analysis pipelines through the tokmd-core library façade.
//!
//! These tests use `tempfile` to create controlled directories with known
//! file contents, ensuring fully deterministic and reproducible results.

use std::fs;
use std::io::Cursor;

use tempfile::TempDir;
use tokmd_core::ffi::run_json;
use tokmd_core::{
    diff_workflow, export_workflow, lang_workflow, module_workflow,
    settings::{DiffSettings, ExportSettings, LangSettings, ModuleSettings, ScanSettings},
};

// ============================================================================
// Helpers
// ============================================================================

/// Create a temp directory with known Rust and Python files.
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

    // Nested module
    fs::create_dir_all(dir.path().join("sub")).unwrap();
    fs::write(
        dir.path().join("sub").join("lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .unwrap();

    dir
}

/// Create a second fixture with different files for diff testing.
fn create_fixture_v2() -> TempDir {
    let dir = TempDir::new().expect("should create temp dir");

    // More Rust code than v1
    fs::write(
        dir.path().join("main.rs"),
        concat!(
            "fn main() {\n",
            "    let x = 1;\n",
            "    let y = 2;\n",
            "    println!(\"{}\", x + y);\n",
            "}\n",
            "\n",
            "fn helper() -> i32 {\n",
            "    42\n",
            "}\n",
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

fn assert_ok(result: &str) -> serde_json::Value {
    let v: serde_json::Value =
        serde_json::from_str(result).expect("run_json must return valid JSON");
    assert_eq!(v["ok"], true, "expected ok:true — {result}");
    v
}

fn assert_err(result: &str) -> serde_json::Value {
    let v: serde_json::Value =
        serde_json::from_str(result).expect("run_json must return valid JSON");
    assert_eq!(v["ok"], false, "expected ok:false — {result}");
    assert!(v.get("error").is_some(), "error envelope needs 'error' key");
    v
}

// ============================================================================
// 1. Scan → Model → Format pipeline
// ============================================================================

#[test]
fn scan_model_format_should_produce_valid_json_receipt() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");

    // Verify receipt structure
    assert_eq!(receipt.mode, "lang");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(!receipt.report.rows.is_empty(), "should find languages");

    // Verify JSON serialization round-trips
    let json = serde_json::to_string(&receipt).expect("should serialize");
    let back: tokmd_types::LangReceipt = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(receipt.schema_version, back.schema_version);
    assert_eq!(receipt.report.rows.len(), back.report.rows.len());
}

#[test]
fn scan_model_format_should_find_known_languages() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");

    let langs: Vec<&str> = receipt
        .report
        .rows
        .iter()
        .map(|r| r.lang.as_str())
        .collect();
    assert!(langs.contains(&"Rust"), "should find Rust, got: {langs:?}");
    assert!(
        langs.contains(&"Python"),
        "should find Python, got: {langs:?}"
    );
}

#[test]
fn scan_model_format_should_be_deterministic() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let lang = LangSettings::default();

    let r1 = lang_workflow(&scan, &lang).expect("first call");
    let r2 = lang_workflow(&scan, &lang).expect("second call");

    // Compare everything except timestamps
    let mut j1: serde_json::Value = serde_json::to_value(&r1).expect("serialize r1");
    let mut j2: serde_json::Value = serde_json::to_value(&r2).expect("serialize r2");

    strip_volatile(&mut j1);
    strip_volatile(&mut j2);
    assert_eq!(j1, j2, "two runs should produce identical output");
}

#[test]
fn scan_model_format_should_have_correct_envelope_metadata() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");

    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(
        receipt.generated_at_ms > 1_577_836_800_000,
        "timestamp after 2020"
    );
    assert!(!receipt.tool.name.is_empty());
    assert!(!receipt.tool.version.is_empty());
    assert_eq!(format!("{:?}", receipt.status), "Complete");
    assert!(!receipt.scan.paths.is_empty());
}

#[test]
fn module_pipeline_should_find_nested_modules() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let module = ModuleSettings::default();

    let receipt = module_workflow(&scan, &module).expect("module_workflow should succeed");

    assert_eq!(receipt.mode, "module");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(!receipt.report.rows.is_empty(), "should find modules");
}

#[test]
fn module_pipeline_should_be_deterministic() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let module = ModuleSettings::default();

    let r1 = module_workflow(&scan, &module).expect("first call");
    let r2 = module_workflow(&scan, &module).expect("second call");

    let mut j1 = serde_json::to_value(&r1).unwrap();
    let mut j2 = serde_json::to_value(&r2).unwrap();
    strip_volatile(&mut j1);
    strip_volatile(&mut j2);
    assert_eq!(j1, j2, "module output should be deterministic");
}

// ============================================================================
// 2. Scan → Analysis pipeline (feature-gated)
// ============================================================================

#[cfg(feature = "analysis")]
#[test]
fn analysis_pipeline_should_produce_receipt_with_derived_metrics() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let analyze = tokmd_core::settings::AnalyzeSettings::default();

    let receipt =
        tokmd_core::analyze_workflow(&scan, &analyze).expect("analyze_workflow should succeed");

    assert_eq!(receipt.mode, "analysis");
    assert_eq!(
        receipt.schema_version,
        tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION
    );
    assert!(
        receipt.derived.is_some(),
        "receipt preset should include derived metrics"
    );
}

#[cfg(feature = "analysis")]
#[test]
fn analysis_pipeline_should_have_all_expected_sections() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let analyze = tokmd_core::settings::AnalyzeSettings::default();

    let receipt =
        tokmd_core::analyze_workflow(&scan, &analyze).expect("analyze_workflow should succeed");

    // Receipt preset includes derived metrics
    let derived = receipt
        .derived
        .as_ref()
        .expect("should have derived section");

    // Derived should have code density and distribution info
    let json = serde_json::to_value(&derived).expect("derived should serialize");
    assert!(json.is_object(), "derived should be a JSON object");
}

#[cfg(feature = "analysis")]
#[test]
fn analysis_pipeline_should_be_deterministic() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let analyze = tokmd_core::settings::AnalyzeSettings::default();

    let r1 = tokmd_core::analyze_workflow(&scan, &analyze).expect("first call");
    let r2 = tokmd_core::analyze_workflow(&scan, &analyze).expect("second call");

    let mut j1 = serde_json::to_value(&r1).unwrap();
    let mut j2 = serde_json::to_value(&r2).unwrap();
    strip_volatile(&mut j1);
    strip_volatile(&mut j2);
    assert_eq!(j1, j2, "analysis output should be deterministic");
}

// ============================================================================
// 3. Export workflow (JSONL / CSV)
// ============================================================================

#[test]
fn export_workflow_should_produce_file_rows_for_known_files() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).expect("export_workflow should succeed");

    assert_eq!(receipt.mode, "export");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(!receipt.data.rows.is_empty(), "should find files");

    // Should find our known files
    let paths: Vec<&str> = receipt.data.rows.iter().map(|r| r.path.as_str()).collect();
    assert!(
        paths.iter().any(|p| p.ends_with("main.rs")),
        "should find main.rs, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.ends_with("helper.py")),
        "should find helper.py, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p.ends_with("sub/lib.rs")),
        "should find sub/lib.rs, got: {paths:?}"
    );
}

#[test]
fn export_as_jsonl_should_produce_valid_json_lines() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).expect("export_workflow should succeed");

    // Serialize each row as a JSON line
    for row in &receipt.data.rows {
        let line = serde_json::to_string(row).expect("row should serialize to JSON");
        let parsed: serde_json::Value =
            serde_json::from_str(&line).expect("each JSONL line should be valid JSON");
        assert!(parsed.is_object(), "each line should be a JSON object");
        assert!(parsed.get("path").is_some(), "row should have path");
        assert!(parsed.get("lang").is_some(), "row should have lang");
        assert!(parsed.get("code").is_some(), "row should have code");
    }
}

#[test]
fn export_as_csv_via_format_should_have_header_and_data_rows() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let export_settings = ExportSettings::default();

    let receipt = export_workflow(&scan, &export_settings).expect("export_workflow should succeed");

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
    tokmd_format::write_export_csv_to(&mut buf, &receipt.data, &export_args)
        .expect("CSV write should succeed");

    let csv_output = String::from_utf8(buf.into_inner()).expect("should be valid UTF-8");
    let lines: Vec<&str> = csv_output.lines().collect();

    // Should have header + data rows
    assert!(
        lines.len() >= 2,
        "CSV should have header + data, got {}",
        lines.len()
    );

    // Header should contain expected columns
    let header = lines[0];
    assert!(header.contains("path"), "CSV header should contain 'path'");
    assert!(header.contains("lang"), "CSV header should contain 'lang'");
    assert!(header.contains("code"), "CSV header should contain 'code'");

    // Data rows should have same number of columns as header
    let header_cols = header.split(',').count();
    for (i, line) in lines.iter().enumerate().skip(1) {
        let cols = line.split(',').count();
        assert_eq!(
            cols, header_cols,
            "row {i} should have {header_cols} columns, got {cols}"
        );
    }
}

#[test]
fn export_paths_should_use_forward_slashes() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).expect("export_workflow should succeed");

    for row in &receipt.data.rows {
        assert!(
            !row.path.contains('\\'),
            "path should use forward slashes: {}",
            row.path
        );
    }
}

#[test]
fn export_should_be_deterministic() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let export = ExportSettings::default();

    let r1 = export_workflow(&scan, &export).expect("first call");
    let r2 = export_workflow(&scan, &export).expect("second call");

    let mut j1 = serde_json::to_value(&r1).unwrap();
    let mut j2 = serde_json::to_value(&r2).unwrap();
    strip_volatile(&mut j1);
    strip_volatile(&mut j2);
    assert_eq!(j1, j2, "export output should be deterministic");
}

// ============================================================================
// 4. Diff workflow
// ============================================================================

#[test]
fn diff_workflow_should_detect_changes_between_two_dirs() {
    let dir1 = create_fixture();
    let dir2 = create_fixture_v2();

    let settings = DiffSettings {
        from: dir1.path().to_string_lossy().into_owned(),
        to: dir2.path().to_string_lossy().into_owned(),
    };

    let receipt = diff_workflow(&settings).expect("diff_workflow should succeed");

    assert_eq!(receipt.mode, "diff");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(!receipt.diff_rows.is_empty(), "should have diff rows");

    // dir2 has only Rust (more code), dir1 has Rust + Python
    // So there should be non-zero deltas
    let has_nonzero_delta = receipt.diff_rows.iter().any(|r| r.delta_code != 0);
    assert!(
        has_nonzero_delta,
        "should detect code changes between different dirs"
    );
}

#[test]
fn diff_workflow_self_diff_should_have_zero_deltas() {
    let dir = create_fixture();
    let path = dir.path().to_string_lossy().into_owned();

    let settings = DiffSettings {
        from: path.clone(),
        to: path,
    };

    let receipt = diff_workflow(&settings).expect("diff_workflow should succeed");

    for row in &receipt.diff_rows {
        assert_eq!(
            row.delta_code, 0,
            "self-diff should have zero delta_code for {}",
            row.lang
        );
    }
    assert_eq!(
        receipt.totals.delta_code, 0,
        "self-diff totals should be zero"
    );
}

#[test]
fn diff_receipt_should_have_correct_structure() {
    let dir = create_fixture();
    let path = dir.path().to_string_lossy().into_owned();

    let settings = DiffSettings {
        from: path.clone(),
        to: path,
    };

    let receipt = diff_workflow(&settings).expect("diff_workflow should succeed");

    // Verify JSON serialization
    let json = serde_json::to_value(&receipt).expect("should serialize");
    assert!(json.get("schema_version").is_some());
    assert!(json.get("mode").is_some());
    assert!(json.get("diff_rows").is_some());
    assert!(json.get("totals").is_some());
    assert!(json.get("from_source").is_some());
    assert!(json.get("to_source").is_some());
}

// ============================================================================
// 5. FFI run_json round-trip
// ============================================================================

#[test]
fn ffi_lang_with_tempdir_should_return_valid_receipt() {
    let dir = create_fixture();
    let path = dir.path().to_string_lossy().into_owned();
    let args = format!(r#"{{"paths": ["{}"]}}"#, path.replace('\\', "\\\\"));

    let result = run_json("lang", &args);
    let v = assert_ok(&result);

    let data = &v["data"];
    assert_eq!(data["mode"].as_str(), Some("lang"));
    assert_eq!(
        data["schema_version"].as_u64(),
        Some(u64::from(tokmd_types::SCHEMA_VERSION))
    );

    // Should find our known languages
    let rows = data["rows"].as_array().expect("rows should be an array");
    let langs: Vec<&str> = rows.iter().filter_map(|r| r["lang"].as_str()).collect();
    assert!(langs.contains(&"Rust"), "FFI should find Rust");
    assert!(langs.contains(&"Python"), "FFI should find Python");
}

#[test]
fn ffi_version_should_return_version_info() {
    let result = run_json("version", "{}");
    let v = assert_ok(&result);

    let data = &v["data"];
    let version = data["version"].as_str().expect("version should be string");
    assert!(version.contains('.'), "version should be semver-like");

    let sv = data["schema_version"]
        .as_u64()
        .expect("schema_version should be number");
    assert_eq!(sv, u64::from(tokmd_types::SCHEMA_VERSION));
}

#[test]
fn ffi_invalid_mode_should_return_error_envelope() {
    let result = run_json("nonexistent_mode", "{}");
    let v = assert_err(&result);

    assert_eq!(v["error"]["code"].as_str(), Some("unknown_mode"));
    assert!(v.get("data").is_none(), "error should not have data");
}

#[test]
fn ffi_export_with_tempdir_should_find_known_files() {
    let dir = create_fixture();
    let path = dir.path().to_string_lossy().into_owned();
    let args = format!(r#"{{"paths": ["{}"]}}"#, path.replace('\\', "\\\\"));

    let result = run_json("export", &args);
    let v = assert_ok(&result);

    let rows = v["data"]["rows"]
        .as_array()
        .expect("rows should be an array");
    assert!(!rows.is_empty(), "should find files");

    // Verify each row has required fields
    for row in rows {
        assert!(row.get("path").is_some(), "row should have path");
        assert!(row.get("lang").is_some(), "row should have lang");
        assert!(row.get("code").is_some(), "row should have code");
    }
}

#[test]
fn ffi_module_with_tempdir_should_return_valid_receipt() {
    let dir = create_fixture();
    let path = dir.path().to_string_lossy().into_owned();
    let args = format!(r#"{{"paths": ["{}"]}}"#, path.replace('\\', "\\\\"));

    let result = run_json("module", &args);
    let v = assert_ok(&result);

    assert_eq!(v["data"]["mode"].as_str(), Some("module"));
    assert!(v["data"]["rows"].is_array());
}

#[test]
fn ffi_diff_with_two_tempdirs_should_detect_changes() {
    let dir1 = create_fixture();
    let dir2 = create_fixture_v2();
    let from = dir1
        .path()
        .to_string_lossy()
        .into_owned()
        .replace('\\', "\\\\");
    let to = dir2
        .path()
        .to_string_lossy()
        .into_owned()
        .replace('\\', "\\\\");
    let args = format!(r#"{{"from": "{from}", "to": "{to}"}}"#);

    let result = run_json("diff", &args);
    let v = assert_ok(&result);

    assert_eq!(v["data"]["mode"].as_str(), Some("diff"));
    assert!(v["data"]["diff_rows"].is_array());
}

#[test]
fn ffi_roundtrip_should_match_workflow_output() {
    let dir = create_fixture();
    let path = dir.path().to_string_lossy().into_owned();
    let scan = ScanSettings::for_paths(vec![path.clone()]);
    let lang = LangSettings::default();

    // Direct workflow call
    let direct = lang_workflow(&scan, &lang).expect("direct call");

    // FFI call
    let args = format!(r#"{{"paths": ["{}"]}}"#, path.replace('\\', "\\\\"));
    let ffi_result = run_json("lang", &args);
    let ffi_v = assert_ok(&ffi_result);

    // Both should find the same languages and counts
    let ffi_rows = ffi_v["data"]["rows"].as_array().expect("ffi rows");
    assert_eq!(
        direct.report.rows.len(),
        ffi_rows.len(),
        "workflow and FFI should produce same number of rows"
    );

    for (direct_row, ffi_row) in direct.report.rows.iter().zip(ffi_rows.iter()) {
        assert_eq!(
            direct_row.lang,
            ffi_row["lang"].as_str().unwrap_or(""),
            "language should match"
        );
        assert_eq!(
            direct_row.code as u64,
            ffi_row["code"].as_u64().unwrap_or(0),
            "code count should match for {}",
            direct_row.lang
        );
    }
}

// ============================================================================
// 6. Cross-cutting: format rendering through the pipeline
// ============================================================================

#[test]
fn lang_receipt_json_should_contain_all_envelope_fields() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow");
    let json = serde_json::to_value(&receipt).expect("should serialize");

    let required_fields = [
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "status",
        "scan",
        "args",
        "rows",
    ];
    for field in &required_fields {
        assert!(
            json.get(*field).is_some(),
            "receipt JSON missing required field: {field}"
        );
    }
}

#[test]
fn export_receipt_json_should_have_correct_data_shape() {
    let dir = create_fixture();
    let scan = scan_for(&dir);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).expect("export_workflow");
    let json = serde_json::to_value(&receipt).expect("should serialize");

    // ExportReceipt uses #[serde(flatten)] for data, so rows appear at top level
    assert!(json.get("rows").is_some(), "should have rows field");
    assert!(json.get("schema_version").is_some());
    assert!(json.get("mode").is_some());

    let rows = json["rows"].as_array().expect("rows should be array");
    for row in rows {
        // Each FileRow should have these fields
        assert!(row.get("path").is_some());
        assert!(row.get("lang").is_some());
        assert!(row.get("code").is_some());
        assert!(row.get("comments").is_some());
        assert!(row.get("blanks").is_some());
        assert!(row.get("lines").is_some());
    }
}
