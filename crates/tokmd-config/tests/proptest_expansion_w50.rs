//! Property-based tests for tokmd-config (W50 expansion).
//!
//! Verifies TOML roundtrip stability, profile merging, and
//! configuration type invariants with arbitrary inputs.

use proptest::prelude::*;
use std::collections::BTreeMap;
use tokmd_config::{Profile, UserConfig};

// ── Strategies ───────────────────────────────────────────────────────────────

fn arb_profile() -> impl Strategy<Value = Profile> {
    (
        prop::option::of(prop_oneof![
            Just("json".to_string()),
            Just("md".to_string()),
            Just("tsv".to_string()),
            Just("csv".to_string()),
            Just("jsonl".to_string()),
        ]),
        prop::option::of(0usize..100),
        prop::option::of(any::<bool>()),
        prop::option::of(prop::collection::vec("[a-z]{1,10}", 0..5)),
        prop::option::of(0usize..10),
        prop::option::of(0usize..1000),
        prop::option::of(0usize..10000),
        prop::option::of(prop_oneof![
            Just(tokmd_config::RedactMode::None),
            Just(tokmd_config::RedactMode::Paths),
            Just(tokmd_config::RedactMode::All),
        ]),
        prop::option::of(any::<bool>()),
        prop::option::of(prop_oneof![
            Just("collapse".to_string()),
            Just("separate".to_string()),
            Just("parents-only".to_string()),
        ]),
    )
        .prop_map(
            |(
                format,
                top,
                files,
                module_roots,
                module_depth,
                min_code,
                max_rows,
                redact,
                meta,
                children,
            )| {
                Profile {
                    format,
                    top,
                    files,
                    module_roots,
                    module_depth,
                    min_code,
                    max_rows,
                    redact,
                    meta,
                    children,
                }
            },
        )
}

fn arb_user_config() -> impl Strategy<Value = UserConfig> {
    (
        prop::collection::btree_map("[a-z_]{1,15}", arb_profile(), 0..5),
        prop::collection::btree_map("[a-z]{1,10}/[a-z]{1,10}", "[a-z_]{1,15}", 0..5),
    )
        .prop_map(|(profiles, repos)| UserConfig { profiles, repos })
}

// ── TOML roundtrip tests ─────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn profile_json_roundtrip(profile in arb_profile()) {
        let json = serde_json::to_string(&profile).unwrap();
        let round: Profile = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(round.format, profile.format);
        prop_assert_eq!(round.top, profile.top);
        prop_assert_eq!(round.files, profile.files);
        prop_assert_eq!(round.module_roots, profile.module_roots);
        prop_assert_eq!(round.module_depth, profile.module_depth);
    }

    #[test]
    fn user_config_json_roundtrip(config in arb_user_config()) {
        let json = serde_json::to_string(&config).unwrap();
        let round: UserConfig = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(round.profiles.len(), config.profiles.len());
        prop_assert_eq!(round.repos.len(), config.repos.len());
    }

    #[test]
    fn user_config_toml_roundtrip(config in arb_user_config()) {
        let toml_str = toml::to_string(&config).unwrap();
        let round: UserConfig = toml::from_str(&toml_str).unwrap();
        prop_assert_eq!(round.profiles.len(), config.profiles.len());
        prop_assert_eq!(round.repos.len(), config.repos.len());
    }

    #[test]
    fn profile_toml_roundtrip(profile in arb_profile()) {
        let toml_str = toml::to_string(&profile).unwrap();
        let round: Profile = toml::from_str(&toml_str).unwrap();
        prop_assert_eq!(round.format, profile.format);
        prop_assert_eq!(round.top, profile.top);
    }
}

// ── Profile merging tests ────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn empty_config_has_no_profiles(_dummy in 0u8..1) {
        let config = UserConfig::default();
        prop_assert!(config.profiles.is_empty());
        prop_assert!(config.repos.is_empty());
    }

    #[test]
    fn profiles_are_independent(
        profile_a in arb_profile(),
        profile_b in arb_profile()
    ) {
        let mut config = UserConfig::default();
        config.profiles.insert("a".to_string(), profile_a.clone());
        config.profiles.insert("b".to_string(), profile_b.clone());

        // Each profile should be retrievable independently
        let a = &config.profiles["a"];
        let b = &config.profiles["b"];
        prop_assert_eq!(a.format.clone(), profile_a.format);
        prop_assert_eq!(b.format.clone(), profile_b.format);
    }

    #[test]
    fn profile_count_matches_inserts(
        profiles in prop::collection::btree_map("[a-z]{1,10}", arb_profile(), 0..10)
    ) {
        let config = UserConfig {
            profiles: profiles.clone(),
            repos: BTreeMap::new(),
        };
        prop_assert_eq!(config.profiles.len(), profiles.len());
    }

    #[test]
    fn repo_mapping_lookups(
        repo in "[a-z]{1,10}/[a-z]{1,10}",
        profile_name in "[a-z_]{1,15}"
    ) {
        let mut config = UserConfig::default();
        config.repos.insert(repo.clone(), profile_name.clone());
        prop_assert_eq!(config.repos.get(&repo), Some(&profile_name));
    }
}

// ── Default value tests ──────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn default_profile_all_none(_dummy in 0u8..1) {
        let profile = Profile::default();
        prop_assert!(profile.format.is_none());
        prop_assert!(profile.top.is_none());
        prop_assert!(profile.files.is_none());
        prop_assert!(profile.module_roots.is_none());
        prop_assert!(profile.module_depth.is_none());
        prop_assert!(profile.min_code.is_none());
        prop_assert!(profile.max_rows.is_none());
        prop_assert!(profile.redact.is_none());
        prop_assert!(profile.meta.is_none());
        prop_assert!(profile.children.is_none());
    }

    #[test]
    fn profile_serialization_skips_none(_dummy in 0u8..1) {
        let profile = Profile::default();
        let json = serde_json::to_string(&profile).unwrap();
        // Default profile serialized to JSON should be relatively small
        prop_assert!(json.len() < 500, "Default profile JSON too large: {}", json.len());
    }
}
