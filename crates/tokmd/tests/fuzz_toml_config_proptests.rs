//! Deterministic property tests extracted from `fuzz_toml_config`.
//!
//! Tests `TomlConfig::parse()` and serialization round-trip behavior.

use proptest::prelude::*;
use tokmd_settings::{TomlConfig};

proptest! {
    #[test]
    fn toml_config_does_not_crash_on_random_utf8(s in "\\PC*") {
        let _ = TomlConfig::parse(&s);
    }
}

// We use basic explicit tests for serde roundtrips because it avoids
// exhaustively hand-rolling an exact proptest strategy for every new optional field.
#[test]
fn toml_config_json_roundtrip() {
    let toml_str = r#"
[scan]
paths = ["src", "lib"]

[module]
roots = ["src"]
depth = 2

[export]
format = "json"

[analyze]
preset = "health"

[view.default]
format = "md"
top = 10
"#;
    let config = TomlConfig::parse(toml_str).expect("should parse");
    let json = serde_json::to_string(&config).expect("should serialize json");
    let roundtrip: TomlConfig = serde_json::from_str(&json).expect("should parse json");

    assert_eq!(config.scan.paths, roundtrip.scan.paths);
    assert_eq!(config.module.roots, roundtrip.module.roots);
    assert_eq!(config.module.depth, roundtrip.module.depth);
    assert_eq!(config.export.format, roundtrip.export.format);
    assert_eq!(config.analyze.preset, roundtrip.analyze.preset);
    assert_eq!(config.view.len(), roundtrip.view.len());
}

#[test]
fn toml_config_toml_roundtrip() {
    let toml_str = r#"
[scan]
paths = ["src", "lib"]

[module]
roots = ["src"]
depth = 2

[export]
format = "json"

[analyze]
preset = "health"

[view.default]
format = "md"
top = 10
"#;
    let config = TomlConfig::parse(toml_str).expect("should parse");
    let serialized = toml::to_string(&config).expect("should serialize toml");
    let roundtrip = TomlConfig::parse(&serialized).expect("should parse serialized toml");

    assert_eq!(config.scan.paths, roundtrip.scan.paths);
    assert_eq!(config.module.roots, roundtrip.module.roots);
    assert_eq!(config.module.depth, roundtrip.module.depth);
    assert_eq!(config.export.format, roundtrip.export.format);
    assert_eq!(config.analyze.preset, roundtrip.analyze.preset);
    assert_eq!(config.view.len(), roundtrip.view.len());
}
