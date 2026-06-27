use serde_json::json;
use tokmd_core::ffi::run_json;

#[test]
fn test_regression_settings_fuzz() {
    let modes = ["lang", "module", "export", "diff"];

    for mode in modes {
        // Send a non-object json value for the settings block
        let input = json!({ mode: "not an object" });

        let envelope_str = run_json(mode, &input.to_string());
        println!("mode: {}, envelope_str: {}", mode, envelope_str);
        let parsed: serde_json::Value = serde_json::from_str(&envelope_str).unwrap();

        let error = parsed.get("error").expect("must have error field");
        let code = error.get("code").and_then(|v| v.as_str()).unwrap();
        assert_eq!(code, "invalid_settings");
    }
}
