use super::*;
use serde_json::json;

fn fixture_inputs() -> Value {
    json!([
        {
            "path": "crates/app/src/lib.rs",
            "text": "pub fn alpha() -> usize { 1 }\n"
        },
        {
            "path": "src/main.rs",
            "text": "fn main() {}\n"
        },
        {
            "path": "tests/basic.py",
            "text": "# TODO: keep smoke\nprint('ok')\n"
        }
    ])
}

#[test]
fn run_json_returns_valid_envelope() {
    let result = run_json("version", "{}");
    let envelope = tokmd_envelope::ffi::parse_envelope(&result).expect("valid JSON envelope");

    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["data"]["version"], env!("CARGO_PKG_VERSION"));
}

#[test]
fn run_data_json_returns_payload_without_envelope() {
    let payload = run_data_json("version", "{}").expect("version payload");
    let value: Value = serde_json::from_str(&payload).expect("valid payload json");

    assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
    assert!(value.get("schema_version").is_some());
}

#[test]
fn capabilities_reports_rootless_surface() {
    let obj: Value = serde_json::from_str(&capabilities_json()).expect("capabilities JSON");

    assert_eq!(obj["modes"][0], "lang");
    assert_eq!(obj["modes"][1], "module");
    assert_eq!(obj["modes"][2], "export");

    #[cfg(feature = "analysis")]
    {
        assert_eq!(obj["modes"][3], "analyze");
        assert_eq!(
            obj["analyze"]["rootlessPresets"],
            json!(["receipt", "estimate"])
        );
    }

    #[cfg(not(feature = "analysis"))]
    {
        assert_eq!(obj["modes"].as_array().expect("modes").len(), 3);
        assert_eq!(obj["analyze"]["rootlessPresets"], json!([]));
    }
}

#[test]
fn normalize_raw_json_args_accepts_json_object_strings() {
    let raw = r#"{"inputs":[{"path":"src/lib.rs","text":"pub fn alpha() {}\n"}]}"#;

    assert_eq!(
        normalize_raw_json_args(raw).expect("valid raw args"),
        raw.to_string()
    );
}

#[test]
fn normalize_raw_json_args_rejects_invalid_json_strings() {
    let err = normalize_raw_json_args("{not json").expect_err("invalid raw args");

    assert!(err.contains("failed to parse JSON string arguments"));
}

#[test]
fn run_mode_value_lang_supports_in_memory_inputs() {
    let data = run_mode_value(
        "lang",
        &json!({
            "inputs": fixture_inputs(),
            "files": true
        }),
    )
    .expect("lang data");

    assert_eq!(data["mode"], "lang");
    assert_eq!(data["scan"]["paths"][0], "crates/app/src/lib.rs");
    assert_eq!(data["total"]["files"], 3);
}

#[test]
fn run_mode_value_export_preserves_logical_paths() {
    let data = run_mode_value(
        "export",
        &json!({
            "inputs": fixture_inputs()
        }),
    )
    .expect("export data");
    let paths: Vec<&str> = data["rows"]
        .as_array()
        .expect("rows array")
        .iter()
        .map(|row| row["path"].as_str().expect("row path"))
        .collect();

    assert_eq!(data["mode"], "export");
    assert!(paths.contains(&"crates/app/src/lib.rs"));
    assert!(paths.contains(&"tests/basic.py"));
}

#[cfg(feature = "analysis")]
#[test]
fn run_mode_value_analyze_estimate_returns_effort_payload() {
    let data = run_mode_value(
        "analyze",
        &json!({
            "inputs": fixture_inputs(),
            "preset": "estimate"
        }),
    )
    .expect("analysis data");

    assert_eq!(data["mode"], "analysis");
    assert_eq!(data["source"]["inputs"][1], "src/main.rs");
    assert_eq!(data["effort"]["model"], "cocomo81-basic");
    assert_eq!(data["effort"]["size_basis"]["total_lines"], 3);
    assert!(
        data["effort"]["results"]["effort_pm_p50"]
            .as_f64()
            .expect("effort p50")
            > 0.0
    );
}

