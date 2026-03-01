// BDD-style scenario tests for tokmd-analysis-explain

use tokmd_analysis_explain::{catalog, lookup};

// ── Scenario: Lookup by canonical key ────────────────────────────────

#[test]
fn given_canonical_key_when_lookup_then_returns_explanation() {
    let result = lookup("doc_density");
    assert!(
        result.is_some(),
        "canonical key 'doc_density' should resolve"
    );
    let text = result.unwrap();
    assert!(
        text.starts_with("doc_density:"),
        "explanation should start with the canonical key"
    );
    assert!(
        text.contains("comment"),
        "doc_density explanation should mention comments"
    );
}

#[test]
fn given_every_canonical_key_when_lookup_then_all_resolve() {
    let canonical_keys = [
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
    for key in canonical_keys {
        let result = lookup(key);
        assert!(result.is_some(), "canonical key '{key}' should resolve");
        let text = result.unwrap();
        assert!(
            text.starts_with(&format!("{key}:")),
            "explanation for '{key}' should start with '{key}:'"
        );
    }
}

// ── Scenario: Lookup by alias ────────────────────────────────────────

#[test]
fn given_alias_when_lookup_then_returns_canonical_explanation() {
    let aliases_to_canonical = [
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
    for (alias, expected_canonical) in aliases_to_canonical {
        let result = lookup(alias);
        assert!(result.is_some(), "alias '{alias}' should resolve");
        let text = result.unwrap();
        assert!(
            text.starts_with(&format!("{expected_canonical}:")),
            "alias '{alias}' should resolve to canonical '{expected_canonical}', got: {text}"
        );
    }
}

// ── Scenario: Normalization variants ─────────────────────────────────

#[test]
fn given_uppercase_key_when_lookup_then_normalizes_and_resolves() {
    assert!(lookup("DOC_DENSITY").is_some());
    assert!(lookup("Doc_Density").is_some());
    assert!(lookup("HALSTEAD").is_some());
}

#[test]
fn given_hyphenated_key_when_lookup_then_normalizes_and_resolves() {
    assert!(lookup("doc-density").is_some());
    assert!(lookup("whitespace-ratio").is_some());
    assert!(lookup("avg-cyclomatic").is_some());
}

#[test]
fn given_dotted_key_when_lookup_then_normalizes_and_resolves() {
    assert!(lookup("doc.density").is_some());
    assert!(lookup("whitespace.ratio").is_some());
}

#[test]
fn given_spaced_key_when_lookup_then_normalizes_and_resolves() {
    assert!(lookup("doc density").is_some());
    assert!(lookup("whitespace ratio").is_some());
}

#[test]
fn given_leading_trailing_whitespace_when_lookup_then_trims_and_resolves() {
    assert!(lookup("  doc_density  ").is_some());
    assert!(lookup("\thalstead\t").is_some());
}

#[test]
fn given_mixed_separators_when_lookup_then_normalizes_and_resolves() {
    assert!(lookup("Doc-Density").is_some());
    assert!(lookup("DOC.DENSITY").is_some());
    assert!(lookup("doc density").is_some());
}

// ── Scenario: Unknown key ────────────────────────────────────────────

#[test]
fn given_unknown_key_when_lookup_then_returns_none() {
    assert!(lookup("nonexistent_metric").is_none());
    assert!(lookup("").is_none());
    assert!(lookup("   ").is_none());
    assert!(lookup("__").is_none());
    assert!(lookup("doc_density_extra").is_none());
}

// ── Scenario: Catalog completeness ───────────────────────────────────

#[test]
fn given_catalog_when_called_then_contains_header() {
    let text = catalog();
    assert!(
        text.starts_with("Available metric/finding keys:\n"),
        "catalog should start with header line"
    );
}

#[test]
fn given_catalog_when_called_then_contains_all_canonical_keys() {
    let text = catalog();
    let expected = [
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
    for key in expected {
        assert!(
            text.contains(&format!("- {key}\n")),
            "catalog should contain '- {key}'"
        );
    }
}

#[test]
fn given_catalog_when_called_then_keys_are_sorted() {
    let text = catalog();
    let keys: Vec<&str> = text
        .lines()
        .skip(1)
        .filter_map(|l| l.strip_prefix("- "))
        .collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(keys, sorted, "catalog keys must be in sorted order");
}

#[test]
fn given_catalog_when_called_then_no_duplicate_keys() {
    let text = catalog();
    let keys: Vec<&str> = text
        .lines()
        .skip(1)
        .filter_map(|l| l.strip_prefix("- "))
        .collect();
    let unique: std::collections::BTreeSet<&str> = keys.iter().copied().collect();
    assert_eq!(
        keys.len(),
        unique.len(),
        "catalog should not contain duplicate keys"
    );
}

#[test]
fn given_catalog_when_called_then_does_not_list_aliases() {
    let text = catalog();
    let aliases = [
        "documentation_density",
        "docs",
        "whitespace",
        "bytes_per_line",
        "tests",
        "todo",
        "fixme",
        "language_entropy",
        "cyclomatic",
        "cognitive",
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
            "catalog should not list alias '{alias}' as a top-level key"
        );
    }
}

// ── Scenario: Explanation format consistency ─────────────────────────

#[test]
fn given_any_lookup_when_resolved_then_format_is_canonical_colon_space_summary() {
    let canonical_keys = [
        "doc_density",
        "gini",
        "hotspots",
        "archetype",
        "context_window_fit",
    ];
    for key in canonical_keys {
        let text = lookup(key).unwrap();
        let parts: Vec<&str> = text.splitn(2, ": ").collect();
        assert_eq!(
            parts.len(),
            2,
            "explanation for '{key}' should have format 'canonical: summary'"
        );
        assert_eq!(parts[0], key, "first part should be canonical key");
        assert!(
            !parts[1].is_empty(),
            "summary for '{key}' should not be empty"
        );
    }
}

#[test]
fn given_alias_lookup_when_resolved_then_uses_canonical_prefix() {
    // When looking up via alias, the output key should be the canonical name
    let text = lookup("mi").unwrap();
    assert!(text.starts_with("maintainability_index:"));

    let text = lookup("churn").unwrap();
    assert!(text.starts_with("predictive_churn:"));
}

// ── Scenario: Valid input produces non-empty explanation ─────────────

#[test]
fn given_any_canonical_key_when_lookup_then_explanation_is_non_empty() {
    let canonical_keys = [
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
    for key in canonical_keys {
        let text = lookup(key).unwrap_or_else(|| panic!("'{key}' should resolve"));
        assert!(
            text.len() > 10,
            "explanation for '{key}' should be substantive, got: {text}"
        );
    }
}

// ── Scenario: Explanations contain expected domain keywords ──────────

#[test]
fn given_doc_density_when_lookup_then_mentions_comment_or_documentation() {
    let text = lookup("doc_density").unwrap();
    assert!(
        text.contains("comment") || text.contains("documentation"),
        "doc_density should mention comments or documentation: {text}"
    );
}

#[test]
fn given_test_density_when_lookup_then_mentions_test() {
    let text = lookup("test_density").unwrap();
    assert!(
        text.contains("test"),
        "test_density should mention tests: {text}"
    );
}

#[test]
fn given_todo_density_when_lookup_then_mentions_todo_or_fixme() {
    let text = lookup("todo_density").unwrap();
    assert!(
        text.contains("TODO") || text.contains("FIXME"),
        "todo_density should mention TODO or FIXME: {text}"
    );
}

#[test]
fn given_cyclomatic_when_lookup_then_mentions_complexity() {
    let text = lookup("avg_cyclomatic").unwrap();
    assert!(
        text.contains("complexity") || text.contains("branching"),
        "avg_cyclomatic should mention complexity or branching: {text}"
    );
}

#[test]
fn given_halstead_when_lookup_then_mentions_operators_or_operands() {
    let text = lookup("halstead").unwrap();
    assert!(
        text.contains("operator") || text.contains("operand"),
        "halstead should mention operators or operands: {text}"
    );
}

#[test]
fn given_hotspots_when_lookup_then_mentions_change_or_frequency() {
    let text = lookup("hotspots").unwrap();
    assert!(
        text.contains("change") || text.contains("frequency"),
        "hotspots should mention change or frequency: {text}"
    );
}

#[test]
fn given_license_radar_when_lookup_then_mentions_license() {
    let text = lookup("license_radar").unwrap();
    assert!(
        text.contains("license") || text.contains("SPDX"),
        "license_radar should mention license or SPDX: {text}"
    );
}

#[test]
fn given_entropy_suspects_when_lookup_then_mentions_entropy() {
    let text = lookup("entropy_suspects").unwrap();
    assert!(
        text.contains("entropy"),
        "entropy_suspects should mention entropy: {text}"
    );
}

// ── Scenario: Empty/graceful handling ────────────────────────────────

#[test]
fn given_empty_string_when_lookup_then_returns_none() {
    assert!(lookup("").is_none());
}

#[test]
fn given_whitespace_only_when_lookup_then_returns_none() {
    assert!(lookup("   ").is_none());
    assert!(lookup("\t").is_none());
    assert!(lookup("\n").is_none());
}

#[test]
fn given_nonsense_key_when_lookup_then_returns_none_gracefully() {
    assert!(lookup("zzz_not_a_metric").is_none());
    assert!(lookup("12345").is_none());
    assert!(lookup("!@#$%").is_none());
}

// ── Scenario: Deterministic output ───────────────────────────────────

#[test]
fn given_same_key_when_lookup_called_twice_then_identical_output() {
    let keys = [
        "doc_density",
        "halstead",
        "hotspots",
        "archetype",
        "context_window_fit",
    ];
    for key in keys {
        let a = lookup(key);
        let b = lookup(key);
        assert_eq!(a, b, "lookup('{key}') should be deterministic");
    }
}

#[test]
fn given_alias_and_canonical_when_lookup_then_identical_output() {
    let pairs = [
        ("mi", "maintainability_index"),
        ("churn", "predictive_churn"),
        ("entropy", "entropy_suspects"),
        ("license", "license_radar"),
        ("dup", "duplicate_waste"),
        ("cognitive", "avg_cognitive"),
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

#[test]
fn given_catalog_when_called_twice_then_identical_output() {
    let a = catalog();
    let b = catalog();
    assert_eq!(a, b, "catalog() should be deterministic");
}

// ── Scenario: All sections properly structured ───────────────────────

#[test]
fn given_every_explanation_when_inspected_then_summary_ends_with_period() {
    let canonical_keys = [
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
    for key in canonical_keys {
        let text = lookup(key).unwrap();
        assert!(
            text.ends_with('.'),
            "explanation for '{key}' should end with a period, got: {text}"
        );
    }
}

#[test]
fn given_every_explanation_when_inspected_then_no_leading_or_trailing_whitespace_in_summary() {
    let canonical_keys = [
        "doc_density",
        "whitespace_ratio",
        "verbosity",
        "gini",
        "halstead",
        "hotspots",
        "archetype",
    ];
    for key in canonical_keys {
        let text = lookup(key).unwrap();
        let parts: Vec<&str> = text.splitn(2, ": ").collect();
        assert_eq!(parts.len(), 2);
        let summary = parts[1];
        assert_eq!(
            summary,
            summary.trim(),
            "summary for '{key}' should have no leading/trailing whitespace"
        );
    }
}

#[test]
fn given_catalog_when_inspected_then_every_line_after_header_is_a_dash_prefixed_key() {
    let text = catalog();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.len() > 1, "catalog should have header + key lines");
    for line in &lines[1..] {
        assert!(
            line.starts_with("- "),
            "catalog line should start with '- ', got: {line}"
        );
        let key = line.strip_prefix("- ").unwrap();
        assert!(
            !key.is_empty(),
            "catalog line should have a non-empty key after '- '"
        );
        assert!(
            !key.contains(' '),
            "catalog key should not contain spaces: {key}"
        );
    }
}
