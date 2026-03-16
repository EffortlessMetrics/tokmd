//! BDD-style expansion tests for the FFI `run_json` entrypoint and workflow functions.
//!
//! These tests complement `json_api.rs` and `workflows.rs` by covering:
//! - Semver validation for `version()`
//! - `CORE_SCHEMA_VERSION` constant contract
//! - Cross-function consistency (FFI helpers vs `run_json` output)
//! - Workflow error handling for nonexistent paths
//! - Receipt structural invariants (row ordering, field presence)
//! - Envelope mutual exclusivity (`data` XOR `error`)

use serde_json::Value;
use tokmd_core::ffi::run_json;
use tokmd_core::{
    export_workflow, lang_workflow, module_workflow,
    settings::{ExportSettings, LangSettings, ModuleSettings, ScanSettings},
};

// ============================================================================
// Helpers
// ============================================================================

fn parse_envelope(result: &str) -> Value {
    let v: Value = serde_json::from_str(result).expect("run_json must always return valid JSON");
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

// ============================================================================
// 1. version() returns valid semver
// ============================================================================

#[test]
fn version_returns_valid_semver_format() {
    let v = tokmd_core::version();
    assert_semver_format(v, "version");
}

fn assert_semver_format(version: &str, label: &str) {
    assert!(version.contains('.'), "{label} should look like semver: {version}");

    let mut meta_parts = version.split('+');
    let core = meta_parts.next().unwrap();
    assert!(meta_parts.next().is_none(), "{label} should only have optional +metadata once: {version}");

    let core_version = core.split('-').next().unwrap();

    let parts: Vec<&str> = core_version.split('.').collect();
    assert_eq!(
        parts.len(),
        3,
        "{label} should be MAJOR.MINOR.PATCH[-...][+...], got: {version}"
    );
    for (i, part) in parts.iter().enumerate() {
        assert!(
            part.parse::<u32>().is_ok(),
            "{label} semver part {i} should be numeric, got: {part}"
        );
    }
}

// ============================================================================
// 2. CORE_SCHEMA_VERSION constant
// ============================================================================

#[test]
fn core_schema_version_equals_types_schema_version() {
    assert_eq!(tokmd_core::CORE_SCHEMA_VERSION, tokmd_types::SCHEMA_VERSION);
}

#[test]
fn core_schema_version_is_positive() {
    const _: () = assert!(tokmd_core::CORE_SCHEMA_VERSION > 0);
}

// ============================================================================
// 3. FFI helpers consistent with run_json output
// ============================================================================

#[test]
fn ffi_version_matches_run_json_version_data() {
    let ffi_ver = tokmd_core::ffi::version();
    let ffi_sv = tokmd_core::ffi::schema_version();

    let result = run_json("version", "{}");
    let v = assert_ok(&result);

    let run_ver = v["data"]["version"].as_str().expect("version string");
    let run_sv = v["data"]["schema_version"]
        .as_u64()
        .expect("schema_version");

    assert_eq!(
        ffi_ver, run_ver,
        "ffi::version() must match run_json version"
    );
    assert_eq!(
        u64::from(ffi_sv),
        run_sv,
        "ffi::schema_version() must match run_json schema_version"
    );
}

#[test]
fn ffi_version_matches_lib_version() {
    assert_eq!(
        tokmd_core::version(),
        tokmd_core::ffi::version(),
        "lib version() and ffi::version() must agree"
    );
}

// ============================================================================
// 4. Envelope mutual exclusivity: success has data, no error; error vice versa
// ============================================================================

#[test]
fn envelope_success_has_data_xor_error() {
    let modes_args: &[(&str, &str)] = &[
        ("version", "{}"),
        ("lang", r#"{"paths":["src"]}"#),
        ("module", r#"{"paths":["src"]}"#),
        ("export", r#"{"paths":["src"]}"#),
    ];

    for &(mode, args) in modes_args {
        let result = run_json(mode, args);
        let v = assert_ok(&result);
        assert!(
            v.get("data").is_some(),
            "success must have 'data' for mode={mode}"
        );
        assert!(
            v.get("error").is_none(),
            "success must NOT have 'error' for mode={mode}"
        );
    }
}

#[test]
fn envelope_error_has_error_xor_data() {
    let modes_args: &[(&str, &str)] = &[
        ("nonexistent", "{}"),
        ("lang", "bad json"),
        ("", "{}"),
        ("lang", r#"{"children":"bad"}"#),
    ];

    for &(mode, args) in modes_args {
        let result = run_json(mode, args);
        let v = assert_err(&result);
        assert!(
            v.get("error").is_some(),
            "error must have 'error' for mode={mode}"
        );
        assert!(
            v.get("data").is_none(),
            "error must NOT have 'data' for mode={mode} args={args}"
        );
    }
}

// ============================================================================
// 5. run_json lang receipt contains deterministic row ordering
// ============================================================================

#[test]
fn run_json_lang_rows_sorted_by_code_descending() {
    let result = run_json("lang", r#"{"paths":["src"]}"#);
    let v = assert_ok(&result);

    let rows = v["data"]["rows"].as_array().expect("rows array");
    let codes: Vec<u64> = rows
        .iter()
        .map(|r| r["code"].as_u64().unwrap_or(0))
        .collect();

    for window in codes.windows(2) {
        assert!(
            window[0] >= window[1],
            "rows should be sorted by code descending: {:?}",
            codes
        );
    }
}

// ============================================================================
// 6. run_json lang receipt has all required envelope fields
// ============================================================================

#[test]
fn run_json_lang_receipt_has_all_required_fields() {
    let result = run_json("lang", r#"{"paths":["src"]}"#);
    let v = assert_ok(&result);
    let data = &v["data"];

    let required = [
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "status",
        "scan",
        "args",
        "rows",
    ];
    for field in &required {
        assert!(
            data.get(*field).is_some(),
            "lang receipt missing required field: {field}"
        );
    }
}

// ============================================================================
// 7. Workflow error handling for nonexistent paths
// ============================================================================

#[test]
fn lang_workflow_nonexistent_path_returns_error() {
    let scan = ScanSettings::for_paths(vec!["/absolutely/nonexistent/path/xyzzy123".to_string()]);
    let lang = LangSettings::default();
    let result = lang_workflow(&scan, &lang);
    // Depending on implementation, this either errors or returns empty rows
    match result {
        Ok(receipt) => {
            assert!(
                receipt.report.rows.is_empty(),
                "nonexistent path should yield no rows"
            );
        }
        Err(_) => { /* error is an acceptable response */ }
    }
}

#[test]
fn module_workflow_nonexistent_path_returns_error() {
    let scan = ScanSettings::for_paths(vec!["/absolutely/nonexistent/path/xyzzy123".to_string()]);
    let module = ModuleSettings::default();
    let result = module_workflow(&scan, &module);
    match result {
        Ok(receipt) => {
            assert!(
                receipt.report.rows.is_empty(),
                "nonexistent path should yield no rows"
            );
        }
        Err(_) => { /* error is an acceptable response */ }
    }
}

#[test]
fn export_workflow_nonexistent_path_returns_error() {
    let scan = ScanSettings::for_paths(vec!["/absolutely/nonexistent/path/xyzzy123".to_string()]);
    let export = ExportSettings::default();
    let result = export_workflow(&scan, &export);
    match result {
        Ok(receipt) => {
            assert!(
                receipt.data.rows.is_empty(),
                "nonexistent path should yield no rows"
            );
        }
        Err(_) => { /* error is an acceptable response */ }
    }
}

// ============================================================================
// 8. run_json with nonexistent path returns valid envelope
// ============================================================================

#[test]
fn run_json_nonexistent_path_produces_valid_envelope() {
    let result = run_json(
        "lang",
        r#"{"paths":["/absolutely/nonexistent/path/xyzzy123"]}"#,
    );
    // Must always produce valid JSON with ok field, regardless of path validity
    let v = parse_envelope(&result);
    assert!(v["ok"].is_boolean());
}

// ============================================================================
// 9. Workflow receipts are deterministic across repeated calls
// ============================================================================

#[test]
fn lang_workflow_deterministic_row_order() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();

    let r1 = lang_workflow(&scan, &lang).expect("first call");
    let r2 = lang_workflow(&scan, &lang).expect("second call");

    assert_eq!(r1.report.rows.len(), r2.report.rows.len());
    for (a, b) in r1.report.rows.iter().zip(r2.report.rows.iter()) {
        assert_eq!(a.lang, b.lang, "row languages must match across calls");
        assert_eq!(a.code, b.code, "row code counts must match across calls");
    }
}

#[test]
fn module_workflow_deterministic_row_order() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings::default();

    let r1 = module_workflow(&scan, &module).expect("first call");
    let r2 = module_workflow(&scan, &module).expect("second call");

    assert_eq!(r1.report.rows.len(), r2.report.rows.len());
    for (a, b) in r1.report.rows.iter().zip(r2.report.rows.iter()) {
        assert_eq!(a.module, b.module, "module names must match across calls");
    }
}

// ============================================================================
// 10. Export receipt rows have all required per-row fields
// ============================================================================

#[test]
fn export_receipt_rows_have_required_fields() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).expect("export_workflow");
    assert!(!receipt.data.rows.is_empty(), "should have rows");

    for row in &receipt.data.rows {
        assert!(!row.path.is_empty(), "row.path must not be empty");
        assert!(!row.lang.is_empty(), "row.lang must not be empty");
        // Forward-slash normalization
        assert!(
            !row.path.contains('\\'),
            "row.path must use forward slashes: {}",
            row.path
        );
    }
}

// ============================================================================
// 11. run_json version mode is idempotent (byte-identical)
// ============================================================================

#[test]
fn run_json_version_idempotent() {
    let r1 = run_json("version", "{}");
    let r2 = run_json("version", "{}");
    assert_eq!(
        r1, r2,
        "version output should be byte-identical across calls"
    );
}

// ============================================================================
// 12. Error code values are valid snake_case strings
// ============================================================================

#[test]
fn error_codes_are_snake_case() {
    let cases: &[(&str, &str)] = &[
        ("nonexistent", "{}"),
        ("lang", "not json"),
        ("lang", r#"{"children":"bad"}"#),
        ("export", r#"{"format":"yaml"}"#),
    ];

    fn is_snake_case(s: &str) -> bool {
        !s.is_empty()
            && s.starts_with(|c: char| c.is_ascii_lowercase())
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    }

    for &(mode, args) in cases {
        let result = run_json(mode, args);
        let v = assert_err(&result);
        let code = v["error"]["code"]
            .as_str()
            .expect("error.code should be string");
        assert!(
            is_snake_case(code),
            "error code should be snake_case, got: {code}"
        );
    }
}