#[cfg(feature = "analysis")]
#[test]
fn run_mode_value_analyze_receipt_returns_rootless_receipt_payload() {
    let data = run_mode_value(
        "analyze",
        &json!({
            "inputs": fixture_inputs(),
            "preset": "receipt"
        }),
    )
    .expect("analysis data");

    assert_eq!(data["mode"], "analysis");
    assert_eq!(data["source"]["inputs"][2], "tests/basic.py");
    assert_eq!(data["derived"]["totals"]["files"], 3);
    assert_eq!(data["effort"], Value::Null);
    assert_eq!(data["git"], Value::Null);
    assert!(
        data["warnings"]
            .as_array()
            .expect("warnings array")
            .iter()
            .filter_map(Value::as_str)
            .any(|warning| warning.contains("no host root") && warning.contains("file-backed"))
    );
    assert!(
        data["warnings"]
            .as_array()
            .expect("warnings array")
            .iter()
            .filter_map(Value::as_str)
            .any(|warning| warning.contains("no host root") && warning.contains("git"))
    );
}

#[cfg(feature = "analysis")]
#[test]
fn run_mode_value_analyze_without_preset_defaults_to_receipt_payload() {
    let data = run_mode_value(
        "analyze",
        &json!({
            "inputs": fixture_inputs()
        }),
    )
    .expect("analysis data");

    assert_eq!(data["mode"], "analysis");
    assert_eq!(data["source"]["inputs"][0], "crates/app/src/lib.rs");
    assert_eq!(data["derived"]["totals"]["files"], 3);
    assert_eq!(data["effort"], Value::Null);
}

#[cfg(feature = "analysis")]
#[test]
fn validate_analyze_args_accepts_rootless_receipt_and_estimate() {
    validate_analyze_args_json(
        r#"{
                "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }]
            }"#,
    )
    .expect("missing preset should default to receipt");

    validate_analyze_args_json(
        r#"{
                "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                "analyze": { "preset": "Receipt" }
            }"#,
    )
    .expect("nested mixed-case receipt should be allowed");

    validate_analyze_args_json(
        r#"{
                "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                "preset": "estimate"
            }"#,
    )
    .expect("estimate should be allowed");

    validate_analyze_args_json(
        r#"{
                "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                "analyze": { "preset": "Estimate" }
            }"#,
    )
    .expect("nested mixed-case estimate should be allowed");

    let err = validate_analyze_args_json(
        r#"{
                "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                "preset": "health"
            }"#,
    )
    .expect_err("unsupported preset should be rejected");

    assert!(err.message.contains("preset=\"receipt\""));
    assert!(err.message.contains("preset=\"estimate\""));
}

#[cfg(feature = "analysis")]
#[test]
fn run_json_analyze_rejects_unsupported_presets() {
    let result = run_json(
        "analyze",
        r#"{
                "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                "preset": "health"
            }"#,
    );
    let envelope = tokmd_envelope::ffi::parse_envelope(&result).expect("valid JSON envelope");

    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["error"]["code"], "not_implemented");
    assert!(
        envelope["error"]["message"]
            .as_str()
            .expect("error message")
            .contains("preset=\"receipt\"")
    );
    assert!(
        envelope["error"]["message"]
            .as_str()
            .expect("error message")
            .contains("preset=\"estimate\"")
    );
}

#[cfg(feature = "analysis")]
#[test]
fn run_mode_value_analyze_accepts_nested_case_insensitive_estimate() {
    let data = run_mode_value(
        "analyze",
        &json!({
            "inputs": fixture_inputs(),
            "analyze": { "preset": "Estimate" }
        }),
    )
    .expect("analysis data");

    assert_eq!(data["mode"], "analysis");
    assert_eq!(data["source"]["inputs"][0], "crates/app/src/lib.rs");
    assert_eq!(data["effort"]["model"], "cocomo81-basic");
}

#[test]
fn run_mode_value_surfaces_upstream_errors() {
    let err = run_mode_value(
        "lang",
        &json!({
            "inputs": fixture_inputs(),
            "paths": ["src"]
        }),
    )
    .expect_err("paths + inputs should error");

    assert!(err.contains("[invalid_settings]"));
    assert!(err.contains("cannot be combined with in-memory inputs"));
}

#[test]
fn schema_version_matches_core_receipts() {
    assert_eq!(schema_version(), tokmd_types::SCHEMA_VERSION);
}

#[cfg(feature = "analysis")]
#[test]
fn analysis_schema_version_matches_analysis_receipts() {
    assert_eq!(analysis_schema_version(), CORE_ANALYSIS_SCHEMA_VERSION);
}
