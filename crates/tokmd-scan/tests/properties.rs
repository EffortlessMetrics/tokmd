//! Property-based tests for tokmd-scan.
//!
//! These tests verify that the ScanOptions to tokei Config mapping is correct
//! and never panics for any valid combination of inputs.
//!
//! ## Test Coverage
//!
//! 1. **Flag implication**: `no_ignore` implies all `no_ignore_*` flags
//! 2. **Config mapping never panics**: Any valid ScanOptions produces valid Config
//! 3. **ConfigMode handling**: Auto vs None both succeed without panicking
//! 4. **Excluded patterns**: Empty, multiple, and special glob patterns work

use proptest::prelude::*;
use proptest::string::string_regex;
use tokmd_settings::ScanOptions;
use tokmd_types::ConfigMode;

// ============================================================================
// Strategies
// ============================================================================

/// Strategy for generating valid exclude patterns.
///
/// Includes common patterns like:
/// - Simple directory names (target, node_modules)
/// - Glob patterns (*.min.js, **/*.bak)
/// - Paths with wildcards (src/*.test.ts)
fn arb_exclude_pattern() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple directory names
        Just("target".to_string()),
        Just("node_modules".to_string()),
        Just(".git".to_string()),
        Just("dist".to_string()),
        Just("build".to_string()),
        Just("vendor".to_string()),
        // Extension globs
        Just("*.min.js".to_string()),
        Just("*.min.css".to_string()),
        Just("*.bak".to_string()),
        Just("*.log".to_string()),
        Just("*.tmp".to_string()),
        // Double-star patterns
        Just("**/*.test.ts".to_string()),
        Just("**/*.spec.js".to_string()),
        Just("**/test/**".to_string()),
        Just("**/tests/**".to_string()),
        Just("**/fixtures/**".to_string()),
        // Generated patterns with special chars
        string_regex("[a-zA-Z0-9_-]{1,20}").unwrap(),
        string_regex("[a-zA-Z0-9_-]{1,10}/[a-zA-Z0-9_-]{1,10}").unwrap(),
        // Patterns with brackets, question marks
        Just("src/[test]/**".to_string()),
        Just("*.?".to_string()),
    ]
}

/// Strategy for generating a vector of exclude patterns (0 to 5 patterns).
fn arb_excluded() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(arb_exclude_pattern(), 0..=5)
}

/// Strategy for generating ConfigMode values.
fn arb_config_mode() -> impl Strategy<Value = ConfigMode> {
    prop_oneof![Just(ConfigMode::Auto), Just(ConfigMode::None),]
}

/// Strategy for generating arbitrary ScanOptions.
///
/// This generates all possible combinations of boolean flags, exclude patterns,
/// config modes, and verbosity levels.
fn arb_global_args() -> impl Strategy<Value = ScanOptions> {
    (
        arb_excluded(),
        arb_config_mode(),
        any::<bool>(), // hidden
        any::<bool>(), // no_ignore
        any::<bool>(), // no_ignore_parent
        any::<bool>(), // no_ignore_dot
        any::<bool>(), // no_ignore_vcs
        any::<bool>(), // treat_doc_strings_as_comments
    )
        .prop_map(
            |(
                excluded,
                config,
                hidden,
                no_ignore,
                no_ignore_parent,
                no_ignore_dot,
                no_ignore_vcs,
                treat_doc_strings_as_comments,
            )| ScanOptions {
                excluded,
                config,
                hidden,
                no_ignore,
                no_ignore_parent,
                no_ignore_dot,
                no_ignore_vcs,
                treat_doc_strings_as_comments,
            },
        )
}

/// Strategy for ScanOptions with no_ignore = true.
fn arb_global_args_with_no_ignore() -> impl Strategy<Value = ScanOptions> {
    arb_global_args().prop_map(|mut args| {
        args.no_ignore = true;
        args
    })
}

/// Strategy for ScanOptions with empty excluded list.
fn arb_global_args_empty_excluded() -> impl Strategy<Value = ScanOptions> {
    arb_global_args().prop_map(|mut args| {
        args.excluded = vec![];
        args
    })
}

