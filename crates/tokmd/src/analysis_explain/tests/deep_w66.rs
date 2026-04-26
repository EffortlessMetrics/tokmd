//! W66 deep tests for `analysis_explain module`.
//!
//! Exercises lookup normalization, catalog structure, edge cases for
//! missing/empty data, alias consistency, and determinism.

use crate::analysis_explain::{catalog, lookup};

// ── Lookup: every canonical key resolves ────────────────────────

mod canonical_keys_w66 {
    use super::*;

    #[test]
    fn all_known_canonical_keys_resolve() {
        let keys = [
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
        for key in keys {
            assert!(
                lookup(key).is_some(),
                "canonical key '{key}' should resolve"
            );
        }
    }

    #[test]
    fn lookup_returns_canonical_name_first() {
        let result = lookup("hotspots").unwrap();
        assert!(result.starts_with("hotspots:"));
    }

    #[test]
    fn lookup_result_contains_summary_text() {
        let result = lookup("bus_factor").unwrap();
        let (_, summary) = result.split_once(": ").unwrap();
        assert!(!summary.is_empty());
        assert!(summary.len() > 10);
    }
}

// ── Lookup: normalization edge cases ────────────────────────────

mod normalization_w66 {
    use super::*;

    #[test]
    fn dash_replaced_by_underscore() {
        assert_eq!(
            lookup("doc-density").unwrap(),
            lookup("doc_density").unwrap(),
        );
    }

    #[test]
    fn dot_replaced_by_underscore() {
        assert_eq!(
            lookup("doc.density").unwrap(),
            lookup("doc_density").unwrap(),
        );
    }

    #[test]
    fn space_replaced_by_underscore() {
        assert_eq!(
            lookup("doc density").unwrap(),
            lookup("doc_density").unwrap(),
        );
    }

    #[test]
    fn mixed_case_normalized() {
        assert_eq!(
            lookup("DOC_DENSITY").unwrap(),
            lookup("doc_density").unwrap(),
        );
    }

    #[test]
    fn leading_trailing_whitespace_trimmed() {
        assert_eq!(
            lookup("  doc_density  ").unwrap(),
            lookup("doc_density").unwrap(),
        );
    }

    #[test]
    fn empty_string_returns_none() {
        assert!(lookup("").is_none());
    }

    #[test]
    fn whitespace_only_returns_none() {
        assert!(lookup("   ").is_none());
    }
}

// ── Lookup: alias completeness ──────────────────────────────────

mod aliases_w66 {
    use super::*;

    #[test]
    fn documentation_density_alias() {
        let canonical = lookup("doc_density").unwrap();
        assert_eq!(lookup("documentation_density").unwrap(), canonical);
    }

    #[test]
    fn docs_alias() {
        let canonical = lookup("doc_density").unwrap();
        assert_eq!(lookup("docs").unwrap(), canonical);
    }

    #[test]
    fn bytes_per_line_alias_for_verbosity() {
        let canonical = lookup("verbosity").unwrap();
        assert_eq!(lookup("bytes_per_line").unwrap(), canonical);
    }

    #[test]
    fn churn_alias_for_predictive_churn() {
        let canonical = lookup("predictive_churn").unwrap();
        assert_eq!(lookup("churn").unwrap(), canonical);
    }

    #[test]
    fn mi_alias_for_maintainability_index() {
        let canonical = lookup("maintainability_index").unwrap();
        assert_eq!(lookup("mi").unwrap(), canonical);
    }

    #[test]
    fn dup_alias_for_duplicate_waste() {
        let canonical = lookup("duplicate_waste").unwrap();
        assert_eq!(lookup("dup").unwrap(), canonical);
    }

    #[test]
    fn entropy_alias_for_entropy_suspects() {
        let canonical = lookup("entropy_suspects").unwrap();
        assert_eq!(lookup("entropy").unwrap(), canonical);
    }
}

// ── Catalog: structure ──────────────────────────────────────────

mod catalog_w66 {
    use super::*;

    #[test]
    fn catalog_starts_with_header() {
        let text = catalog();
        assert!(text.starts_with("Available metric/finding keys:\n"));
    }

    #[test]
    fn catalog_keys_use_dash_prefix() {
        let text = catalog();
        for line in text.lines().skip(1) {
            assert!(line.starts_with("- "));
        }
    }

    #[test]
    fn catalog_keys_are_sorted() {
        let text = catalog();
        let keys: Vec<&str> = text
            .lines()
            .skip(1)
            .filter_map(|l| l.strip_prefix("- "))
            .collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted);
    }

    #[test]
    fn catalog_has_no_duplicates() {
        let text = catalog();
        let keys: Vec<&str> = text
            .lines()
            .skip(1)
            .filter_map(|l| l.strip_prefix("- "))
            .collect();
        let unique: std::collections::BTreeSet<&str> = keys.iter().copied().collect();
        assert_eq!(keys.len(), unique.len());
    }

    #[test]
    fn catalog_is_deterministic() {
        assert_eq!(catalog(), catalog());
    }

    #[test]
    fn catalog_every_key_resolvable() {
        let text = catalog();
        for line in text.lines().skip(1) {
            if let Some(key) = line.strip_prefix("- ") {
                assert!(
                    lookup(key).is_some(),
                    "catalog key '{key}' must be resolvable"
                );
            }
        }
    }
}

// ── Missing/invalid data ────────────────────────────────────────

mod missing_data_w66 {
    use super::*;

    #[test]
    fn nonexistent_key_returns_none() {
        assert!(lookup("nonexistent_metric_xyz").is_none());
    }

    #[test]
    fn numeric_key_returns_none() {
        assert!(lookup("12345").is_none());
    }

    #[test]
    fn special_chars_key_returns_none() {
        assert!(lookup("@#$%^&*").is_none());
    }

    #[test]
    fn very_long_key_returns_none() {
        let long = "a".repeat(50_000);
        assert!(lookup(&long).is_none());
    }
}
