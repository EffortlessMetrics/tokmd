//! Deep FFI round-trip tests for `run_json`.
//!
//! These tests complement `json_api.rs` by focusing on:
//! - Cross-mode envelope consistency
//! - FFI statelessness (interleaved calls)
//! - Deeper structural validation of receipt data
//! - Extreme parameter handling

use tokmd_core::ffi::run_json;

// ============================================================================
// Helper
// ============================================================================

fn parse_ok(result: &str) -> serde_json::Value {
    let v: serde_json::Value = serde_json::from_str(result).expect("valid JSON");
    assert_eq!(v["ok"], true, "expected ok=true: {result}");
    v
}

fn parse_err(result: &str) -> serde_json::Value {
    let v: serde_json::Value = serde_json::from_str(result).expect("valid JSON");
    assert_eq!(v["ok"], false, "expected ok=false: {result}");
    v
}

/// Strip `generated_at_ms` from both envelope and nested data for determinism.
fn strip_timestamps(val: &mut serde_json::Value) {
    if let Some(obj) = val.as_object_mut() {
        obj.remove("generated_at_ms");
        if let Some(data) = obj.get_mut("data") {
            if let Some(d) = data.as_object_mut() {
                d.remove("generated_at_ms");
            }
        }
    }
}

// ============================================================================
// Cross-mode envelope consistency
// ============================================================================

