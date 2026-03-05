//! Wave 72 property-based invariant tests for tokmd-scan.
//!
//! Covers: scan idempotency, exclusion patterns, ScanOptions construction
//! robustness, config-mode equivalence, and path-pattern handling.

use proptest::prelude::*;
use proptest::string::string_regex;
use std::path::PathBuf;
use tokmd_scan::scan;
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
    ]
}

fn default_scan_options() -> ScanOptions {
    ScanOptions {
        excluded: vec![],
        config: ConfigMode::None,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    }
}

fn test_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    // ========================================================================
    // 1. Scan of same directory is idempotent
    // ========================================================================

    #[test]
    fn scan_idempotent_code_counts(_seed in 0u32..100) {
        let args = default_scan_options();
        let paths = vec![test_path()];
        let r1 = scan(&paths, &args).unwrap();
        let r2 = scan(&paths, &args).unwrap();
        // Same directory with same options → same language set
        let langs1: Vec<_> = r1.keys().map(|lt| lt.name()).collect();
        let langs2: Vec<_> = r2.keys().map(|lt| lt.name()).collect();
        prop_assert_eq!(langs1, langs2);
    }

    #[test]
    fn scan_idempotent_line_counts(_seed in 0u32..100) {
        let args = default_scan_options();
        let paths = vec![test_path()];
        let r1 = scan(&paths, &args).unwrap();
        let r2 = scan(&paths, &args).unwrap();
        for (lt, lang1) in r1.iter() {
            let lang2 = r2.get(lt).unwrap();
            prop_assert_eq!(lang1.code, lang2.code);
            prop_assert_eq!(lang1.comments, lang2.comments);
            prop_assert_eq!(lang1.blanks, lang2.blanks);
        }
    }

    // ========================================================================
    // 2. Excluding all files produces zero or fewer counts
    // ========================================================================

    #[test]
    fn excluding_everything_produces_empty(_seed in 0u32..20) {
        let args = ScanOptions {
            excluded: vec!["**/*".to_string()],
            config: ConfigMode::None,
            hidden: false,
            no_ignore: false,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
        };
        let paths = vec![test_path()];
        let result = scan(&paths, &args).unwrap();
        let total_code: usize = result.values().map(|l| l.code).sum();
        prop_assert_eq!(total_code, 0, "Excluding **/* should yield 0 code lines");
    }

    // ========================================================================
    // 3. Adding exclude patterns never increases code counts
    // ========================================================================

    #[test]
    fn more_excludes_never_increase_code(
        extra_excludes in prop::collection::vec(arb_exclude_pattern(), 1..4),
    ) {
        let base_args = default_scan_options();
        let paths = vec![test_path()];
        let base_result = scan(&paths, &base_args).unwrap();
        let base_code: usize = base_result.values().map(|l| l.code).sum();

        let restricted_args = ScanOptions {
            excluded: extra_excludes,
            ..default_scan_options()
        };
        let restricted_result = scan(&paths, &restricted_args).unwrap();
        let restricted_code: usize = restricted_result.values().map(|l| l.code).sum();

        prop_assert!(
            restricted_code <= base_code,
            "With excludes: {} > base: {}", restricted_code, base_code
        );
    }

    // ========================================================================
    // 4. ScanOptions construction is always valid
    // ========================================================================

    #[test]
    fn arbitrary_scan_options_do_not_panic(
        excludes in prop::collection::vec(arb_exclude_pattern(), 0..5),
        hidden in any::<bool>(),
        no_ignore in any::<bool>(),
        no_ignore_parent in any::<bool>(),
        no_ignore_dot in any::<bool>(),
        no_ignore_vcs in any::<bool>(),
        treat_doc in any::<bool>(),
    ) {
        let args = ScanOptions {
            excluded: excludes,
            config: ConfigMode::None,
            hidden,
            no_ignore,
            no_ignore_parent,
            no_ignore_dot,
            no_ignore_vcs,
            treat_doc_strings_as_comments: treat_doc,
        };
        let paths = vec![test_path()];
        // Should never panic regardless of flag combination
        let result = scan(&paths, &args);
        prop_assert!(result.is_ok());
    }

    // ========================================================================
    // 5. Scan always finds Rust in its own src/
    // ========================================================================

    #[test]
    fn scan_always_finds_rust(_seed in 0u32..50) {
        let args = default_scan_options();
        let paths = vec![test_path()];
        let result = scan(&paths, &args).unwrap();
        let has_rust = result.get(&tokei::LanguageType::Rust).is_some();
        prop_assert!(has_rust, "Should always find Rust in crate src/");
    }

    // ========================================================================
    // 6. Code lines are always ≤ total lines
    // ========================================================================

    #[test]
    fn code_leq_total_lines(_seed in 0u32..50) {
        let args = default_scan_options();
        let paths = vec![test_path()];
        let result = scan(&paths, &args).unwrap();
        for (lt, lang) in result.iter() {
            let total = lang.code + lang.comments + lang.blanks;
            prop_assert!(
                lang.code <= total,
                "{}: code {} > total {}", lt.name(), lang.code, total
            );
        }
    }

    // ========================================================================
    // 7. Nonexistent path always errors
    // ========================================================================

    #[test]
    fn nonexistent_path_errors(suffix in "[a-z]{5,15}") {
        let dir = tempfile::tempdir().unwrap();
        let bad = dir.path().join(suffix);
        let args = default_scan_options();
        let result = scan(&[bad], &args);
        prop_assert!(result.is_err());
    }

    // ========================================================================
    // 8. Hidden flag doesn't reduce visible file counts
    // ========================================================================

    #[test]
    fn hidden_flag_does_not_reduce_counts(_seed in 0u32..20) {
        let base = default_scan_options();
        let hidden = ScanOptions { hidden: true, ..default_scan_options() };
        let paths = vec![test_path()];
        let base_code: usize = scan(&paths, &base).unwrap().values().map(|l| l.code).sum();
        let hidden_code: usize = scan(&paths, &hidden).unwrap().values().map(|l| l.code).sum();
        prop_assert!(hidden_code >= base_code, "hidden should include more, not less");
    }

    // ========================================================================
    // 9. Doc-string-as-comments flag doesn't change total lines
    // ========================================================================

    #[test]
    fn doc_strings_flag_preserves_total_lines(_seed in 0u32..20) {
        let base = default_scan_options();
        let doc = ScanOptions { treat_doc_strings_as_comments: true, ..default_scan_options() };
        let paths = vec![test_path()];
        let base_total: usize = scan(&paths, &base).unwrap()
            .values().map(|l| l.code + l.comments + l.blanks).sum();
        let doc_total: usize = scan(&paths, &doc).unwrap()
            .values().map(|l| l.code + l.comments + l.blanks).sum();
        prop_assert_eq!(base_total, doc_total, "Total lines should be unchanged");
    }
}
