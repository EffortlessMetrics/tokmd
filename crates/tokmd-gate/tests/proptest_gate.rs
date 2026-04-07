use proptest::prelude::*;
use tokmd_gate::{resolve_pointer, evaluate_policy, PolicyConfig};
use serde_json::json;

proptest! {
    #[test]
    fn test_resolve_pointer_never_panics(ptr in ".*") {
        let doc = json!({
            "foo": {"bar": 42},
            "arr": [1, 2, 3],
            "nested": [{"a": 1}, {"b": 2}]
        });

        let _ = resolve_pointer(&doc, &ptr);
    }
}
