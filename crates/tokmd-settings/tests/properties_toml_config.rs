use proptest::prelude::*;
use tokmd_settings::*;

proptest! {
    #[test]
    fn toml_config_parses_valid_utf8(s in ".*") {
        let _ = TomlConfig::parse(&s);
    }

    #[test]
    fn toml_config_json_roundtrip_prop(config in any_toml_config()) {
        if let Ok(json) = serde_json::to_string(&config) {
            let roundtrip = serde_json::from_str::<TomlConfig>(&json).unwrap();

            // Check key fields
            prop_assert_eq!(roundtrip.scan.paths, config.scan.paths);
            prop_assert_eq!(roundtrip.module.roots, config.module.roots);
            prop_assert_eq!(roundtrip.module.depth, config.module.depth);
            prop_assert_eq!(roundtrip.export.format, config.export.format);
            prop_assert_eq!(roundtrip.analyze.preset, config.analyze.preset);
        }
    }

    #[test]
    fn toml_config_toml_roundtrip_prop(config in any_toml_config()) {
        if let Ok(toml_str) = toml::to_string(&config) {
            let roundtrip = TomlConfig::parse(&toml_str).unwrap();

            // Check key fields
            prop_assert_eq!(roundtrip.scan.paths, config.scan.paths);
            prop_assert_eq!(roundtrip.module.roots, config.module.roots);
            prop_assert_eq!(roundtrip.module.depth, config.module.depth);
        }
    }
}

fn any_toml_config() -> impl Strategy<Value = TomlConfig> {
    (
        proptest::option::of(proptest::collection::vec(".*", 0..5)),
        proptest::option::of(proptest::collection::vec(".*", 0..5)),
        proptest::option::of(any::<usize>()),
        proptest::option::of(".*"),
        proptest::option::of(".*"),
    )
        .prop_map(|(paths, roots, depth, format, preset)| {
            let mut config = TomlConfig::default();
            config.scan.paths = paths;
            config.module.roots = roots;
            config.module.depth = depth;
            config.export.format = format;
            config.analyze.preset = preset;
            config
        })
}
