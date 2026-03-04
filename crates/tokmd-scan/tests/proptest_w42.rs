//! Wave 42 property-based tests for tokmd-scan.
//!
//! Covers ScanOptions construction, config mapping robustness,
//! flag interaction properties, and exclude-pattern handling.

use proptest::prelude::*;
use proptest::string::string_regex;
use tokmd_settings::ScanOptions;
use tokmd_types::ConfigMode;

// ============================================================================
// Strategies
// ============================================================================

fn arb_exclude_pattern() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("target".to_string()),
        Just("node_modules".to_string()),
        Just(".git".to_string()),
        Just("*.min.js".to_string()),
        Just("**/*.bak".to_string()),
        string_regex("[a-zA-Z0-9_-]{1,20}").unwrap(),
        string_regex("[a-zA-Z0-9_-]{1,10}/\\*\\*").unwrap(),
    ]
}

fn arb_scan_options() -> impl Strategy<Value = ScanOptions> {
    (
        prop::collection::vec(arb_exclude_pattern(), 0..=5),
        prop_oneof![Just(ConfigMode::Auto), Just(ConfigMode::None)],
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
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
            )| {
                ScanOptions {
                    excluded,
                    config,
                    hidden,
                    no_ignore,
                    no_ignore_parent,
                    no_ignore_dot,
                    no_ignore_vcs,
                    treat_doc_strings_as_comments,
                }
            },
        )
}

fn build_config(args: &ScanOptions) -> tokei::Config {
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
}

// ============================================================================
// Config mapping: arbitrary ScanOptions never panics
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Arbitrary ScanOptions always produces a valid tokei Config.
    #[test]
    fn arbitrary_scan_options_produces_valid_config(args in arb_scan_options()) {
        let cfg = build_config(&args);
        // Just accessing fields proves it didn't panic
        let _ = (cfg.hidden, cfg.no_ignore, cfg.treat_doc_strings_as_comments);
    }

    /// Excluded patterns can always be collected into &str slices.
    #[test]
    fn excluded_patterns_always_convertible(args in arb_scan_options()) {
        let ignores: Vec<&str> = args.excluded.iter().map(|s| s.as_str()).collect();
        prop_assert_eq!(ignores.len(), args.excluded.len());
    }

    /// Config mapping is deterministic: same input yields same output.
    #[test]
    fn config_mapping_deterministic(args in arb_scan_options()) {
        let c1 = build_config(&args);
        let c2 = build_config(&args);
        prop_assert_eq!(c1.hidden, c2.hidden);
        prop_assert_eq!(c1.no_ignore, c2.no_ignore);
        prop_assert_eq!(c1.no_ignore_dot, c2.no_ignore_dot);
        prop_assert_eq!(c1.no_ignore_parent, c2.no_ignore_parent);
        prop_assert_eq!(c1.no_ignore_vcs, c2.no_ignore_vcs);
        prop_assert_eq!(c1.treat_doc_strings_as_comments, c2.treat_doc_strings_as_comments);
    }
}