/// Strategy for ScanOptions with many exclude patterns (stress test).
fn arb_global_args_many_excludes() -> impl Strategy<Value = ScanOptions> {
    (
        arb_global_args(),
        prop::collection::vec(arb_exclude_pattern(), 10..=20),
    )
        .prop_map(|(mut args, excludes)| {
            args.excluded = excludes;
            args
        })
}

// ============================================================================
// Config Mapping Tests
// ============================================================================

proptest! {
    /// Any valid ScanOptions should produce a tokei Config without panicking.
    ///
    /// This is the primary safety property: we can't test the actual scan
    /// behavior without filesystem access, but we can ensure the configuration
    /// mapping is robust.
    #[test]
    fn config_mapping_never_panics(args in arb_global_args()) {
        // Simulate the config building logic from lib.rs
        let mut cfg = match args.config {
            ConfigMode::Auto => tokei::Config::from_config_files(),
            ConfigMode::None => tokei::Config::default(),
        };

        // Apply all flags - should not panic
        if args.hidden {
            cfg.hidden = Some(true);
        }
        if args.no_ignore {
            cfg.no_ignore = Some(true);
            cfg.no_ignore_dot = Some(true);
            cfg.no_ignore_parent = Some(true);
            cfg.no_ignore_vcs = Some(true);
        }
        if args.no_ignore_dot {
            cfg.no_ignore_dot = Some(true);
        }
        if args.no_ignore_parent {
            cfg.no_ignore_parent = Some(true);
        }
        if args.no_ignore_vcs {
            cfg.no_ignore_vcs = Some(true);
        }
        if args.treat_doc_strings_as_comments {
            cfg.treat_doc_strings_as_comments = Some(true);
        }

        // The excluded patterns should be convertible to string slices
        let _ignores: Vec<&str> = args.excluded.iter().map(|s| s.as_str()).collect();

        // Verify config fields are accessible
        let _ = cfg.hidden;
        let _ = cfg.no_ignore;
    }

    /// ConfigMode::Auto should produce a valid config.
    #[test]
    fn config_mode_auto_succeeds(_dummy in 0..100u8) {
        let cfg = tokei::Config::from_config_files();
        // Should not panic, verify fields are accessible
        let _ = cfg.hidden;
        let _ = cfg.no_ignore;
    }

    /// ConfigMode::None should produce a valid default config.
    #[test]
    fn config_mode_none_succeeds(_dummy in 0..100u8) {
        let cfg = tokei::Config::default();
        // Should not panic, verify fields are accessible
        let _ = cfg.hidden;
        let _ = cfg.no_ignore;
    }
}

// ============================================================================
// Flag Implication Tests
// ============================================================================

proptest! {
    /// When no_ignore is true, all no_ignore_* flags should be effectively true.
    ///
    /// This tests the semantic implication: setting no_ignore=true is equivalent
    /// to setting all individual no_ignore_* flags to true.
    #[test]
    fn no_ignore_implies_all_no_ignore_flags(args in arb_global_args_with_no_ignore()) {
        // Build config the same way as lib.rs
        let mut cfg = match args.config {
            ConfigMode::Auto => tokei::Config::from_config_files(),
            ConfigMode::None => tokei::Config::default(),
        };

        // Apply flags (mimicking lib.rs logic)
        if args.hidden {
            cfg.hidden = Some(true);
        }
        if args.no_ignore {
            cfg.no_ignore = Some(true);
            cfg.no_ignore_dot = Some(true);
            cfg.no_ignore_parent = Some(true);
            cfg.no_ignore_vcs = Some(true);
        }
        if args.no_ignore_dot {
            cfg.no_ignore_dot = Some(true);
        }
        if args.no_ignore_parent {
            cfg.no_ignore_parent = Some(true);
        }
        if args.no_ignore_vcs {
            cfg.no_ignore_vcs = Some(true);
        }
        if args.treat_doc_strings_as_comments {
            cfg.treat_doc_strings_as_comments = Some(true);
        }

        // When no_ignore is true, all no_ignore_* should be set
        prop_assert!(args.no_ignore);
        prop_assert_eq!(cfg.no_ignore, Some(true));
        prop_assert_eq!(cfg.no_ignore_dot, Some(true));
        prop_assert_eq!(cfg.no_ignore_parent, Some(true));
        prop_assert_eq!(cfg.no_ignore_vcs, Some(true));
    }

    /// Individual no_ignore_* flags are independent when no_ignore is false.
    ///
    /// Setting one flag should not affect the others (unless no_ignore is true).
    #[test]
    fn individual_flags_are_independent(args in arb_global_args()) {
        prop_assume!(!args.no_ignore);

        let mut cfg = tokei::Config::default();

        // Apply only the individual flags (not no_ignore)
        if args.no_ignore_dot {
            cfg.no_ignore_dot = Some(true);
        }
        if args.no_ignore_parent {
            cfg.no_ignore_parent = Some(true);
        }
        if args.no_ignore_vcs {
            cfg.no_ignore_vcs = Some(true);
        }

        // Each flag should match its input
        prop_assert_eq!(cfg.no_ignore_dot.unwrap_or(false), args.no_ignore_dot);
        prop_assert_eq!(cfg.no_ignore_parent.unwrap_or(false), args.no_ignore_parent);
        prop_assert_eq!(cfg.no_ignore_vcs.unwrap_or(false), args.no_ignore_vcs);
    }
}

