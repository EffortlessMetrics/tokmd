//! Deeper scenario tests for explanation/narration generation.
//!
//! Covers determinism, text generation consistency, and edge cases.

use tokmd_analysis_explain::{catalog, lookup};

// ===========================================================================
// Determinism: same input always produces same explanation text
// ===========================================================================

#[test]
fn determinism_lookup_returns_identical_text_every_time() {
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
        let a = lookup(key);
        let b = lookup(key);
        let c = lookup(key);
        assert_eq!(a, b, "lookup('{key}') should be deterministic (a vs b)");
        assert_eq!(b, c, "lookup('{key}') should be deterministic (b vs c)");
    }
}

#[test]
fn determinism_catalog_returns_identical_text_every_time() {
    let a = catalog();
    let b = catalog();
    let c = catalog();
    assert_eq!(a, b, "catalog() first vs second");
    assert_eq!(b, c, "catalog() second vs third");
}

#[test]
fn determinism_alias_and_canonical_produce_same_text() {
    let pairs = [
        ("documentation_density", "doc_density"),
        ("docs", "doc_density"),
        ("whitespace", "whitespace_ratio"),
        ("bytes_per_line", "verbosity"),
        ("tests", "test_density"),
        ("todo", "todo_density"),
        ("fixme", "todo_density"),
        ("language_entropy", "polyglot_entropy"),
        ("polyglot", "polyglot_entropy"),
        ("distribution_gini", "gini"),
        ("cyclomatic", "avg_cyclomatic"),
        ("cognitive", "avg_cognitive"),
        ("nesting_depth", "max_nesting_depth"),
        ("mi", "maintainability_index"),
        ("debt_ratio", "technical_debt_ratio"),
        ("technical_debt", "technical_debt_ratio"),
        ("halstead_volume", "halstead"),
        ("halstead_effort", "halstead"),
        ("histogram", "complexity_histogram"),
        ("git_hotspots", "hotspots"),
        ("ownership", "bus_factor"),
        ("staleness", "freshness"),
        ("code_age", "code_age_distribution"),
        ("age_buckets", "code_age_distribution"),
        ("module_coupling", "coupling"),
        ("churn", "predictive_churn"),
        ("dup", "duplicate_waste"),
        ("duplication", "duplicate_waste"),
        ("dup_density", "duplication_density"),
        ("import_graph", "imports"),
        ("entropy", "entropy_suspects"),
        ("license", "license_radar"),
        ("project_archetype", "archetype"),
        ("window_fit", "context_window_fit"),
        ("context_fit", "context_window_fit"),
    ];

    for (alias, canonical) in pairs {
        let via_alias = lookup(alias);
        let via_canonical = lookup(canonical);
        assert_eq!(
            via_alias, via_canonical,
            "lookup('{alias}') should equal lookup('{canonical}')"
        );
    }
}

// ===========================================================================
// Normalization: various input formats resolve correctly
// ===========================================================================

#[test]
fn normalization_case_variants_resolve() {
    let variants = [
        "doc_density",
        "DOC_DENSITY",
        "Doc_Density",
        "doc-density",
        "doc.density",
        "doc density",
        "  doc_density  ",
    ];
    let expected = lookup("doc_density");
    for v in variants {
        assert_eq!(
            lookup(v),
            expected,
            "lookup('{v}') should match canonical form"
        );
    }
}

#[test]
fn normalization_mixed_separators() {
    // Dash, dot, space, and underscore should all normalize to the same thing
    assert_eq!(lookup("avg-cyclomatic"), lookup("avg_cyclomatic"));
    assert_eq!(lookup("avg.cyclomatic"), lookup("avg_cyclomatic"));
    assert_eq!(lookup("avg cyclomatic"), lookup("avg_cyclomatic"));
}

#[test]
fn normalization_whitespace_trimmed() {
    assert_eq!(lookup("  halstead  "), lookup("halstead"));
    assert_eq!(lookup("\tgini\t"), lookup("gini"));
}

// ===========================================================================
// Explanation text format: "canonical: summary"
// ===========================================================================

#[test]
fn format_all_explanations_have_colon_separator() {
    let keys = [
        "doc_density",
        "gini",
        "halstead",
        "hotspots",
        "archetype",
        "context_window_fit",
        "bus_factor",
        "freshness",
        "coupling",
    ];
    for key in keys {
        let text = lookup(key).unwrap();
        let parts: Vec<&str> = text.splitn(2, ": ").collect();
        assert_eq!(
            parts.len(),
            2,
            "'{key}' should have 'canonical: summary' format"
        );
        assert_eq!(parts[0], key, "first part should be canonical key");
        assert!(!parts[1].is_empty(), "summary should not be empty");
    }
}

#[test]
fn format_all_explanations_end_with_period() {
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
        let text = lookup(key).unwrap();
        assert!(
            text.ends_with('.'),
            "explanation for '{key}' should end with period, got: {text}"
        );
    }
}