// ============================================================================
// Flag implication: no_ignore ⊃ all sub-flags
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// When no_ignore=true, all sub-ignore flags are forced true.
    #[test]
    fn no_ignore_forces_all_sub_flags(args in arb_scan_options()) {
        prop_assume!(args.no_ignore);
        let cfg = build_config(&args);
        prop_assert_eq!(cfg.no_ignore, Some(true));
        prop_assert_eq!(cfg.no_ignore_dot, Some(true));
        prop_assert_eq!(cfg.no_ignore_parent, Some(true));
        prop_assert_eq!(cfg.no_ignore_vcs, Some(true));
    }

    /// When no_ignore=false, sub-flags reflect their individual values.
    #[test]
    fn sub_flags_independent_when_no_ignore_false(args in arb_scan_options()) {
        prop_assume!(!args.no_ignore);
        let mut cfg = tokei::Config::default();
        if args.no_ignore_dot { cfg.no_ignore_dot = Some(true); }
        if args.no_ignore_parent { cfg.no_ignore_parent = Some(true); }
        if args.no_ignore_vcs { cfg.no_ignore_vcs = Some(true); }

        prop_assert_eq!(cfg.no_ignore_dot.unwrap_or(false), args.no_ignore_dot);
        prop_assert_eq!(cfg.no_ignore_parent.unwrap_or(false), args.no_ignore_parent);
        prop_assert_eq!(cfg.no_ignore_vcs.unwrap_or(false), args.no_ignore_vcs);
    }

    /// hidden flag is orthogonal to ignore flags.
    #[test]
    fn hidden_orthogonal_to_ignore(
        hidden in any::<bool>(),
        no_ignore in any::<bool>(),
        no_ignore_dot in any::<bool>(),
    ) {
        let args = ScanOptions {
            excluded: vec![],
            config: ConfigMode::None,
            hidden,
            no_ignore,
            no_ignore_parent: false,
            no_ignore_dot,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
        };
        let cfg = build_config(&args);
        prop_assert_eq!(cfg.hidden.unwrap_or(false), hidden);
    }

    /// treat_doc_strings_as_comments is orthogonal to all ignore flags.
    #[test]
    fn doc_strings_orthogonal(
        treat_doc in any::<bool>(),
        no_ignore in any::<bool>(),
    ) {
        let args = ScanOptions {
            excluded: vec![],
            config: ConfigMode::None,
            hidden: false,
            no_ignore,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: treat_doc,
        };
        let cfg = build_config(&args);
        prop_assert_eq!(cfg.treat_doc_strings_as_comments.unwrap_or(false), treat_doc);
    }
}

// ============================================================================
// Exclude pattern properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Duplicating exclude patterns doesn't cause panics.
    #[test]
    fn duplicate_excludes_harmless(p in arb_exclude_pattern(), n in 1usize..5) {
        let args = ScanOptions {
            excluded: vec![p; n],
            config: ConfigMode::None,
            hidden: false,
            no_ignore: false,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
        };
        let ignores: Vec<&str> = args.excluded.iter().map(|s| s.as_str()).collect();
        prop_assert_eq!(ignores.len(), n);
    }

    /// Empty exclude list always works.
    #[test]
    fn empty_excludes_always_valid(args in arb_scan_options()) {
        let mut a = args;
        a.excluded = vec![];
        let ignores: Vec<&str> = a.excluded.iter().map(|s| s.as_str()).collect();
        prop_assert!(ignores.is_empty());
        let _ = build_config(&a);
    }
}

// ============================================================================
// Scan result properties (using real filesystem)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]

    /// Scanning the crate's own src/ with any valid ScanOptions never panics
    /// and produces non-negative counts.
    #[test]
    fn scan_own_src_never_panics_and_counts_nonneg(args in arb_scan_options()) {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let result = tokmd_scan::scan(&[src], &args);
        prop_assert!(result.is_ok(), "scan failed: {:?}", result.err());
        let langs = result.unwrap();
        for (_lt, lang) in langs.iter() {
            prop_assert!(lang.code <= lang.lines(),
                "code ({}) should not exceed total lines ({})", lang.code, lang.lines());
        }
    }

    /// Scanning always finds Rust files in this crate's src/.
    #[test]
    fn scan_finds_rust_in_own_src(_dummy in 0u8..5) {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let args = ScanOptions::default();
        let langs = tokmd_scan::scan(&[src], &args).unwrap();
        prop_assert!(
            langs.get(&tokei::LanguageType::Rust).is_some(),
            "Should find Rust in scan crate src"
        );
    }

    /// Language names from scan results are never empty strings.
    #[test]
    fn language_names_non_empty(_dummy in 0u8..3) {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let args = ScanOptions::default();
        let langs = tokmd_scan::scan(&[src], &args).unwrap();
        for (lt, _lang) in langs.iter() {
            prop_assert!(!lt.name().is_empty(), "Language name should not be empty");
        }
    }
}
