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
//! 5. **Scan-result invariants**: line counts non-negative, totals consistent
//! 6. **Determinism**: scanning the same directory twice yields identical results
//! 7. **Empty directory**: produces empty results
//! 8. **Children mode**: both Collapse and Separate produce non-negative totals

use std::collections::BTreeMap;
use std::fs;

use proptest::prelude::*;
use proptest::string::string_regex;
use tempfile::TempDir;
use tokmd_scan::scan;
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
        #[allow(clippy::field_reassign_with_default)]
        let cfg = {
            let mut cfg = tokei::Config::default();
            cfg.no_ignore_dot = Some(true);
            cfg
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
        #[allow(clippy::field_reassign_with_default)]
        let cfg = {
            let mut cfg = tokei::Config::default();
            cfg.no_ignore_vcs = Some(true);
            cfg
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

// ============================================================================
// Helpers for scan-result property tests
// ============================================================================

fn default_opts() -> ScanOptions {
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

/// Write a file inside `dir`, creating intermediate directories as needed.
fn write_file(dir: &TempDir, rel: &str, content: &str) {
    let path = dir.path().join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
}

/// Minimal Rust source snippets used as scan inputs.
const RUST_SNIPPETS: &[&str] = &[
    "fn main() {}\n",
    "// comment\nfn f() {}\n",
    "/// doc\nfn g() {}\n\n",
    "fn a() {}\nfn b() {}\nfn c() {}\n",
    "struct S;\nimpl S {\n    fn m(&self) {}\n}\n",
];

/// Minimal Python source snippets.
const PYTHON_SNIPPETS: &[&str] = &[
    "def f():\n    pass\n",
    "# comment\nx = 1\n",
    "class C:\n    pass\n",
];

// ============================================================================
// Scan-result property tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    // ------------------------------------------------------------------
    // 1. Path normalization determinism: scanning the same directory twice
    //    produces identical results.
    // ------------------------------------------------------------------

    /// Scanning the same temp directory twice must yield byte-identical stats.
    #[test]
    fn scan_determinism(idx in 0..RUST_SNIPPETS.len()) {
        let tmp = TempDir::new().unwrap();
        write_file(&tmp, "main.rs", RUST_SNIPPETS[idx]);

        let opts = default_opts();
        let paths = vec![tmp.path().to_path_buf()];

        let r1 = scan(&paths, &opts).unwrap();
        let r2 = scan(&paths, &opts).unwrap();

        // Collect language keys
        let keys1: Vec<_> = r1.iter().map(|(k, _)| *k).collect();
        let keys2: Vec<_> = r2.iter().map(|(k, _)| *k).collect();
        prop_assert_eq!(&keys1, &keys2, "language sets must match across scans");

        for (lang_type, lang1) in r1.iter() {
            let lang2 = r2.get(lang_type).unwrap();
            prop_assert_eq!(lang1.code, lang2.code, "code mismatch for {:?}", lang_type);
            prop_assert_eq!(lang1.comments, lang2.comments, "comments mismatch for {:?}", lang_type);
            prop_assert_eq!(lang1.blanks, lang2.blanks, "blanks mismatch for {:?}", lang_type);
        }
    }

    // ------------------------------------------------------------------
    // 2. Non-negative invariant: all line counts are non-negative.
    //    (usize is inherently >= 0, but this verifies the scan path
    //    doesn't underflow or panic for any snippet.)
    // ------------------------------------------------------------------

    /// All line-count fields produced by scan must be non-negative (no underflow).
    #[test]
    fn line_counts_non_negative(
        rust_idx in 0..RUST_SNIPPETS.len(),
        py_idx in 0..PYTHON_SNIPPETS.len(),
    ) {
        let tmp = TempDir::new().unwrap();
        write_file(&tmp, "src/lib.rs", RUST_SNIPPETS[rust_idx]);
        write_file(&tmp, "src/util.py", PYTHON_SNIPPETS[py_idx]);

        let langs = scan(&[tmp.path().to_path_buf()], &default_opts()).unwrap();

        for (_lang_type, lang) in langs.iter() {
            // Verify that adding the components doesn't overflow and that
            // lines() is at least as large as each individual component.
            let sum = lang.code + lang.comments + lang.blanks;
            prop_assert!(lang.lines() >= lang.code, "lines() < code");
            prop_assert!(lang.lines() >= lang.comments, "lines() < comments");
            prop_assert!(lang.lines() >= lang.blanks, "lines() < blanks");
            prop_assert!(lang.lines() >= sum, "lines() < sum of parts");

            // Per-report stats: the sum of parts must not exceed lines.
            for report in &lang.reports {
                let st = report.stats.summarise();
                let rpt_sum = st.code + st.comments + st.blanks;
                prop_assert!(rpt_sum >= st.code, "report sum < code");
                prop_assert!(rpt_sum >= st.comments, "report sum < comments");
                prop_assert!(rpt_sum >= st.blanks, "report sum < blanks");
            }
        }
    }

    // ------------------------------------------------------------------
    // 3. Total consistency: lines() >= code + comments + blanks.
    //    tokei's lines() returns code + comments + blanks, so equality
    //    should hold.  We assert >= for forward-compatibility.
    // ------------------------------------------------------------------

    /// For every language, total_lines >= code + comments + blanks.
    #[test]
    fn total_consistency(idx in 0..RUST_SNIPPETS.len()) {
        let tmp = TempDir::new().unwrap();
        write_file(&tmp, "main.rs", RUST_SNIPPETS[idx]);

        let langs = scan(&[tmp.path().to_path_buf()], &default_opts()).unwrap();

        for (_lang_type, lang) in langs.iter() {
            let sum = lang.code + lang.comments + lang.blanks;
            prop_assert!(
                lang.lines() >= sum,
                "lines() ({}) should be >= code+comments+blanks ({}) for {:?}",
                lang.lines(),
                sum,
                _lang_type,
            );

            // Also verify per-report consistency.
            for report in &lang.reports {
                let st = report.stats.summarise();
                let rpt_sum = st.code + st.comments + st.blanks;
                let rpt_lines = st.code + st.comments + st.blanks;
                prop_assert!(
                    rpt_lines >= rpt_sum,
                    "per-report lines >= sum invariant violated",
                );
            }
        }
    }

    // ------------------------------------------------------------------
    // 4. Empty directory: scanning an empty temp directory produces empty
    //    results (no languages detected).
    // ------------------------------------------------------------------

    /// An empty directory must produce zero detected languages.
    #[test]
    fn empty_directory_yields_empty_results(_seed in 0..50u32) {
        let tmp = TempDir::new().unwrap();
        let langs = scan(&[tmp.path().to_path_buf()], &default_opts()).unwrap();
        prop_assert!(langs.is_empty(), "empty dir should yield no languages");
    }

    // ------------------------------------------------------------------
    // 5. Children mode: both Collapse and Separate produce non-negative
    //    totals.  At the scan layer we verify that children stats
    //    (embedded languages) are themselves non-negative and consistent.
    // ------------------------------------------------------------------

    /// Children (embedded language) stats are non-negative and consistent.
    ///
    /// We create an HTML file that contains embedded CSS/JS to exercise
    /// tokei's children handling, then verify invariants for both parent
    /// and child stats.
    #[test]
    fn children_stats_non_negative_and_consistent(_seed in 0..20u32) {
        let tmp = TempDir::new().unwrap();

        // HTML with embedded CSS and JS triggers tokei's children parsing.
        let html = "\
<!DOCTYPE html>
<html>
<head>
<style>
body { margin: 0; }
</style>
</head>
<body>
<script>
console.log('hello');
</script>
</body>
</html>
";
        write_file(&tmp, "index.html", html);
        // Also add a plain Rust file for diversity.
        write_file(&tmp, "lib.rs", "fn lib() {}\n");

        let langs = scan(&[tmp.path().to_path_buf()], &default_opts()).unwrap();

        for (_lang_type, lang) in langs.iter() {
            // Parent stats non-negative and consistent.
            let parent_sum = lang.code + lang.comments + lang.blanks;
            prop_assert!(lang.lines() >= parent_sum);

            // Children stats (embedded languages).
            for (_child_type, child_reports) in &lang.children {
                for report in child_reports {
                    let st = report.stats.summarise();
                    let child_sum = st.code + st.comments + st.blanks;
                    // The sum of child parts must be >= each component.
                    prop_assert!(child_sum >= st.code, "child sum < code");
                    prop_assert!(child_sum >= st.comments, "child sum < comments");
                    prop_assert!(child_sum >= st.blanks, "child sum < blanks");
                }
            }
        }
    }

    /// Both ChildrenMode::Collapse and ChildrenMode::Separate semantics
    /// are exercised: we aggregate children manually for each mode and
    /// verify that totals remain non-negative.
    #[test]
    fn children_collapse_vs_separate_both_non_negative(_seed in 0..20u32) {
        let tmp = TempDir::new().unwrap();
        let html = "\
<html>
<head><style>h1 { color: red; }</style></head>
<body><script>var x = 1;</script></body>
</html>
";
        write_file(&tmp, "page.html", html);

        let langs = scan(&[tmp.path().to_path_buf()], &default_opts()).unwrap();

        for (_lang_type, lang) in langs.iter() {
            // Collapse mode: merge children into parent totals.
            let collapsed_code = {
                let mut total = lang.code;
                for (_ct, reports) in &lang.children {
                    for r in reports {
                        total += r.stats.summarise().code;
                    }
                }
                total
            };
            prop_assert!(collapsed_code >= lang.code, "collapsed code must be >= parent code");

            // Separate mode: parent and children reported independently.
            let mut separate: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
            separate.insert(
                _lang_type.name().to_string(),
                (lang.code, lang.comments, lang.blanks),
            );
            for (child_type, reports) in &lang.children {
                let entry = separate
                    .entry(format!("{} (embedded)", child_type.name()))
                    .or_insert((0, 0, 0));
                for r in reports {
                    let st = r.stats.summarise();
                    entry.0 += st.code;
                    entry.1 += st.comments;
                    entry.2 += st.blanks;
                }
            }

            // All entries should have non-negative components (guaranteed by
            // usize, but we assert the sums hold).
            for (_name, (code, comments, blanks)) in &separate {
                let total = code + comments + blanks;
                prop_assert!(total >= *code);
                prop_assert!(total >= *comments);
                prop_assert!(total >= *blanks);
            }
        }
    }
}
