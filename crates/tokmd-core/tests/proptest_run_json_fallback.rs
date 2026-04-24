use proptest::prelude::*;
use serde_json::Value;
use tokmd_core::ffi::run_json;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5000))]

    #[test]
    fn run_json_never_panics(mode in "\\PC*", args_json in "\\PC*") {
        let result = run_json(&mode, &args_json);
        let parsed: Value = serde_json::from_str(&result).expect("run_json must return valid JSON");
        assert!(parsed.get("ok").and_then(Value::as_bool).is_some());
    }
}
