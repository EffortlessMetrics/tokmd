//! Bindings parity tests — verify the core API contract that Python and
//! Node.js bindings depend on.
//!
//! These tests exercise `tokmd_core` from the outside (integration-test
//! style) and confirm:
//!   1. All workflow functions are accessible.
//!   2. `run_json` handles every documented mode.
//!   3. The JSON response envelope always contains `ok`, `data`, or `error`.
//!   4. The `version` mode returns valid version info.
//!   5. Arbitrary JSON never causes a panic inside `run_json` (property test).

use std::fs;
use std::path::Path;

use proptest::prelude::*;
use serde_json::Value;
use tokmd_core::ffi::{run_json, schema_version, version};
use tokmd_core::settings::{ExportSettings, LangSettings, ModuleSettings, ScanSettings};
use tokmd_core::{export_workflow, lang_workflow, module_workflow};

// ============================================================================
// Helpers
// ============================================================================

fn parse_envelope(json: &str) -> Value {
    let v: Value = serde_json::from_str(json).expect("run_json must return valid JSON");
    assert!(v.get("ok").is_some(), "envelope must contain 'ok': {json}");
    v
}

fn assert_ok(json: &str) -> Value {
    let v = parse_envelope(json);
    assert_eq!(v["ok"], true, "expected ok:true – {json}");
    v
}

fn assert_err(json: &str) -> Value {
    let v = parse_envelope(json);
    assert_eq!(v["ok"], false, "expected ok:false – {json}");
    assert!(v.get("error").is_some(), "error envelope needs 'error'");
    v
}

fn write_file(root: &Path, rel: &str, contents: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, contents).unwrap();
}

fn make_repo(code: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(dir.path(), "src/lib.rs", code);
    dir
}

// ============================================================================
// 1. Workflow function accessibility
// ============================================================================

#[test]
fn lang_workflow_is_accessible() {
    let scan = ScanSettings::for_paths(vec!["src".into()]);
    let settings = LangSettings::default();
    let receipt = lang_workflow(&scan, &settings).expect("lang_workflow");
    assert_eq!(receipt.mode, "lang");
    assert!(!receipt.report.rows.is_empty());
}

#[test]
fn module_workflow_is_accessible() {
    let scan = ScanSettings::for_paths(vec!["src".into()]);
    let settings = ModuleSettings::default();
    let receipt = module_workflow(&scan, &settings).expect("module_workflow");
    assert_eq!(receipt.mode, "module");
}

#[test]
fn export_workflow_is_accessible() {
    let scan = ScanSettings::for_paths(vec!["src".into()]);
    let settings = ExportSettings::default();
    let receipt = export_workflow(&scan, &settings).expect("export_workflow");
    assert_eq!(receipt.mode, "export");
}

// ============================================================================
// 2. run_json handles all modes
// ============================================================================