#[test]
fn format_no_leading_trailing_whitespace_in_summaries() {
    let keys = [
        "doc_density",
        "halstead",
        "hotspots",
        "archetype",
        "freshness",
    ];
    for key in keys {
        let text = lookup(key).unwrap();
        let parts: Vec<&str> = text.splitn(2, ": ").collect();
        let summary = parts[1];
        assert_eq!(
            summary,
            summary.trim(),
            "summary for '{key}' should have no leading/trailing whitespace"
        );
    }
}

// ===========================================================================
// Domain-specific content in explanations
// ===========================================================================

#[test]
fn content_doc_density_mentions_comment() {
    let text = lookup("doc_density").unwrap();
    assert!(
        text.contains("comment") || text.contains("documentation"),
        "doc_density should mention comment/documentation"
    );
}

#[test]
fn content_halstead_mentions_operators() {
    let text = lookup("halstead").unwrap();
    assert!(
        text.contains("operator") || text.contains("operand"),
        "halstead should mention operators/operands"
    );
}

#[test]
fn content_gini_mentions_inequality() {
    let text = lookup("gini").unwrap();
    assert!(
        text.contains("inequality")
            || text.contains("concentration")
            || text.contains("Inequality"),
        "gini should mention inequality or concentration"
    );
}

#[test]
fn content_entropy_mentions_entropy() {
    let text = lookup("entropy_suspects").unwrap();
    assert!(
        text.contains("entropy"),
        "entropy_suspects should mention entropy"
    );
}

#[test]
fn content_license_mentions_spdx_or_license() {
    let text = lookup("license_radar").unwrap();
    assert!(
        text.contains("license") || text.contains("SPDX"),
        "license_radar should mention license or SPDX"
    );
}

#[test]
fn content_freshness_mentions_recency_or_stale() {
    let text = lookup("freshness").unwrap();
    assert!(
        text.contains("Recency") || text.contains("stale") || text.contains("change"),
        "freshness should mention recency, stale, or change: {text}"
    );
}

// ===========================================================================
// Catalog generation
// ===========================================================================

#[test]
fn catalog_has_header_and_keys() {
    let text = catalog();
    assert!(text.starts_with("Available metric/finding keys:\n"));
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.len() > 1, "catalog should have header + keys");
}

#[test]
fn catalog_keys_are_sorted_and_unique() {
    let text = catalog();
    let keys: Vec<&str> = text
        .lines()
        .skip(1)
        .filter_map(|l| l.strip_prefix("- "))
        .collect();

    let mut sorted = keys.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(keys, sorted, "catalog keys should be sorted and unique");
}

#[test]
fn catalog_contains_all_canonical_keys() {
    let text = catalog();
    let expected = [
        "archetype",
        "avg_cognitive",
        "avg_cyclomatic",
        "bus_factor",
        "code_age_distribution",
        "complexity_histogram",
        "context_window_fit",
        "coupling",
        "doc_density",
        "duplicate_waste",
        "duplication_density",
        "entropy_suspects",
        "freshness",
        "gini",
        "halstead",
        "hotspots",
        "imports",
        "license_radar",
        "maintainability_index",
        "max_cyclomatic",
        "max_nesting_depth",
        "polyglot_entropy",
        "predictive_churn",
        "technical_debt_ratio",
        "test_density",
        "todo_density",
        "verbosity",
        "whitespace_ratio",
    ];
    for key in expected {
        assert!(
            text.contains(&format!("- {key}\n")),
            "catalog should contain '- {key}'"
        );
    }
}

#[test]
fn catalog_does_not_contain_aliases() {
    let text = catalog();
    let aliases = [
        "documentation_density",
        "docs",
        "whitespace",
        "tests",
        "todo",
        "fixme",
        "mi",
        "debt_ratio",
        "churn",
        "dup",
        "entropy",
        "license",
    ];
    for alias in aliases {
        assert!(
            !text.contains(&format!("- {alias}\n")),
            "catalog should not list alias '{alias}'"
        );
    }
}

// ===========================================================================
// Edge cases: unknown keys
// ===========================================================================

#[test]
fn edge_unknown_keys_return_none() {
    assert!(lookup("nonexistent_metric").is_none());
    assert!(lookup("").is_none());
    assert!(lookup("   ").is_none());
    assert!(lookup("12345").is_none());
    assert!(lookup("!@#$%").is_none());
    assert!(lookup("doc_density_extra").is_none());
    assert!(lookup("doc").is_none());
}

// ===========================================================================
// Every explanation is substantive (not trivially short)
// ===========================================================================

#[test]
fn all_explanations_are_substantive() {
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
        let text = lookup(key).unwrap_or_else(|| panic!("'{key}' should resolve"));
        assert!(
            text.len() > 15,
            "explanation for '{key}' should be substantive (>15 chars), got: {text}"
        );
    }
}

// ===========================================================================
// Catalog line format: every line after header starts with "- "
// ===========================================================================

#[test]
fn catalog_line_format_consistent() {
    let text = catalog();
    for line in text.lines().skip(1) {
        assert!(
            line.starts_with("- "),
            "line should start with '- ': {line}"
        );
        let key = line.strip_prefix("- ").unwrap();
        assert!(!key.is_empty(), "key should not be empty");
        assert!(!key.contains(' '), "key should not contain spaces: {key}");
    }
}
