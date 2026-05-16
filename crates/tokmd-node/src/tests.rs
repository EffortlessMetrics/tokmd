use super::*;
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::future::Future;
use std::path::Path;

fn block_on<T>(future: impl Future<Output = Result<T>>) -> Result<T> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .build()
        .expect("build tokio runtime");
    runtime.block_on(future)
}

fn write_file(root: &Path, rel: &str, contents: &str) {
    let path = root.join(rel);
    let parent = path.parent().unwrap_or(root);
    fs::create_dir_all(parent).expect("create parent dirs");
    fs::write(path, contents).expect("write file");
}

fn make_repo(contents: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("create temp dir");
    write_file(dir.path(), "src/lib.rs", contents);
    dir
}

#[test]
fn version_and_schema_version_are_nonzero() {
    let v = version();
    assert!(!v.is_empty());
    let schema = schema_version();
    assert!(schema > 0);
}

#[test]
fn run_json_version_returns_envelope() {
    let output = block_on(run_json("version".to_string(), "{}".to_string()))
        .expect("run_json should succeed");
    let env: serde_json::Value = serde_json::from_str(&output).expect("parse json");
    assert!(env["ok"].as_bool().unwrap_or(false));
    assert!(!env["data"]["version"].as_str().unwrap_or("").is_empty());
    assert!(env["data"]["schema_version"].as_u64().unwrap_or(0) > 0);
}

#[test]
fn run_json_invalid_json_returns_error_envelope() {
    let output = block_on(run_json("lang".to_string(), "{".to_string()))
        .expect("run_json should return envelope");
    let env: serde_json::Value = serde_json::from_str(&output).expect("parse json");
    assert!(!env["ok"].as_bool().unwrap_or(true));
    assert_eq!(env["error"]["code"].as_str().unwrap_or(""), "invalid_json");
}

#[test]
fn run_invalid_mode_returns_error() {
    let err = block_on(run("nope".to_string(), json!({}))).unwrap_err();
    let message = err.to_string();
    assert!(message.contains("unknown_mode"));
}

#[test]
fn wrappers_scan_small_repo() {
    let repo = make_repo("fn main() { println!(\"hi\"); }\n");
    let path = repo.path().to_string_lossy().to_string();

    let lang_result = block_on(lang(Some(json!({
        "paths": [path.clone()],
        "files": true
    }))))
    .expect("lang should succeed");
    assert_eq!(lang_result["mode"].as_str().unwrap_or(""), "lang");
    assert!(
        lang_result["rows"]
            .as_array()
            .map(|r| !r.is_empty())
            .unwrap_or(false)
    );

    let module_result = block_on(module_fn(Some(json!({
        "paths": [path.clone()],
        "module_roots": ["src"],
        "module_depth": 1
    }))))
    .expect("module should succeed");
    assert_eq!(module_result["mode"].as_str().unwrap_or(""), "module");
    assert!(
        module_result["rows"]
            .as_array()
            .map(|r| !r.is_empty())
            .unwrap_or(false)
    );

    let export_result = block_on(export_fn(Some(json!({
        "paths": [path.clone()],
        "format": "json"
    }))))
    .expect("export should succeed");
    assert_eq!(export_result["mode"].as_str().unwrap_or(""), "export");
    assert!(
        export_result["rows"]
            .as_array()
            .map(|r| !r.is_empty())
            .unwrap_or(false)
    );
}

#[test]
fn export_accepts_meta_and_strip_prefix_options() {
    let repo = make_repo("fn main() { println!(\"hi\"); }\n");
    let path = repo.path().to_string_lossy().to_string();

    let export_result = block_on(export_fn(Some(json!({
        "paths": [path.clone()],
        "format": "json",
        "meta": false,
        "strip_prefix": path.clone(),
    }))))
    .expect("export should succeed");

    assert_eq!(
        export_result["args"]["strip_prefix"].as_str().unwrap_or(""),
        path
    );
}

#[test]
fn analyze_returns_receipt() {
    let repo = make_repo("fn main() {}\n");
    let path = repo.path().to_string_lossy().to_string();
    let result = block_on(analyze(Some(
        json!({ "paths": [path], "preset": "receipt" }),
    )))
    .expect("analyze should succeed");
    assert_eq!(result["mode"].as_str().unwrap_or(""), "analysis");
}

