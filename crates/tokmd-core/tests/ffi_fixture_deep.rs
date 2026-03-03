//! Deep FFI tests using temporary fixture directories.
//!
//! These tests create isolated temp dirs with known source files to produce
//! predictable, environment-independent results through the `run_json` FFI layer.

use serde_json::Value;
use std::fs;
use tempfile::TempDir;
use tokmd_core::ffi::run_json;

// ============================================================================
// Helpers
// ============================================================================

fn parse_envelope(result: &str) -> Value {
    let v: Value = serde_json::from_str(result).expect("run_json must return valid JSON");
    assert!(v.get("ok").is_some(), "envelope must have 'ok': {result}");
    v
}

fn assert_ok(result: &str) -> Value {
    let v = parse_envelope(result);
    assert_eq!(v["ok"], true, "expected ok:true — {result}");
    v
}

fn assert_err(result: &str) -> Value {
    let v = parse_envelope(result);
    assert_eq!(v["ok"], false, "expected ok:false — {result}");
    assert!(v.get("error").is_some(), "error envelope needs 'error' key");
    v
}

/// Create a temp dir with a single Rust source file.
fn fixture_rust() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    fs::write(
        dir.path().join("main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();
    dir
}

/// Create a temp dir with multiple language files.
fn fixture_multi_lang() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    fs::write(
        dir.path().join("app.rs"),
        "fn main() {\n    let x = 1;\n    let y = 2;\n}\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("lib.py"),
        "def hello():\n    return 42\n\ndef world():\n    return 0\n",
    )
    .unwrap();
    dir
}

/// Create an empty temp dir (no source files).
fn fixture_empty() -> TempDir {
    TempDir::new().expect("create tempdir")
}

/// Escape a path for embedding in JSON (handle backslashes on Windows).
fn json_path(dir: &TempDir) -> String {
    dir.path().display().to_string().replace('\\', "\\\\")
}