// ============================================================================
// Excluded Pattern Tests
// ============================================================================

proptest! {
    /// Empty excluded list should work.
    #[test]
    fn empty_excluded_works(args in arb_global_args_empty_excluded()) {
        prop_assert!(args.excluded.is_empty());

        // Should be able to convert to ignores slice
        let ignores: Vec<&str> = args.excluded.iter().map(|s| s.as_str()).collect();
        prop_assert!(ignores.is_empty());
    }

    /// Multiple exclude patterns should work.
    #[test]
    fn multiple_excluded_patterns_work(args in arb_global_args_many_excludes()) {
        prop_assert!(args.excluded.len() >= 10);

        // All patterns should be convertible to string slices
        let ignores: Vec<&str> = args.excluded.iter().map(|s| s.as_str()).collect();
        prop_assert_eq!(ignores.len(), args.excluded.len());
    }

    /// Patterns with special glob characters should be handled.
    #[test]
    fn special_glob_patterns_work(pattern in arb_exclude_pattern()) {
        // Patterns containing *, ?, [, ] should not cause panics
        let ignores: Vec<&str> = vec![&pattern];
        prop_assert_eq!(ignores.len(), 1);
        prop_assert_eq!(ignores[0], pattern.as_str());
    }
}

// ============================================================================
// Boolean Flag Combination Tests
// ============================================================================