#[test]
fn diff_compares_two_paths() {
    let repo_a = make_repo("fn main() { println!(\"a\"); }\n");
    let repo_b = make_repo("fn main() { println!(\"b\"); }\n");
    let path_a = repo_a.path().to_string_lossy().to_string();
    let path_b = repo_b.path().to_string_lossy().to_string();

    let diff_result = block_on(diff(Some(json!({
        "from": path_a,
        "to": path_b
    }))))
    .expect("diff should succeed");
    assert_eq!(diff_result["mode"].as_str().unwrap_or(""), "diff");
    assert!(diff_result.get("totals").is_some());
}

#[test]
fn extract_envelope_returns_envelope_when_data_missing() {
    let envelope = json!({
        "ok": true,
        "mode": "version"
    });
    let result = extract_envelope(envelope.clone()).expect("should return envelope");
    assert_eq!(result, envelope);
}

#[test]
fn extract_envelope_returns_unknown_error_when_error_missing() {
    let err = extract_envelope(json!({ "ok": false })).unwrap_err();
    let message = err.to_string();
    assert!(message.contains("Unknown error"));
}

#[derive(Debug)]
struct BadSerialize;

impl Serialize for BadSerialize {
    fn serialize<S>(&self, _serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom("boom"))
    }
}

#[test]
fn encode_args_maps_serde_error() {
    let err = encode_args(&BadSerialize).unwrap_err();
    let message = err.to_string();
    assert!(message.contains("JSON error"));
}

#[test]
fn parse_envelope_maps_json_error() {
    let err = parse_envelope("{").unwrap_err();
    let message = err.to_string();
    assert!(message.contains("JSON parse error"));
}

#[test]
fn run_blocking_maps_join_error() {
    let err = block_on(run_blocking(|| panic!("boom"))).unwrap_err();
    let message = err.to_string();
    assert!(message.contains("Task join error"));
}

#[test]
fn options_or_empty_returns_default() {
    assert_eq!(options_or_empty(None), json!({}));
    let value = json!({ "paths": ["src"] });
    assert_eq!(options_or_empty(Some(value.clone())), value);
}

#[test]
fn run_with_args_json_propagates_encode_error() {
    let err = block_on(run_with_args_json(
        "lang".to_string(),
        Err(Error::from_reason("encode fail")),
    ))
    .unwrap_err();
    assert!(err.to_string().contains("encode fail"));
}

#[test]
fn parse_and_extract_propagates_result_error() {
    let err = parse_and_extract(Err(Error::from_reason("join fail"))).unwrap_err();
    assert!(err.to_string().contains("join fail"));
}

#[test]
fn parse_and_extract_maps_json_error() {
    let err = parse_and_extract(Ok("{".to_string())).unwrap_err();
    assert!(err.to_string().contains("JSON parse error"));
}

// ========================================================================
// Compile-check stubs: verify the core API surface that bindings depend on
// ========================================================================

/// Integration tests for cdylib crates cannot live in `tests/` because
/// Cargo does not produce an rlib for linking.  These inline stubs verify
/// that the underlying `tokmd_core` contract is stable.

#[test]
fn core_version_matches_binding_version() {
    let core_ver = tokmd_core::ffi::version();
    let binding_ver = version();
    assert_eq!(
        core_ver,
        binding_ver.as_str(),
        "binding must delegate to core"
    );
}

#[test]
fn core_schema_version_matches_binding() {
    let core_sv = tokmd_core::ffi::schema_version();
    let binding_sv = schema_version();
    assert_eq!(core_sv, binding_sv, "binding must delegate to core");
}

#[test]
fn core_run_json_returns_valid_json_for_all_modes() {
    let modes = ["lang", "module", "export", "analyze", "diff", "version"];
    for mode in modes {
        let result = tokmd_core::ffi::run_json(mode, "{}");
        let v: serde_json::Value =
            serde_json::from_str(&result).expect("run_json must return valid JSON");
        assert!(
            v.get("ok").is_some(),
            "envelope for mode '{mode}' missing 'ok'"
        );
    }
}

#[test]
fn core_run_json_unknown_mode_returns_error() {
    let result = tokmd_core::ffi::run_json("bogus", "{}");
    let v: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"].as_str(), Some("unknown_mode"));
}

#[test]
fn map_envelope_error_preserves_message() {
    let err = tokmd_envelope::ffi::EnvelopeExtractError::JsonParse("test error".to_string());
    let napi_err = map_envelope_error(err);
    assert!(napi_err.to_string().contains("test error"));
}
