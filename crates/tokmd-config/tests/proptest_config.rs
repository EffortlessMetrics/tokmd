use proptest::prelude::*;
use tokmd_config::{TomlConfig, Profile, UserConfig};

proptest! {
    #[test]
    fn test_toml_config_deterministic(toml_str in "[a-zA-Z0-9_ \\n=]{0,100}") {
        if let Ok(config1) = TomlConfig::parse(&toml_str) {
            let config2 = TomlConfig::parse(&toml_str).unwrap();

            // Check essential fields instead of Eq
            assert_eq!(config1.scan.paths, config2.scan.paths);
            assert_eq!(config1.module.roots, config2.module.roots);
            assert_eq!(config1.module.depth, config2.module.depth);
            assert_eq!(config1.view.len(), config2.view.len());
        }
    }
}