proptest! {
    /// All boolean flags set to true should work.
    #[test]
    fn all_flags_true_works(_dummy in 0..100u8) {
        let args = ScanOptions {
            excluded: vec!["target".to_string(), "node_modules".to_string()],
            config: ConfigMode::None,
            hidden: true,
            no_ignore: true,
            no_ignore_parent: true,
            no_ignore_dot: true,
            no_ignore_vcs: true,
            treat_doc_strings_as_comments: true,
        };

        // Build config
        let cfg = tokei::Config {
            hidden: Some(true),
            no_ignore: Some(true),
            no_ignore_dot: Some(true),
            no_ignore_parent: Some(true),
            no_ignore_vcs: Some(true),
            treat_doc_strings_as_comments: Some(true),
            ..Default::default()
        };

        // All should be true
        prop_assert_eq!(cfg.hidden, Some(true));
        prop_assert_eq!(cfg.no_ignore, Some(true));
        prop_assert_eq!(cfg.treat_doc_strings_as_comments, Some(true));

        // excluded should be valid
        let ignores: Vec<&str> = args.excluded.iter().map(|s| s.as_str()).collect();
        prop_assert_eq!(ignores.len(), 2);
    }

    /// All boolean flags set to false should work.
    #[test]
    fn all_flags_false_works(_dummy in 0..100u8) {
        let args = ScanOptions {
            excluded: vec![],
            config: ConfigMode::Auto,
            hidden: false,
            no_ignore: false,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
        };

        // With all flags false, config remains at defaults
        let cfg = tokei::Config::from_config_files();

        // The config should be valid (may have values from config files)
        let _ = cfg.hidden;

        // Excluded should be empty
        prop_assert!(args.excluded.is_empty());
    }

    /// Hidden flag should be independent of ignore flags.
    #[test]
    fn hidden_flag_independent(hidden in any::<bool>(), no_ignore in any::<bool>()) {
        let args = ScanOptions {
            excluded: vec![],
            config: ConfigMode::None,
            hidden,
            no_ignore,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
        };

        let mut cfg = tokei::Config::default();
        if args.hidden {
            cfg.hidden = Some(true);
        }
        if args.no_ignore {
            cfg.no_ignore = Some(true);
        }

        // hidden should match input
        prop_assert_eq!(cfg.hidden.unwrap_or(false), hidden);
        // no_ignore should match input
        prop_assert_eq!(cfg.no_ignore.unwrap_or(false), no_ignore);
    }

    /// treat_doc_strings_as_comments flag should be independent.
    #[test]
    fn doc_strings_flag_independent(treat_doc in any::<bool>(), no_ignore in any::<bool>()) {
        let mut cfg = tokei::Config::default();

        if treat_doc {
            cfg.treat_doc_strings_as_comments = Some(true);
        }
        if no_ignore {
            cfg.no_ignore = Some(true);
        }

        prop_assert_eq!(cfg.treat_doc_strings_as_comments.unwrap_or(false), treat_doc);
        prop_assert_eq!(cfg.no_ignore.unwrap_or(false), no_ignore);
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

proptest! {
    /// Default ScanOptions should produce valid config.
    #[test]
    fn default_global_args_work(_dummy in 0..100u8) {
        let args = ScanOptions::default();

        // Should have sensible defaults
        prop_assert!(args.excluded.is_empty());
        prop_assert!(!args.hidden);
        prop_assert!(!args.no_ignore);
        prop_assert!(!args.no_ignore_dot);
        prop_assert!(!args.no_ignore_parent);
        prop_assert!(!args.no_ignore_vcs);
        prop_assert!(!args.treat_doc_strings_as_comments);

        // Config creation should succeed
        let _cfg = tokei::Config::default();
    }
}

// ============================================================================
// Consistency Tests
// ============================================================================

proptest! {
    /// ScanOptions to config mapping should be deterministic.
    ///
    /// The same ScanOptions should always produce the same Config behavior.
    #[test]
    fn config_mapping_is_deterministic(args in arb_global_args()) {
        // Build config twice with the same args
        let build_config = |args: &ScanOptions| {
            let mut cfg = match args.config {
                ConfigMode::Auto => tokei::Config::from_config_files(),
                ConfigMode::None => tokei::Config::default(),
            };

            if args.hidden {
                cfg.hidden = Some(true);
            }
            if args.no_ignore {
                cfg.no_ignore = Some(true);
                cfg.no_ignore_dot = Some(true);
                cfg.no_ignore_parent = Some(true);
                cfg.no_ignore_vcs = Some(true);
            }
            if args.no_ignore_dot {
                cfg.no_ignore_dot = Some(true);
            }
            if args.no_ignore_parent {
                cfg.no_ignore_parent = Some(true);
            }
            if args.no_ignore_vcs {
                cfg.no_ignore_vcs = Some(true);
            }
            if args.treat_doc_strings_as_comments {
                cfg.treat_doc_strings_as_comments = Some(true);
            }
            cfg
        };

        let cfg1 = build_config(&args);
        let cfg2 = build_config(&args);

        // The resulting configs should have the same values
        prop_assert_eq!(cfg1.hidden, cfg2.hidden);
        prop_assert_eq!(cfg1.no_ignore, cfg2.no_ignore);
        prop_assert_eq!(cfg1.no_ignore_dot, cfg2.no_ignore_dot);
        prop_assert_eq!(cfg1.no_ignore_parent, cfg2.no_ignore_parent);
        prop_assert_eq!(cfg1.no_ignore_vcs, cfg2.no_ignore_vcs);
        prop_assert_eq!(cfg1.treat_doc_strings_as_comments, cfg2.treat_doc_strings_as_comments);
    }
}

// ============================================================================
// Edge-Case Tests (additional property coverage)
// ============================================================================

proptest! {
    /// Duplicate exclude patterns should not cause panics or alter semantics.
    #[test]
    fn duplicate_excluded_patterns_are_harmless(pattern in arb_exclude_pattern()) {
        let args = ScanOptions {
            excluded: vec![pattern.clone(), pattern.clone(), pattern],
            config: ConfigMode::None,
            hidden: false,
            no_ignore: false,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
        };

        let ignores: Vec<&str> = args.excluded.iter().map(|s| s.as_str()).collect();
        prop_assert_eq!(ignores.len(), 3);
    }

    /// Enabling *only* no_ignore_dot should not touch the other ignore fields.
    #[test]
    fn no_ignore_dot_alone_does_not_set_others(_dummy in 0..50u8) {
        let cfg = tokei::Config {
            no_ignore_dot: Some(true),
            ..Default::default()
        };

        prop_assert_eq!(cfg.no_ignore_dot, Some(true));
        // The other fields should remain at their defaults (None).
        prop_assert!(cfg.no_ignore.is_none() || cfg.no_ignore == Some(false));
        prop_assert!(cfg.no_ignore_parent.is_none() || cfg.no_ignore_parent == Some(false));
        prop_assert!(cfg.no_ignore_vcs.is_none() || cfg.no_ignore_vcs == Some(false));
    }

    /// Enabling *only* no_ignore_vcs should not touch the other ignore fields.
    #[test]
    fn no_ignore_vcs_alone_does_not_set_others(_dummy in 0..50u8) {
        let cfg = tokei::Config {
            no_ignore_vcs: Some(true),
            ..Default::default()
        };

        prop_assert_eq!(cfg.no_ignore_vcs, Some(true));
        prop_assert!(cfg.no_ignore.is_none() || cfg.no_ignore == Some(false));
        prop_assert!(cfg.no_ignore_parent.is_none() || cfg.no_ignore_parent == Some(false));
        prop_assert!(cfg.no_ignore_dot.is_none() || cfg.no_ignore_dot == Some(false));
    }

    /// The `hidden` flag should never affect any `no_ignore*` config fields.
    #[test]
    fn hidden_never_affects_ignore_fields(hidden in any::<bool>()) {
        let mut cfg = tokei::Config::default();
        if hidden {
            cfg.hidden = Some(true);
        }

        // None of the ignore fields should be set.
        prop_assert!(cfg.no_ignore.is_none());
        prop_assert!(cfg.no_ignore_dot.is_none());
        prop_assert!(cfg.no_ignore_parent.is_none());
        prop_assert!(cfg.no_ignore_vcs.is_none());
    }

    /// Exclude list length should be preserved through the mapping.
    #[test]
    fn excluded_length_preserved(args in arb_global_args()) {
        let ignores: Vec<&str> = args.excluded.iter().map(|s| s.as_str()).collect();
        prop_assert_eq!(ignores.len(), args.excluded.len());
    }

    /// Config built with ConfigMode::None should have all fields at defaults
    /// *before* any flag overrides are applied.
    #[test]
    fn config_none_starts_with_defaults(_dummy in 0..50u8) {
        let cfg = tokei::Config::default();
        prop_assert!(cfg.hidden.is_none());
        prop_assert!(cfg.no_ignore.is_none());
        prop_assert!(cfg.no_ignore_dot.is_none());
        prop_assert!(cfg.no_ignore_parent.is_none());
        prop_assert!(cfg.no_ignore_vcs.is_none());
        prop_assert!(cfg.treat_doc_strings_as_comments.is_none());
    }

    /// Setting no_ignore after individual flags should still result in all
    /// no_ignore_* being true (order-independence).
    #[test]
    fn no_ignore_overrides_regardless_of_order(
        dot in any::<bool>(),
        parent in any::<bool>(),
        vcs in any::<bool>(),
    ) {
        // Apply individual flags first, then no_ignore.
        let mut cfg = tokei::Config::default();
        if dot { cfg.no_ignore_dot = Some(true); }
        if parent { cfg.no_ignore_parent = Some(true); }
        if vcs { cfg.no_ignore_vcs = Some(true); }

        // Now apply no_ignore (like scan() does).
        cfg.no_ignore = Some(true);
        cfg.no_ignore_dot = Some(true);
        cfg.no_ignore_parent = Some(true);
        cfg.no_ignore_vcs = Some(true);

        prop_assert_eq!(cfg.no_ignore, Some(true));
        prop_assert_eq!(cfg.no_ignore_dot, Some(true));
        prop_assert_eq!(cfg.no_ignore_parent, Some(true));
        prop_assert_eq!(cfg.no_ignore_vcs, Some(true));
    }
}
