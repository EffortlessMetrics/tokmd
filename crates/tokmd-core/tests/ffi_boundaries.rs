use serde_json::json;
use tokmd_core::ffi::run_json;

#[test]
fn test_strict_nested_object_parsing() {
    let mode = "lang";
    let args = json!({
        "scan": "not an object",
        "paths": ["."]
    });
    let result = run_json(mode, &args.to_string());
    let res_json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(res_json["ok"], false);
    assert_eq!(
        res_json["error"]["message"],
        "Invalid value for 'scan': expected a JSON object"
    );
}