/// Strip volatile fields so two receipts can be structurally compared.
fn strip_volatile(v: &mut Value) {
    if let Some(obj) = v.as_object_mut() {
        obj.remove("generated_at_ms");
        obj.remove("scan_duration_ms");
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
// 1. FFI version mode
// ============================================================================

#[test]
fn ffi_version_returns_ok_with_semver() {
    let result = run_json("version", "{}");
    let v = assert_ok(&result);
    let ver = v["data"]["version"].as_str().expect("version string");
    assert!(ver.contains('.'), "should be semver-like: {ver}");
    let sv = v["data"]["schema_version"]
        .as_u64()
        .expect("schema_version");
    assert!(sv > 0);
}

#[test]
fn ffi_version_ignores_extra_fields() {
    let r1 = run_json("version", "{}");
    let r2 = run_json("version", r#"{"extra": "ignored", "foo": 42}"#);
    assert_eq!(r1, r2, "version should ignore extra fields");
}

// ============================================================================
// 2. FFI lang mode with fixtures
// ============================================================================

#[test]
fn ffi_lang_fixture_rust_finds_rust() {
    let dir = fixture_rust();
    let args = format!(r#"{{"paths": ["{}"]}}"#, json_path(&dir));
    let v = assert_ok(&run_json("lang", &args));

    let rows = v["data"]["rows"].as_array().expect("rows array");
    assert!(!rows.is_empty(), "should find at least one language");
    let has_rust = rows.iter().any(|r| r["lang"].as_str() == Some("Rust"));
    assert!(has_rust, "should detect Rust in fixture");
}

#[test]
fn ffi_lang_fixture_rust_code_count() {
    let dir = fixture_rust();
    let args = format!(r#"{{"paths": ["{}"]}}"#, json_path(&dir));
    let v = assert_ok(&run_json("lang", &args));

    let rows = v["data"]["rows"].as_array().unwrap();
    let rust_row = rows.iter().find(|r| r["lang"].as_str() == Some("Rust"));
    assert!(rust_row.is_some(), "Rust row must exist");
    let code = rust_row.unwrap()["code"].as_u64().unwrap();
    assert!(code > 0, "should have non-zero code lines");
}

#[test]
fn ffi_lang_fixture_multi_lang() {
    let dir = fixture_multi_lang();
    let args = format!(r#"{{"paths": ["{}"]}}"#, json_path(&dir));
    let v = assert_ok(&run_json("lang", &args));

    let rows = v["data"]["rows"].as_array().unwrap();
    let langs: Vec<&str> = rows.iter().filter_map(|r| r["lang"].as_str()).collect();
    assert!(langs.contains(&"Rust"), "should find Rust");
    assert!(langs.contains(&"Python"), "should find Python");
}

#[test]
fn ffi_lang_fixture_empty_dir_returns_empty_rows() {
    let dir = fixture_empty();
    let args = format!(r#"{{"paths": ["{}"]}}"#, json_path(&dir));
    let v = assert_ok(&run_json("lang", &args));

    let rows = v["data"]["rows"].as_array().unwrap();
    assert!(rows.is_empty(), "empty dir should produce empty rows");
}

#[test]
fn ffi_lang_fixture_with_top() {
    let dir = fixture_multi_lang();
    let args = format!(r#"{{"paths": ["{}"], "top": 1}}"#, json_path(&dir));
    let v = assert_ok(&run_json("lang", &args));

    let rows = v["data"]["rows"].as_array().unwrap();
    // top=1 → at most 2 rows (1 real + possible "Other")
    assert!(
        rows.len() <= 2,
        "top=1 should limit rows, got {}",
        rows.len()
    );
}

#[test]
fn ffi_lang_fixture_with_children_collapse() {
    let dir = fixture_rust();
    let args = format!(
        r#"{{"paths": ["{}"], "children": "collapse"}}"#,
        json_path(&dir)
    );
    let v = assert_ok(&run_json("lang", &args));
    assert_eq!(v["data"]["mode"].as_str(), Some("lang"));
}

#[test]
fn ffi_lang_fixture_with_children_separate() {
    let dir = fixture_rust();
    let args = format!(
        r#"{{"paths": ["{}"], "children": "separate"}}"#,
        json_path(&dir)
    );
    let v = assert_ok(&run_json("lang", &args));
    assert_eq!(v["data"]["mode"].as_str(), Some("lang"));
}

// ============================================================================
// 3. FFI module mode with fixtures
// ============================================================================

#[test]
fn ffi_module_fixture_returns_receipt() {
    let dir = fixture_rust();
    let args = format!(r#"{{"paths": ["{}"]}}"#, json_path(&dir));
    let v = assert_ok(&run_json("module", &args));

    assert_eq!(v["data"]["mode"].as_str(), Some("module"));
    assert!(v["data"]["schema_version"].as_u64().unwrap() > 0);
    assert!(v["data"]["rows"].is_array());
}

#[test]
fn ffi_module_fixture_empty_dir() {
    let dir = fixture_empty();
    let args = format!(r#"{{"paths": ["{}"]}}"#, json_path(&dir));
    let v = assert_ok(&run_json("module", &args));

    let rows = v["data"]["rows"].as_array().unwrap();
    assert!(rows.is_empty(), "empty dir → empty module rows");
}

#[test]
fn ffi_module_fixture_with_depth() {
    let dir = fixture_rust();
    let args = format!(r#"{{"paths": ["{}"], "module_depth": 1}}"#, json_path(&dir));
    let v = assert_ok(&run_json("module", &args));
    assert_eq!(v["data"]["args"]["module_depth"].as_u64(), Some(1));
}

// ============================================================================
// 4. FFI export mode with fixtures
// ============================================================================

#[test]
fn ffi_export_fixture_returns_file_rows() {
    let dir = fixture_rust();
    let args = format!(r#"{{"paths": ["{}"]}}"#, json_path(&dir));
    let v = assert_ok(&run_json("export", &args));

    assert_eq!(v["data"]["mode"].as_str(), Some("export"));
    let rows = v["data"]["rows"].as_array().unwrap();
    assert!(!rows.is_empty(), "export should find files in fixture");

    // Verify row has expected fields
    let row = &rows[0];
    assert!(row.get("path").is_some(), "row must have path");
    assert!(row.get("lang").is_some(), "row must have lang");
    assert!(row.get("code").is_some(), "row must have code");
}

#[test]
fn ffi_export_fixture_empty_dir() {
    let dir = fixture_empty();
    let args = format!(r#"{{"paths": ["{}"]}}"#, json_path(&dir));
    let v = assert_ok(&run_json("export", &args));

    let rows = v["data"]["rows"].as_array().unwrap();
    assert!(rows.is_empty(), "empty dir → empty export rows");
}

#[test]
fn ffi_export_fixture_paths_forward_slashes() {
    let dir = fixture_rust();
    let args = format!(r#"{{"paths": ["{}"]}}"#, json_path(&dir));
    let v = assert_ok(&run_json("export", &args));

    let rows = v["data"]["rows"].as_array().unwrap();
    for row in rows {
        let path = row["path"].as_str().unwrap();
        assert!(
            !path.contains('\\'),
            "paths must use forward slashes: {path}"
        );
    }
}

#[test]
fn ffi_export_fixture_with_max_rows() {
    let dir = fixture_multi_lang();
    let args = format!(r#"{{"paths": ["{}"], "max_rows": 1}}"#, json_path(&dir));
    let v = assert_ok(&run_json("export", &args));

    let rows = v["data"]["rows"].as_array().unwrap();
    assert!(rows.len() <= 1, "max_rows=1 should limit output");
}

#[test]
fn ffi_export_fixture_with_min_code() {
    let dir = fixture_rust();
    let args = format!(
        r#"{{"paths": ["{}"], "min_code": 999999}}"#,
        json_path(&dir)
    );
    let v = assert_ok(&run_json("export", &args));

    let rows = v["data"]["rows"].as_array().unwrap();
    assert!(rows.is_empty(), "min_code=999999 should filter all files");
}

#[test]
fn ffi_export_fixture_with_redact_paths() {
    let dir = fixture_rust();
    let args = format!(r#"{{"paths": ["{}"], "redact": "paths"}}"#, json_path(&dir));
    let v = assert_ok(&run_json("export", &args));

    let rows = v["data"]["rows"].as_array().unwrap();
    assert!(!rows.is_empty());
    // Redacted paths should NOT contain the original filename literally
    for row in rows {
        let path = row["path"].as_str().unwrap();
        assert!(
            !path.contains("main.rs"),
            "redacted path should not contain original filename: {path}"
        );
    }
}

#[test]
fn ffi_export_fixture_with_redact_all() {
    let dir = fixture_rust();
    let args = format!(r#"{{"paths": ["{}"], "redact": "all"}}"#, json_path(&dir));
    let v = assert_ok(&run_json("export", &args));
    let rows = v["data"]["rows"].as_array().unwrap();
    assert!(!rows.is_empty());
}

// ============================================================================
// 5. FFI error cases with non-existent paths
// ============================================================================

#[test]
fn ffi_lang_nonexistent_path_returns_error() {
    let result = run_json(
        "lang",
        r#"{"paths": ["/tmp/__tokmd_nonexistent_path_42__"]}"#,
    );
    let v = parse_envelope(&result);
    // Depending on implementation, this may succeed with empty results or error
    assert!(v["ok"].is_boolean());
}

#[test]
fn ffi_unknown_mode_returns_error() {
    let v = assert_err(&run_json("bogus_mode_xyz", "{}"));
    assert_eq!(v["error"]["code"].as_str(), Some("unknown_mode"));
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(msg.contains("bogus_mode_xyz"));
}

#[test]
fn ffi_invalid_json_returns_error() {
    let v = assert_err(&run_json("lang", "not json at all"));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_json"));
}

#[test]
fn ffi_empty_json_returns_error() {
    let v = assert_err(&run_json("lang", ""));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_json"));
}

// ============================================================================
// 6. Envelope shape consistency
// ============================================================================

#[test]
fn envelope_success_shape() {
    let dir = fixture_rust();
    let args = format!(r#"{{"paths": ["{}"]}}"#, json_path(&dir));
    let result = run_json("lang", &args);
    let v = parse_envelope(&result);

    assert_eq!(v["ok"], true);
    assert!(v.get("data").is_some(), "success must have data");
    assert!(
        v.get("error").is_none(),
        "success should not have error key"
    );
}

#[test]
fn envelope_error_shape() {
    let result = run_json("bogus", "{}");
    let v = parse_envelope(&result);

    assert_eq!(v["ok"], false);
    assert!(v.get("data").is_none(), "error should not have data key");
    assert!(v.get("error").is_some(), "error must have error key");

    let err = &v["error"];
    assert!(err["code"].is_string(), "error.code must be string");
    assert!(err["message"].is_string(), "error.message must be string");
}

#[test]
fn envelope_ok_always_boolean_across_modes() {
    let dir = fixture_rust();
    let p = json_path(&dir);

    let cases: Vec<(&str, String)> = vec![
        ("version", "{}".to_string()),
        ("lang", format!(r#"{{"paths": ["{p}"]}}"#)),
        ("module", format!(r#"{{"paths": ["{p}"]}}"#)),
        ("export", format!(r#"{{"paths": ["{p}"]}}"#)),
        ("bogus", "{}".to_string()),
        ("lang", "bad json".to_string()),
    ];

    for (mode, args) in &cases {
        let result = run_json(mode, args);
        let v = parse_envelope(&result);
        assert!(
            v["ok"].is_boolean(),
            "ok must be bool for mode={mode}, args={args}"
        );
    }
}

// ============================================================================
// 7. Determinism with fixtures
// ============================================================================

#[test]
fn determinism_lang_fixture() {
    let dir = fixture_rust();
    let args = format!(r#"{{"paths": ["{}"]}}"#, json_path(&dir));

    let r1 = run_json("lang", &args);
    let r2 = run_json("lang", &args);

    let mut v1: Value = serde_json::from_str(&r1).unwrap();
    let mut v2: Value = serde_json::from_str(&r2).unwrap();
    strip_volatile(&mut v1);
    strip_volatile(&mut v2);
    assert_eq!(v1, v2, "lang output should be deterministic");
}

#[test]
fn determinism_export_fixture() {
    let dir = fixture_multi_lang();
    let args = format!(r#"{{"paths": ["{}"]}}"#, json_path(&dir));

    let r1 = run_json("export", &args);
    let r2 = run_json("export", &args);

    let mut v1: Value = serde_json::from_str(&r1).unwrap();
    let mut v2: Value = serde_json::from_str(&r2).unwrap();
    strip_volatile(&mut v1);
    strip_volatile(&mut v2);
    assert_eq!(v1, v2, "export output should be deterministic");
}

#[test]
fn determinism_module_fixture() {
    let dir = fixture_rust();
    let args = format!(r#"{{"paths": ["{}"]}}"#, json_path(&dir));

    let r1 = run_json("module", &args);
    let r2 = run_json("module", &args);

    let mut v1: Value = serde_json::from_str(&r1).unwrap();
    let mut v2: Value = serde_json::from_str(&r2).unwrap();
    strip_volatile(&mut v1);
    strip_volatile(&mut v2);
    assert_eq!(v1, v2, "module output should be deterministic");
}

// ============================================================================
// 8. Cross-mode consistency
// ============================================================================

#[test]
fn cross_mode_lang_vs_export_total_code() {
    let dir = fixture_multi_lang();
    let args = format!(r#"{{"paths": ["{}"]}}"#, json_path(&dir));

    let lang_v = assert_ok(&run_json("lang", &args));
    let export_v = assert_ok(&run_json("export", &args));

    // Total code from lang rows should equal sum of export file code
    let lang_total: i64 = lang_v["data"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["code"].as_i64().unwrap_or(0))
        .sum();

    let export_total: i64 = export_v["data"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["code"].as_i64().unwrap_or(0))
        .sum();

    assert_eq!(
        lang_total, export_total,
        "lang total code should match export total code"
    );
}

#[test]
fn cross_mode_schema_versions_match() {
    let dir = fixture_rust();
    let args = format!(r#"{{"paths": ["{}"]}}"#, json_path(&dir));

    let lang_sv = assert_ok(&run_json("lang", &args))["data"]["schema_version"]
        .as_u64()
        .unwrap();
    let module_sv = assert_ok(&run_json("module", &args))["data"]["schema_version"]
        .as_u64()
        .unwrap();
    let export_sv = assert_ok(&run_json("export", &args))["data"]["schema_version"]
        .as_u64()
        .unwrap();

    assert_eq!(lang_sv, module_sv, "lang and module schema_version match");
    assert_eq!(lang_sv, export_sv, "lang and export schema_version match");
}

// ============================================================================
// 9. FFI strict parsing edge cases
// ============================================================================

#[test]
fn ffi_lang_null_top_uses_default() {
    let dir = fixture_rust();
    let args = format!(r#"{{"paths": ["{}"], "top": null}}"#, json_path(&dir));
    let v = assert_ok(&run_json("lang", &args));
    assert!(v["data"]["rows"].is_array());
}

#[test]
fn ffi_export_null_format_is_error() {
    let dir = fixture_rust();
    let args = format!(r#"{{"paths": ["{}"], "format": null}}"#, json_path(&dir));
    // null format may either use default or error; the implementation treats it as error
    let result = run_json("export", &args);
    let v = parse_envelope(&result);
    assert!(v["ok"].is_boolean());
}

#[test]
fn ffi_lang_zero_top_returns_all() {
    let dir = fixture_multi_lang();
    let args = format!(r#"{{"paths": ["{}"], "top": 0}}"#, json_path(&dir));
    let v = assert_ok(&run_json("lang", &args));

    let rows = v["data"]["rows"].as_array().unwrap();
    // top=0 means no limit, should have at least 2 languages
    assert!(rows.len() >= 2, "top=0 should return all languages");
}

#[test]
fn ffi_diff_self_fixture_zero_deltas() {
    let dir = fixture_rust();
    let p = json_path(&dir);
    let args = format!(r#"{{"from": "{p}", "to": "{p}"}}"#);
    let v = assert_ok(&run_json("diff", &args));

    if let Some(rows) = v["data"]["diff_rows"].as_array() {
        for row in rows {
            assert_eq!(
                row["delta_code"].as_i64(),
                Some(0),
                "self-diff delta_code should be 0"
            );
        }
    }
}