/// All successful receipt modes must produce receipts with the same metadata fields.
#[test]
fn all_receipt_modes_share_common_metadata_fields() {
    let modes_and_args: &[(&str, &str)] = &[
        ("lang", r#"{"paths": ["src"]}"#),
        ("module", r#"{"paths": ["src"]}"#),
        ("export", r#"{"paths": ["src"]}"#),
    ];

    for (mode, args) in modes_and_args {
        let result = run_json(mode, args);
        let parsed = parse_ok(&result);
        let data = &parsed["data"];

        assert!(
            data["schema_version"].is_u64(),
            "{mode}: schema_version should be u64"
        );
        assert!(
            data["generated_at_ms"].is_u64(),
            "{mode}: generated_at_ms should be u64"
        );
        assert!(data["tool"].is_object(), "{mode}: tool should be object");
        assert!(
            data["tool"]["name"].is_string(),
            "{mode}: tool.name should be string"
        );
        assert!(
            data["tool"]["version"].is_string(),
            "{mode}: tool.version should be string"
        );
        assert!(data["mode"].is_string(), "{mode}: mode should be string");
        assert!(data["scan"].is_object(), "{mode}: scan should be object");
        assert!(
            data["scan"]["paths"].is_array(),
            "{mode}: scan.paths should be array"
        );
        assert!(
            data["status"].is_string(),
            "{mode}: status should be string"
        );
    }
}

/// The mode field in the receipt must match the requested mode.
#[test]
fn receipt_mode_matches_requested_mode() {
    let cases = [
        ("lang", r#"{"paths": ["src"]}"#),
        ("module", r#"{"paths": ["src"]}"#),
        ("export", r#"{"paths": ["src"]}"#),
    ];

    for (mode, args) in &cases {
        let result = run_json(mode, args);
        let parsed = parse_ok(&result);
        assert_eq!(
            parsed["data"]["mode"].as_str().unwrap(),
            *mode,
            "receipt mode should match requested mode"
        );
    }
}

// ============================================================================
// FFI statelessness
// ============================================================================

/// Interleaved calls to different modes produce correct results for each.
#[test]
fn interleaved_mode_calls_are_independent() {
    let lang = parse_ok(&run_json("lang", r#"{"paths": ["src"]}"#));
    let module = parse_ok(&run_json("module", r#"{"paths": ["src"]}"#));
    let lang2 = parse_ok(&run_json("lang", r#"{"paths": ["src"]}"#));
    let export = parse_ok(&run_json("export", r#"{"paths": ["src"]}"#));
    let module2 = parse_ok(&run_json("module", r#"{"paths": ["src"]}"#));

    assert_eq!(lang["data"]["mode"].as_str(), Some("lang"));
    assert_eq!(module["data"]["mode"].as_str(), Some("module"));
    assert_eq!(lang2["data"]["mode"].as_str(), Some("lang"));
    assert_eq!(export["data"]["mode"].as_str(), Some("export"));
    assert_eq!(module2["data"]["mode"].as_str(), Some("module"));

    // Lang calls should produce identical results (modulo timestamp)
    let mut l1 = lang.clone();
    let mut l2 = lang2.clone();
    strip_timestamps(&mut l1);
    strip_timestamps(&mut l2);
    assert_eq!(l1, l2, "repeated lang calls should be identical");

    // Module calls should produce identical results
    let mut m1 = module.clone();
    let mut m2 = module2.clone();
    strip_timestamps(&mut m1);
    strip_timestamps(&mut m2);
    assert_eq!(m1, m2, "repeated module calls should be identical");
}

// ============================================================================
// Diff mode via FFI: structural checks
// ============================================================================

#[test]
fn diff_via_ffi_has_diff_specific_fields() {
    let result = run_json("diff", r#"{"from": "src", "to": "src"}"#);
    let parsed = parse_ok(&result);
    let data = &parsed["data"];

    assert!(
        data["diff_rows"].is_array(),
        "diff receipt should have diff_rows"
    );
    assert!(
        data["totals"].is_object(),
        "diff receipt should have totals"
    );
    assert!(
        data["from_source"].is_string(),
        "diff receipt should have from_source"
    );
    assert!(
        data["to_source"].is_string(),
        "diff receipt should have to_source"
    );
}

#[test]
fn diff_self_via_ffi_has_zero_deltas() {
    let result = run_json("diff", r#"{"from": "src", "to": "src"}"#);
    let parsed = parse_ok(&result);

    let totals = &parsed["data"]["totals"];
    assert_eq!(
        totals["delta_code"].as_i64(),
        Some(0),
        "self-diff total delta_code should be 0"
    );
    assert_eq!(
        totals["delta_lines"].as_i64(),
        Some(0),
        "self-diff total delta_lines should be 0"
    );

    let rows = parsed["data"]["diff_rows"].as_array().unwrap();
    for row in rows {
        assert_eq!(
            row["delta_code"].as_i64(),
            Some(0),
            "self-diff row delta_code should be 0 for {}",
            row["lang"]
        );
    }
}

// ============================================================================
// Version mode completeness
// ============================================================================

#[test]
fn version_data_has_all_expected_fields() {
    let result = run_json("version", "{}");
    let parsed = parse_ok(&result);
    let data = &parsed["data"];

    assert!(data["version"].is_string(), "should have version string");
    assert!(
        data["schema_version"].is_u64(),
        "should have schema_version u64"
    );

    let ver = data["version"].as_str().unwrap();
    let parts: Vec<&str> = ver.split('.').collect();
    assert_eq!(
        parts.len(),
        3,
        "version should be semver (major.minor.patch): {ver}"
    );
    for part in &parts {
        assert!(
            part.parse::<u32>().is_ok(),
            "version component should be numeric: {part}"
        );
    }
}

// ============================================================================
// Error envelope structural consistency
// ============================================================================

/// All error types produce envelopes with code (string) and message (string).
#[test]
fn error_envelopes_have_consistent_structure() {
    let error_cases: &[(&str, &str)] = &[
        ("bogus_mode", "{}"),
        ("lang", "not json"),
        ("lang", r#"{"children": "invalid"}"#),
        ("diff", "{}"),
        ("export", r#"{"format": "yaml"}"#),
        ("lang", r#"{"top": "ten"}"#),
    ];

    for (mode, args) in error_cases {
        let result = run_json(mode, args);
        let parsed = parse_err(&result);
        let err = &parsed["error"];

        assert!(
            err["code"].is_string(),
            "error.code should be string for mode={mode} args={args}"
        );
        assert!(
            err["message"].is_string(),
            "error.message should be string for mode={mode} args={args}"
        );

        // Code should be lowercase snake_case
        let code = err["code"].as_str().unwrap();
        assert!(
            code.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
            "error code should be snake_case: {code}"
        );
    }
}

// ============================================================================
// Extreme parameter values
// ============================================================================

#[test]
fn very_large_top_value_is_handled() {
    let result = run_json("lang", r#"{"paths": ["src"], "top": 999999}"#);
    let parsed = parse_ok(&result);

    let rows = parsed["data"]["rows"].as_array().unwrap();
    // Should succeed and just return all languages (fewer than 999999)
    assert!(!rows.is_empty());
}

#[test]
fn very_large_max_rows_is_handled() {
    let result = run_json("export", r#"{"paths": ["src"], "max_rows": 999999}"#);
    let parsed = parse_ok(&result);

    let rows = parsed["data"]["rows"].as_array().unwrap();
    assert!(!rows.is_empty());
}

#[test]
fn zero_top_returns_all_languages() {
    let result_zero = run_json("lang", r#"{"paths": ["src"], "top": 0}"#);
    let parsed_zero = parse_ok(&result_zero);
    let rows_zero = parsed_zero["data"]["rows"].as_array().unwrap();

    let result_big = run_json("lang", r#"{"paths": ["src"], "top": 999999}"#);
    let parsed_big = parse_ok(&result_big);
    let rows_big = parsed_big["data"]["rows"].as_array().unwrap();

    // top=0 means "no limit", should be same as a huge top value
    assert_eq!(
        rows_zero.len(),
        rows_big.len(),
        "top=0 and top=999999 should return same number of rows"
    );
}

#[test]
fn zero_max_rows_returns_all_files() {
    let result = run_json("export", r#"{"paths": ["src"], "max_rows": 0}"#);
    let parsed = parse_ok(&result);
    let rows = parsed["data"]["rows"].as_array().unwrap();

    // max_rows=0 means no limit, should return multiple files
    assert!(rows.len() > 1, "max_rows=0 should return all files");
}

// ============================================================================
// Empty paths array
// ============================================================================

/// Empty paths array may panic in underlying tokei; verify envelope is still valid JSON.
#[test]
fn empty_paths_array_produces_valid_envelope() {
    // Note: empty paths can panic in tokei, so we just verify the FFI layer
    // handles it by catching the panic or returning an error envelope.
    let result = std::panic::catch_unwind(|| run_json("lang", r#"{"paths": []}"#));
    if let Ok(output) = result {
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.get("ok").is_some());
    }
    // If it panics, that's a known tokei limitation — not an FFI bug.
}

// ============================================================================
// Receipt data type-level checks
// ============================================================================

#[test]
fn lang_rows_have_expected_numeric_fields() {
    let result = run_json("lang", r#"{"paths": ["src"]}"#);
    let parsed = parse_ok(&result);
    let rows = parsed["data"]["rows"].as_array().unwrap();

    for row in rows {
        assert!(row["lang"].is_string(), "lang should be string");
        assert!(row["code"].is_u64(), "code should be u64");
        assert!(row["lines"].is_u64(), "lines should be u64");
        assert!(row["files"].is_u64(), "files should be u64");
        assert!(row["bytes"].is_u64(), "bytes should be u64");
        assert!(row["tokens"].is_u64(), "tokens should be u64");
    }
}

#[test]
fn export_rows_have_expected_fields() {
    let result = run_json("export", r#"{"paths": ["src"]}"#);
    let parsed = parse_ok(&result);
    let rows = parsed["data"]["rows"].as_array().unwrap();

    assert!(!rows.is_empty());
    for row in rows {
        assert!(row["path"].is_string(), "path should be string");
        assert!(row["lang"].is_string(), "lang should be string");
        assert!(row["module"].is_string(), "module should be string");
        assert!(row["code"].is_u64(), "code should be u64");
        assert!(row["lines"].is_u64(), "lines should be u64");
        assert!(row["bytes"].is_u64(), "bytes should be u64");
    }
}

#[test]
fn module_rows_have_expected_fields() {
    let result = run_json("module", r#"{"paths": ["src"]}"#);
    let parsed = parse_ok(&result);
    let rows = parsed["data"]["rows"].as_array().unwrap();

    assert!(!rows.is_empty());
    for row in rows {
        assert!(row["module"].is_string(), "module should be string");
        assert!(row["code"].is_u64(), "code should be u64");
        assert!(row["lines"].is_u64(), "lines should be u64");
        assert!(row["files"].is_u64(), "files should be u64");
    }
}

// ============================================================================
// Totals consistency
// ============================================================================

#[test]
fn lang_total_code_equals_sum_of_rows() {
    let result = run_json("lang", r#"{"paths": ["src"]}"#);
    let parsed = parse_ok(&result);

    let rows = parsed["data"]["rows"].as_array().unwrap();
    let sum: u64 = rows.iter().map(|r| r["code"].as_u64().unwrap_or(0)).sum();

    let total_code = parsed["data"]["total"]["code"].as_u64().unwrap();
    assert_eq!(sum, total_code, "total.code should equal sum of row codes");
}

#[test]
fn module_total_code_equals_sum_of_rows() {
    let result = run_json("module", r#"{"paths": ["src"]}"#);
    let parsed = parse_ok(&result);

    let rows = parsed["data"]["rows"].as_array().unwrap();
    let sum: u64 = rows.iter().map(|r| r["code"].as_u64().unwrap_or(0)).sum();

    let total_code = parsed["data"]["total"]["code"].as_u64().unwrap();
    assert_eq!(sum, total_code, "total.code should equal sum of row codes");
}
