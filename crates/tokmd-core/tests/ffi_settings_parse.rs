#[test]
fn test_ffi_settings_parse_bad_type() {
    let result = tokmd_core::ffi::run_json("lang", r#"{"lang": "string value"}"#);
    let ok = result.contains(r#""ok":false"#);
    assert_eq!(ok, true);
}
