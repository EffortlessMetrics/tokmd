//! Deterministic property tests ported from fuzz_toml_config.
//!
//! Validates `TomlConfig::parse` does not panic on arbitrary inputs.

use proptest::prelude::*;
use tokmd_settings::TomlConfig;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5000))]

    #[test]
    fn toml_config_parse_never_panics(s in "\\PC*") {
        let _ = TomlConfig::parse(&s);
    }
}
