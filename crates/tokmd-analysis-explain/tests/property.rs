//! Property-based tests for tokmd-analysis-explain.

use proptest::prelude::*;
use tokmd_analysis_explain::{catalog, lookup};

// ── Property: lookup never panics on arbitrary input ─────────────────

proptest! {
    #[test]
    fn lookup_never_panics(key in ".*") {
        let _ = lookup(&key);
    }

    #[test]
    fn lookup_never_panics_on_ascii(key in "[a-zA-Z0-9_ \\-\\.]{0,200}") {
        let _ = lookup(&key);
    }

    #[test]
    fn lookup_never_panics_on_long_input(key in ".{500,2000}") {
        let _ = lookup(&key);
    }
}

// ── Property: normalize is idempotent (tested via lookup consistency) ─

const CANONICAL_KEYS: &[&str] = &[
    "doc_density",
    "whitespace_ratio",
    "verbosity",
    "test_density",
    "todo_density",
    "polyglot_entropy",
    "gini",
    "avg_cyclomatic",
    "max_cyclomatic",
    "avg_cognitive",
    "max_nesting_depth",
    "maintainability_index",
    "technical_debt_ratio",
    "halstead",
    "complexity_histogram",
    "hotspots",
    "bus_factor",
    "freshness",
    "code_age_distribution",
    "coupling",
    "predictive_churn",
    "duplicate_waste",
    "duplication_density",
    "imports",
    "entropy_suspects",
    "license_radar",
    "archetype",
    "context_window_fit",
];

fn random_separator_variant(key: &str) -> impl Strategy<Value = String> + '_ {
    // Replace underscores with a random separator (_, -, ., space)
    let parts: Vec<&str> = key.split('_').collect();
    let n = parts.len().saturating_sub(1).max(1);
    proptest::collection::vec(prop_oneof!["_", "-", ".", " "], n).prop_map(move |seps| {
        let mut result = String::new();
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                result.push_str(&seps[i - 1]);
            }
            result.push_str(part);
        }
        result
    })
}

proptest! {
    #[test]
    fn lookup_resolves_canonical_with_any_separator(
        idx in 0..CANONICAL_KEYS.len(),
        sep in prop_oneof![Just("_"), Just("-"), Just("."), Just(" ")],
    ) {
        let key = CANONICAL_KEYS[idx];
        let variant = key.replace('_', &sep);
        let result = lookup(&variant);
        prop_assert!(
            result.is_some(),
            "separator variant '{}' of '{}' should resolve",
            variant,
            key
        );
        let text = result.unwrap();
        prop_assert!(
            text.starts_with(&format!("{key}:")),
            "variant '{}' should resolve to canonical '{}'",
            variant,
            key
        );
    }

    #[test]
    fn lookup_resolves_canonical_with_random_case(
        idx in 0..CANONICAL_KEYS.len(),
        upper in proptest::bool::ANY,
    ) {
        let key = CANONICAL_KEYS[idx];
        let variant = if upper {
            key.to_uppercase()
        } else {
            key.to_lowercase()
        };
        let result = lookup(&variant);
        prop_assert!(
            result.is_some(),
            "case variant '{}' of '{}' should resolve",
            variant,
            key
        );
    }

    #[test]
    fn lookup_canonical_with_padding_resolves(
        idx in 0..CANONICAL_KEYS.len(),
        left_pad in "[ \\t]{0,5}",
        right_pad in "[ \\t]{0,5}",
    ) {
        let key = CANONICAL_KEYS[idx];
        let padded = format!("{left_pad}{key}{right_pad}");
        let result = lookup(&padded);
        prop_assert!(
            result.is_some(),
            "padded '{}' should resolve",
            padded.escape_debug()
        );
    }
}

// ── Property: lookup result format invariant ────────────────────────

proptest! {
    #[test]
    fn lookup_result_always_has_colon_space_format(idx in 0..CANONICAL_KEYS.len()) {
        let key = CANONICAL_KEYS[idx];
        let text = lookup(key).unwrap();
        let parts: Vec<&str> = text.splitn(2, ": ").collect();
        prop_assert_eq!(parts.len(), 2, "should have 'key: summary' format");
        prop_assert!(!parts[0].is_empty(), "key part should not be empty");
        prop_assert!(!parts[1].is_empty(), "summary part should not be empty");
        prop_assert!(parts[1].ends_with('.'), "summary should end with period");
    }
}

// ── Property: catalog is deterministic ──────────────────────────────

proptest! {
    #[test]
    fn catalog_is_always_identical(_seed in 0u32..1000) {
        let a = catalog();
        let b = catalog();
        prop_assert_eq!(a, b);
    }
}

// ── Property: lookup returns None for random gibberish ──────────────

proptest! {
    #[test]
    fn lookup_returns_none_for_numeric_strings(s in "[0-9]{1,20}") {
        prop_assert!(lookup(&s).is_none(), "numeric '{}' should not resolve", s);
    }

    #[test]
    fn lookup_returns_none_for_pure_separators(s in "[_ \\-\\.]{1,10}") {
        prop_assert!(
            lookup(&s).is_none(),
            "separator-only '{}' should not resolve",
            s
        );
    }
}
