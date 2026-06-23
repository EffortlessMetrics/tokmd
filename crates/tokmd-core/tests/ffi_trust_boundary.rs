use serde_json::json;
use tokmd_core::ffi::run_json;

#[test]
fn ffi_rejects_non_object_config_blocks() {
    let bad_args = json!({
        "mode": "lang",
        "scan": "should-be-object-or-missing",
        "lang": "should-be-object-or-missing"
    });
    let result = run_json("lang", &bad_args.to_string());
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(
        parsed.get("error").is_some(),
        "Expected error when configuration block is not an object, got success: {}",
        result
    );
}
