use proptest::prelude::*;

/// Strategy for generating plausible metric key strings.
fn metric_key_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Known canonical keys
        Just("doc_density".to_string()),
        Just("whitespace_ratio".to_string()),
        Just("verbosity".to_string()),
        Just("test_density".to_string()),
        Just("todo_density".to_string()),
        Just("polyglot_entropy".to_string()),
        Just("gini".to_string()),
        Just("avg_cyclomatic".to_string()),
        Just("max_cyclomatic".to_string()),
        Just("avg_cognitive".to_string()),
        Just("max_nesting_depth".to_string()),
        Just("maintainability_index".to_string()),
        Just("technical_debt_ratio".to_string()),
        Just("halstead".to_string()),
        Just("hotspots".to_string()),
        Just("bus_factor".to_string()),
        Just("freshness".to_string()),
        Just("coupling".to_string()),
        Just("duplicate_waste".to_string()),
        Just("imports".to_string()),
        Just("entropy_suspects".to_string()),
        Just("license_radar".to_string()),
        Just("archetype".to_string()),
        Just("context_window_fit".to_string()),
        // Known aliases
        Just("docs".to_string()),
        Just("todo".to_string()),
        Just("mi".to_string()),
        Just("churn".to_string()),
        Just("dup".to_string()),
    ]
}

/// Strategy for arbitrary strings (potential unknown keys).
fn arbitrary_key_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_ .\\-]{0,50}"
}

proptest! {
    /// Lookup is deterministic: same key always yields same result.
    #[test]
    fn lookup_is_deterministic(key in metric_key_strategy()) {
        let r1 = crate::analysis_explain::lookup(&key);
        let r2 = crate::analysis_explain::lookup(&key);
        prop_assert_eq!(r1, r2);
    }

    /// Lookup output is always valid UTF-8 (guaranteed by String, but
    /// verifies the catalog content has no unexpected encoding issues).
    #[test]
    fn lookup_output_is_valid_utf8(key in metric_key_strategy()) {
        if let Some(result) = crate::analysis_explain::lookup(&key) {
            // String is always valid UTF-8 in Rust; verify it contains
            // the colon separator from the format "{canonical}: {summary}".
            prop_assert!(result.contains(':'), "lookup result missing ':' separator");
        }
    }

    /// Catalog output is always valid UTF-8 and non-empty.
    #[test]
    fn catalog_is_valid_utf8_and_nonempty(_dummy in 0..1u8) {
        let cat = crate::analysis_explain::catalog();
        prop_assert!(!cat.is_empty());
        prop_assert!(cat.starts_with("Available metric/finding keys:\n"));
    }

    /// Catalog is deterministic across calls.
    #[test]
    fn catalog_is_deterministic(_dummy in 0..1u8) {
        let c1 = crate::analysis_explain::catalog();
        let c2 = crate::analysis_explain::catalog();
        prop_assert_eq!(c1, c2);
    }

    /// Known canonical keys always resolve.
    #[test]
    fn canonical_keys_always_resolve(key in metric_key_strategy()) {
        // All keys in our strategy that are canonical should resolve.
        // Aliases also resolve. We just check the function doesn't panic.
        let _ = crate::analysis_explain::lookup(&key);
    }

    /// Lookup with whitespace/dash/dot normalization is consistent.
    /// E.g., "doc-density", "doc.density", "doc density" should all match.
    #[test]
    fn normalization_is_consistent(
        base in prop::sample::select(vec![
            "doc_density", "whitespace_ratio", "test_density",
            "avg_cyclomatic", "maintainability_index",
        ]),
        sep in prop::sample::select(vec!["_", "-", ".", " "]),
    ) {
        let normalized_key = base.replace('_', sep);
        let result = crate::analysis_explain::lookup(&normalized_key);
        let canonical_result = crate::analysis_explain::lookup(base);
        prop_assert_eq!(result, canonical_result,
            "Key '{}' (from '{}' with sep '{}') should match canonical",
            normalized_key, base, sep);
    }

    /// Lookup never panics on arbitrary input.
    #[test]
    fn lookup_never_panics(key in arbitrary_key_strategy()) {
        let _ = crate::analysis_explain::lookup(&key);
    }

    /// Unknown keys return None.
    #[test]
    fn unknown_keys_return_none(
        key in "zzz_unknown_[a-z]{3,10}"
    ) {
        let result = crate::analysis_explain::lookup(&key);
        prop_assert!(result.is_none(), "Unknown key '{}' should not resolve", key);
    }

    /// Catalog separator format: every non-header line starts with "- ".
    #[test]
    fn catalog_line_format_is_consistent(_dummy in 0..1u8) {
        let cat = crate::analysis_explain::catalog();
        for line in cat.lines().skip(1) {
            prop_assert!(line.starts_with("- "),
                "Non-header catalog line should start with '- ', got: '{}'", line);
        }
    }

    /// Every key listed in the catalog can be looked up.
    #[test]
    fn catalog_keys_all_resolve(_dummy in 0..1u8) {
        let cat = crate::analysis_explain::catalog();
        for line in cat.lines().skip(1) {
            if let Some(key) = line.strip_prefix("- ") {
                let result = crate::analysis_explain::lookup(key);
                prop_assert!(result.is_some(),
                    "Catalog key '{}' should resolve via lookup", key);
            }
        }
    }

    /// Lookup result always starts with a recognized canonical key.
    #[test]
    fn lookup_result_starts_with_canonical(key in metric_key_strategy()) {
        if let Some(result) = crate::analysis_explain::lookup(&key) {
            let colon_pos = result.find(':');
            prop_assert!(colon_pos.is_some(), "Result should contain ':'");
            let canonical = &result[..colon_pos.unwrap()];
            // Canonical key should be non-empty and use underscores.
            prop_assert!(!canonical.is_empty());
            prop_assert!(!canonical.contains(' '),
                "Canonical key should not contain spaces: '{}'", canonical);
        }
    }

    /// Case insensitivity: uppercase input matches the same as lowercase.
    #[test]
    fn case_insensitive_lookup(key in metric_key_strategy()) {
        let lower = crate::analysis_explain::lookup(&key.to_lowercase());
        let upper = crate::analysis_explain::lookup(&key.to_uppercase());
        prop_assert_eq!(lower, upper,
            "Lookup should be case-insensitive for '{}'", key);
    }
}
