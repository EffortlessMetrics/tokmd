use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::{Value, json};
use tokmd_core::ffi::run_json;

fn assert_ok(result: &str) -> Value {
    let parsed: Value = serde_json::from_str(result).expect("run_json must return valid JSON");
    assert_eq!(parsed["ok"], true, "expected ok envelope: {result}");
    parsed["data"].clone()
}

fn assert_error(result: &str) -> Value {
    let parsed: Value = serde_json::from_str(result).expect("run_json must return valid JSON");
    assert_eq!(parsed["ok"], false, "expected error envelope: {result}");
    parsed["error"].clone()
}

#[cfg(feature = "analysis")]
fn analyze_inputs_json() -> Vec<Value> {
    vec![
        json!({
            "path": "crates/app/src/lib.rs",
            "text": "pub fn alpha() -> usize { 1 }\n"
        }),
        json!({
            "path": "src/main.rs",
            "text": "fn main() {}\n"
        }),
        json!({
            "path": "tests/basic.py",
            "text": "# Smoke test file\nprint('ok')\n"
        }),
    ]
}

#[test]
fn run_json_export_accepts_in_memory_inputs_and_preserves_logical_paths() {
    let args = json!({
        "inputs": [
            {
                "path": "repo/src/lib.rs",
                "text": "pub fn alpha() -> usize { 1 }\n"
            },
            {
                "path": "repo/tests/basic.py",
                "text": "print('ok')\n"
            }
        ],
        "format": "json",
        "strip_prefix": "repo"
    });

    let data = assert_ok(&run_json("export", &args.to_string()));
    let paths: Vec<_> = data["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|row| row["path"].as_str().expect("row path").to_string())
        .collect();

    assert_eq!(data["mode"], "export");
    assert_eq!(data["scan"]["config"], "none");
    assert_eq!(
        data["scan"]["paths"],
        json!(["repo/src/lib.rs", "repo/tests/basic.py"])
    );
    assert_eq!(paths, vec!["src/lib.rs", "tests/basic.py"]);
}

#[test]
fn run_json_lang_and_module_accept_mixed_text_and_base64_inputs() {
    let inputs = vec![
        json!({
            "path": "crates/app/src/lib.rs",
            "base64": BASE64.encode("pub fn alpha() -> usize { 1 }\n")
        }),
        json!({
            "path": "src/main.rs",
            "text": "fn main() {}\n"
        }),
    ];

    let lang_data = assert_ok(&run_json(
        "lang",
        &json!({
            "inputs": inputs,
            "files": true
        })
        .to_string(),
    ));
    let module_data = assert_ok(&run_json(
        "module",
        &json!({
            "inputs": [
                {
                    "path": "crates/app/src/lib.rs",
                    "base64": BASE64.encode("pub fn alpha() -> usize { 1 }\n")
                },
                {
                    "path": "src/main.rs",
                    "text": "fn main() {}\n"
                }
            ]
        })
        .to_string(),
    ));

    assert_eq!(lang_data["mode"], "lang");
    assert_eq!(lang_data["scan"]["config"], "none");
    assert_eq!(lang_data["total"]["files"], 2);
    assert_eq!(module_data["mode"], "module");
    assert_eq!(module_data["scan"]["config"], "none");
    assert_eq!(module_data["total"]["files"], 2);
}

#[test]
fn run_json_export_accepts_top_level_inputs_with_nested_scan_options() {
    let args = json!({
        "inputs": [
            {
                "path": "repo/src/lib.rs",
                "text": "pub fn alpha() -> usize { 1 }\n"
            }
        ],
        "scan": {
            "hidden": true,
            "config": "auto"
        },
        "format": "json"
    });

    let data = assert_ok(&run_json("export", &args.to_string()));

    assert_eq!(data["mode"], "export");
    assert_eq!(data["scan"]["config"], "none");
    assert_eq!(data["scan"]["paths"], json!(["repo/src/lib.rs"]));
    assert_eq!(data["rows"][0]["path"], "repo/src/lib.rs");
}

#[cfg(feature = "analysis")]
#[test]
fn run_json_analyze_estimate_accepts_in_memory_inputs() {
    let args = json!({
        "inputs": analyze_inputs_json(),
        "preset": "estimate"
    });

    let data = assert_ok(&run_json("analyze", &args.to_string()));

    assert_eq!(data["mode"], "analysis");
    assert_eq!(
        data["source"]["inputs"],
        json!(["crates/app/src/lib.rs", "src/main.rs", "tests/basic.py"])
    );
    assert_eq!(data["derived"]["totals"]["files"], 3);
    assert_eq!(data["derived"]["totals"]["code"], 3);
    assert_eq!(data["derived"]["totals"]["comments"], 1);
    assert_eq!(data["derived"]["totals"]["lines"], 4);
    assert_eq!(data["effort"]["model"], "cocomo81-basic");
    assert_eq!(data["effort"]["size_basis"]["total_lines"], 3);
    assert_eq!(data["effort"]["size_basis"]["authored_lines"], 3);
    assert_eq!(data["effort"]["size_basis"]["generated_lines"], 0);
    assert_eq!(data["effort"]["size_basis"]["vendored_lines"], 0);
}

#[cfg(feature = "analysis")]
#[test]
fn run_json_analyze_health_accepts_nested_scan_inputs() {
    let args = json!({
        "scan": {
            "inputs": analyze_inputs_json()
        },
        "analyze": {
            "preset": "health"
        }
    });

    let data = assert_ok(&run_json("analyze", &args.to_string()));
    let tags = data["derived"]["todo"]["tags"]
        .as_array()
        .expect("todo tags");

    assert_eq!(data["mode"], "analysis");
    assert_eq!(
        data["source"]["inputs"],
        json!(["crates/app/src/lib.rs", "src/main.rs", "tests/basic.py"])
    );
    assert_eq!(data["derived"]["totals"]["files"], 3);
    assert!(data["derived"]["todo"]["total"].as_u64().unwrap_or(0) > 0);
    assert!(tags.iter().any(|tag| {
        tag["tag"]
            .as_str()
            .map(|value| value.eq_ignore_ascii_case("todo"))
            .unwrap_or(false)
    }));
}

#[test]
fn run_json_rejects_paths_when_inputs_are_present() {
    let result = run_json(
        "export",
        &json!({
            "paths": ["."],
            "inputs": [
                {
                    "path": "src/lib.rs",
                    "text": "pub fn alpha() {}\n"
                }
            ]
        })
        .to_string(),
    );
    let error = assert_error(&result);

    assert_eq!(error["code"], "invalid_settings");
    assert!(
        error["message"]
            .as_str()
            .expect("error message")
            .contains("paths")
    );
}

#[test]
fn run_json_allows_null_paths_when_inputs_are_present() {
    let data = assert_ok(&run_json(
        "export",
        &json!({
            "inputs": [
                {
                    "path": "src/lib.rs",
                    "text": "pub fn alpha() {}\n"
                }
            ],
            "paths": null,
            "format": "json"
        })
        .to_string(),
    ));

    assert_eq!(data["scan"]["paths"], json!(["src/lib.rs"]));
    assert_eq!(data["rows"][0]["path"], "src/lib.rs");
}
