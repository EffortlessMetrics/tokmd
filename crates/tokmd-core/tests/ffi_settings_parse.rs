#[test]
fn test_ffi_settings_parse_bad_type() {
    let result = tokmd_core::ffi::run_json("lang", r#"{"lang": "string value"}"#);
    println!("result: {}", result);
    let envelope: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(envelope.get("ok").unwrap().as_bool().unwrap(), false);
}
