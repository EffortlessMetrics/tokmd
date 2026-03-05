//! FFI parity tests for `run_json` entrypoint (w53).
//!
//! Verifies that every documented mode returns the correct envelope shape,
//! required fields are present, and error handling is robust.

use serde_json::Value;
use std::fs;
use std::path::Path;
use tokmd_core::ffi::run_json;

// ============================================================================
// Helpers
// ============================================================================

fn parse_envelope(json: &str) -> Value {
    serde_json::from_str(json).expect("run_json must return valid JSON")
}

fn assert_ok(json: &str) -> Value {
    let v = parse_envelope(json);
    assert_eq!(v["ok"], true, "expected ok:true — got: {json}");
    v
}

fn assert_err(json: &str) -> Value {
    let v = parse_envelope(json);
    assert_eq!(v["ok"], false, "expected ok:false — got: {json}");
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
// lang mode
// ============================================================================

#[test]
fn lang_mode_has_ok_true() {
    let r = run_json("lang", r#"{"paths":["src"]}"#);
    assert_ok(&r);
}

#[test]
fn lang_mode_data_has_rows() {
    let r = run_json("lang", r#"{"paths":["src"]}"#);
    let v = assert_ok(&r);
    assert!(v["data"]["rows"].is_array(), "data must have rows array");
    assert!(
        !v["data"]["rows"].as_array().unwrap().is_empty(),
        "rows should not be empty for src/"
    );
}

#[test]
fn lang_mode_data_has_total() {
    let r = run_json("lang", r#"{"paths":["src"]}"#);
    let v = assert_ok(&r);
    assert!(
        v["data"]["total"].is_object(),
        "data must have total object"
    );
}

#[test]
fn lang_mode_data_has_tool_info() {
    let r = run_json("lang", r#"{"paths":["src"]}"#);
    let v = assert_ok(&r);
    assert!(v["data"]["tool"].is_object(), "data must have tool object");
    assert!(
        v["data"]["tool"]["name"].is_string(),
        "tool.name must be a string"
    );
    assert!(
        v["data"]["tool"]["version"].is_string(),
        "tool.version must be a string"
    );
}

#[test]
fn lang_mode_data_has_schema_version() {
    let r = run_json("lang", r#"{"paths":["src"]}"#);
    let v = assert_ok(&r);
    assert!(
        v["data"]["schema_version"].is_number(),
        "must have schema_version"
    );
}

#[test]
fn lang_mode_respects_top_parameter() {
    let r = run_json("lang", r#"{"paths":["src"],"top":1}"#);
    let v = assert_ok(&r);
    let rows = v["data"]["rows"].as_array().unwrap();
    // top=1 means at most 2 rows (1 real + possible "Other")
    assert!(rows.len() <= 2, "top=1 should limit rows, got {}", rows.len());
}

#[test]
fn lang_mode_with_files_flag() {
    let r = run_json("lang", r#"{"paths":["src"],"files":true}"#);
    let v = assert_ok(&r);
    assert_eq!(v["data"]["args"]["with_files"], true);
}

#[test]
fn lang_mode_with_tempdir() {
    let repo = make_repo("fn main() {}\n");
    let p = repo.path().to_string_lossy().replace('\\', "/");
    let args = format!(r#"{{"paths":["{p}"]}}"#);
    let r = run_json("lang", &args);
    let v = assert_ok(&r);
    assert!(v["data"]["rows"].is_array());
}

// ============================================================================
// module mode
// ============================================================================

#[test]
fn module_mode_has_ok_true() {
    let r = run_json("module", r#"{"paths":["src"]}"#);
    assert_ok(&r);
}

#[test]
fn module_mode_data_has_rows() {
    let r = run_json("module", r#"{"paths":["src"]}"#);
    let v = assert_ok(&r);
    assert!(v["data"]["rows"].is_array(), "module data must have rows");
}

#[test]
fn module_mode_has_correct_mode_field() {
    let r = run_json("module", r#"{"paths":["src"]}"#);
    let v = assert_ok(&r);
    assert_eq!(v["data"]["mode"], "module");
}

#[test]
fn module_mode_with_depth() {
    let r = run_json("module", r#"{"paths":["src"],"module_depth":1}"#);
    let v = assert_ok(&r);
    assert_eq!(v["data"]["args"]["module_depth"], 1);
}

// ============================================================================
// export mode
// ============================================================================

#[test]
fn export_mode_has_ok_true() {
    let r = run_json("export", r#"{"paths":["src"]}"#);
    assert_ok(&r);
}

#[test]
fn export_mode_data_has_file_level_rows() {
    let r = run_json("export", r#"{"paths":["src"]}"#);
    let v = assert_ok(&r);
    let rows = v["data"]["rows"].as_array().expect("rows must be array");
    assert!(!rows.is_empty(), "export should have file-level rows");
    // Each row should have a path field
    for row in rows {
        assert!(row["path"].is_string(), "each export row needs a path");
    }
}

#[test]
fn export_mode_has_correct_mode_field() {
    let r = run_json("export", r#"{"paths":["src"]}"#);
    let v = assert_ok(&r);
    assert_eq!(v["data"]["mode"], "export");
}

#[test]
fn export_mode_rows_have_lang_field() {
    let r = run_json("export", r#"{"paths":["src"]}"#);
    let v = assert_ok(&r);
    let rows = v["data"]["rows"].as_array().unwrap();
    for row in rows {
        assert!(row["lang"].is_string(), "each export row needs a lang");
    }
}

// ============================================================================
// version mode
// ============================================================================

#[test]
fn version_mode_has_ok_true() {
    let r = run_json("version", "{}");
    assert_ok(&r);
}

#[test]
fn version_mode_returns_semver_string() {
    let r = run_json("version", "{}");
    let v = assert_ok(&r);
    let ver = v["data"]["version"].as_str().expect("version must be string");
    let parts: Vec<&str> = ver.split('.').collect();
    assert!(parts.len() >= 2, "version should be semver: {ver}");
}

#[test]
fn version_mode_matches_cargo_pkg_version() {
    let r = run_json("version", "{}");
    let v = assert_ok(&r);
    let ver = v["data"]["version"].as_str().unwrap();
    assert_eq!(ver, tokmd_core::version());
}

#[test]
fn version_mode_schema_version_matches_constant() {
    let r = run_json("version", "{}");
    let v = assert_ok(&r);
    let sv = v["data"]["schema_version"].as_u64().unwrap();
    assert_eq!(sv as u32, tokmd_types::SCHEMA_VERSION);
}

// ============================================================================
// diff mode
// ============================================================================

#[test]
fn diff_mode_nonexistent_from_returns_error() {
    let r = run_json(
        "diff",
        r#"{"from":"/nonexistent/path/abc123","to":"src"}"#,
    );
    assert_err(&r);
}

#[test]
fn diff_mode_nonexistent_to_returns_error() {
    let r = run_json(
        "diff",
        r#"{"from":"src","to":"/nonexistent/path/xyz789"}"#,
    );
    assert_err(&r);
}

#[test]
fn diff_mode_self_diff_succeeds() {
    let r = run_json("diff", r#"{"from":"src","to":"src"}"#);
    let v = assert_ok(&r);
    assert_eq!(v["data"]["mode"], "diff");
}

// ============================================================================
// Invalid mode
// ============================================================================

#[test]
fn invalid_mode_returns_ok_false() {
    let r = run_json("nonexistent", "{}");
    let v = assert_err(&r);
    assert_eq!(v["ok"], false);
}

#[test]
fn invalid_mode_has_unknown_mode_code() {
    let r = run_json("nonexistent", "{}");
    let v = assert_err(&r);
    assert_eq!(v["error"]["code"], "unknown_mode");
}

#[test]
fn invalid_mode_has_error_message() {
    let r = run_json("nonexistent", "{}");
    let v = assert_err(&r);
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(!msg.is_empty());
    assert!(msg.contains("nonexistent"), "message should mention the mode");
}

// ============================================================================
// Empty and invalid args
// ============================================================================

#[test]
fn empty_object_args_defaults_to_cwd() {
    // With empty args, lang should default to scanning "."
    let r = run_json("lang", "{}");
    // Should succeed (defaults to current dir)
    assert_ok(&r);
}

#[test]
fn invalid_json_args_returns_error() {
    let r = run_json("lang", "this is not json");
    let v = assert_err(&r);
    assert_eq!(v["error"]["code"], "invalid_json");
}

#[test]
fn truncated_json_returns_error() {
    let r = run_json("lang", r#"{"paths":["src"]"#);
    assert_err(&r);
}

#[test]
fn wrong_type_for_top_returns_error() {
    let r = run_json("lang", r#"{"paths":["src"],"top":"not_a_number"}"#);
    let v = assert_err(&r);
    assert_eq!(v["error"]["code"], "invalid_settings");
}

#[test]
fn wrong_type_for_files_returns_error() {
    let r = run_json("lang", r#"{"paths":["src"],"files":"yes"}"#);
    let v = assert_err(&r);
    assert_eq!(v["error"]["code"], "invalid_settings");
}
