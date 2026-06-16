use tokmd_core::ffi::run_json;
use serde_json::Value;

#[test]
fn missing_git_includes_suggestions_in_envelope() {
    let args = serde_json::json!({
        "mode": "diff",
        "from": "missing",
        "to": "also_missing"
    });
    let result = run_json("diff", &args.to_string());
    let value: Value = serde_json::from_str(&result).unwrap();

    assert_eq!(value["ok"], false);

    let error = &value["error"];
    assert!(error.get("suggestions").is_some());
    let suggestions = error["suggestions"].as_array().unwrap();
    assert!(!suggestions.is_empty());
}
