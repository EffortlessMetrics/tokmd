use tokmd_core::ffi::run_json;

#[test]
fn ffi_object_verification() {
    let result = run_json("lang", "123");
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
}

#[test]
fn ffi_array_verification() {
    let result = run_json("lang", "[]");
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
}
