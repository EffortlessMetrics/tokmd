use tokmd_settings::{TomlConfig, ViewProfile};

#[test]
fn test_toml_config_serialization_determinism() {
    let mut config = TomlConfig::default();

    // Add multiple view profiles with keys that are not already sorted alphabetically in insert order
    // to ensure insert order doesn't mask the issue if HashMap preserves it by chance.
    let keys = vec!["zebra", "apple", "mango", "banana", "cherry"];

    for key in &keys {
        config.view.insert(key.to_string(), ViewProfile::default());
    }

    let toml_output = toml::to_string_pretty(&config).unwrap();
    println!("{}", toml_output);

    let apple_pos = toml_output.find("[view.apple]").expect("should find apple");
    let banana_pos = toml_output
        .find("[view.banana]")
        .expect("should find banana");
    let cherry_pos = toml_output
        .find("[view.cherry]")
        .expect("should find cherry");
    let mango_pos = toml_output.find("[view.mango]").expect("should find mango");
    let zebra_pos = toml_output.find("[view.zebra]").expect("should find zebra");

    // With BTreeMap, these must be in order.
    // With HashMap, they MIGHT be in order, but we want to enforce BTreeMap.
    // We can't strictly "fail" if HashMap happens to be sorted, but this test
    // serves as a regression test once we switch to BTreeMap.

    assert!(apple_pos < banana_pos, "apple should be before banana");
    assert!(banana_pos < cherry_pos, "banana should be before cherry");
    assert!(cherry_pos < mango_pos, "cherry should be before mango");
    assert!(mango_pos < zebra_pos, "mango should be before zebra");
}
