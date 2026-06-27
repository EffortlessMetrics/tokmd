use tokmd_core::ffi::run_json;

#[test]
fn test_ffi_trust_boundary_non_object() {
    let payload = serde_json::json!({
        "lang": "this_should_be_an_object_not_a_string",
        "top": 10
    });

    let result = run_json("lang", &payload.to_string());
    println!("result: {}", result);

    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // We expect this to fail with an error about `lang` not being an object
    let is_ok = parsed.get("ok").unwrap().as_bool().unwrap();
    assert!(
        !is_ok,
        "Expected an error because 'lang' is a string instead of an object, but got success!"
    );

    let err = parsed
        .get("error")
        .unwrap()
        .get("message")
        .unwrap()
        .as_str()
        .unwrap();
    assert!(
        err.contains("a JSON object"),
        "Error didn't mention being an object: {}",
        err
    );
}