#[test]
fn run_json_lang_mode_succeeds() {
    let r = run_json("lang", r#"{"paths":["src"]}"#);
    let v = assert_ok(&r);
    assert_eq!(v["data"]["mode"].as_str(), Some("lang"));
}

#[test]
fn run_json_module_mode_succeeds() {
    let r = run_json("module", r#"{"paths":["src"]}"#);
    let v = assert_ok(&r);
    assert_eq!(v["data"]["mode"].as_str(), Some("module"));
}

#[test]
fn run_json_export_mode_succeeds() {
    let r = run_json("export", r#"{"paths":["src"]}"#);
    let v = assert_ok(&r);
    assert_eq!(v["data"]["mode"].as_str(), Some("export"));
}

#[cfg(feature = "analysis")]
#[test]
fn run_json_analyze_mode_succeeds() {
    let repo = make_repo("fn main() {}\n");
    let p = repo.path().to_string_lossy();
    let args = format!(r#"{{"paths":["{p}"],"preset":"receipt"}}"#).replace('\\', "/");
    let r = run_json("analyze", &args);
    let v = assert_ok(&r);
    assert_eq!(v["data"]["mode"].as_str(), Some("analysis"));
}

#[cfg(not(feature = "analysis"))]
#[test]
fn run_json_analyze_mode_returns_not_implemented() {
    let r = run_json("analyze", "{}");
    let v = assert_err(&r);
    assert_eq!(v["error"]["code"].as_str(), Some("not_implemented"));
}

#[test]
fn run_json_diff_mode_succeeds() {
    let a = make_repo("fn a() {}\n");
    let b = make_repo("fn b() {}\n");
    let pa = a.path().to_string_lossy().replace('\\', "/");
    let pb = b.path().to_string_lossy().replace('\\', "/");
    let args = format!(r#"{{"from":"{pa}","to":"{pb}"}}"#);
    let r = run_json("diff", &args);
    let v = assert_ok(&r);
    assert_eq!(v["data"]["mode"].as_str(), Some("diff"));
}

#[test]
fn run_json_version_mode_succeeds() {
    let r = run_json("version", "{}");
    let v = assert_ok(&r);
    assert!(v["data"]["version"].as_str().is_some());
    assert!(v["data"]["schema_version"].as_u64().unwrap_or(0) > 0);
}

#[test]
fn run_json_unknown_mode_returns_error() {
    let r = run_json("nonexistent_mode", "{}");
    let v = assert_err(&r);
    assert_eq!(
        v["error"]["code"].as_str(),
        Some("unknown_mode"),
        "unknown mode should produce unknown_mode error code"
    );
}

// ============================================================================
// 3. Envelope format
// ============================================================================

#[test]
fn success_envelope_has_ok_and_data() {
    let r = run_json("version", "{}");
    let v: Value = serde_json::from_str(&r).unwrap();
    assert_eq!(v["ok"], true);
    assert!(v["data"].is_object(), "data must be an object");
    assert!(v.get("error").is_none(), "success must not have error");
}

#[test]
fn error_envelope_has_ok_and_error() {
    let r = run_json("lang", "{invalid json");
    let v: Value = serde_json::from_str(&r).unwrap();
    assert_eq!(v["ok"], false);
    assert!(v["error"].is_object(), "error must be an object");
    assert!(
        v["error"]["code"].is_string(),
        "error.code must be a string"
    );
    assert!(
        v["error"]["message"].is_string(),
        "error.message must be a string"
    );
}

#[test]
fn error_envelope_invalid_json_code() {
    let r = run_json("lang", "not json at all");
    let v = assert_err(&r);
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_json"));
}

// ============================================================================
// 4. Version mode validation
// ============================================================================

#[test]
fn version_function_returns_semver() {
    let v = version();
    assert!(!v.is_empty());
    let parts: Vec<&str> = v.split('.').collect();
    assert!(parts.len() >= 2, "version should look like semver: {v}");
}

#[test]
fn schema_version_is_positive() {
    assert!(schema_version() > 0);
}

#[test]
fn version_mode_matches_function() {
    let r = run_json("version", "{}");
    let v = assert_ok(&r);
    assert_eq!(v["data"]["version"].as_str().unwrap(), version());
    assert_eq!(
        v["data"]["schema_version"].as_u64().unwrap(),
        u64::from(schema_version()),
    );
}

// ============================================================================
// 5. Property test — run_json never panics on arbitrary input
// ============================================================================

proptest! {
    #[test]
    fn run_json_never_panics(mode in "\\PC{0,20}", args in "\\PC{0,200}") {
        let result = run_json(&mode, &args);
        // Must always produce valid JSON
        let _: Value = serde_json::from_str(&result)
            .expect("run_json must always return valid JSON");
    }

    #[test]
    fn run_json_valid_json_never_panics(
        mode in prop::sample::select(vec![
            "lang", "module", "export", "analyze", "cockpit",
            "diff", "version", "unknown",
        ]),
        args in prop::collection::hash_map(
            "[a-z_]{1,10}",
            prop_oneof![
                Just(Value::Null),
                any::<bool>().prop_map(Value::Bool),
                any::<i64>().prop_map(|n| Value::Number(n.into())),
                "[a-z /\\.]{0,30}".prop_map(|s| Value::String(s)),
            ],
            0..5,
        ),
    ) {
        let json_str = serde_json::to_string(&args).unwrap();
        let result = run_json(mode, &json_str);
        let _: Value = serde_json::from_str(&result)
            .expect("run_json must always return valid JSON");
    }
}
