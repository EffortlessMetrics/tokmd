//! W55 depth tests for tokmd-analysis-explain: lookup semantics,
//! catalog invariants, normalization boundary, and determinism.

use tokmd_analysis_explain::{catalog, lookup};

// ── lookup: semantic content per metric category ────────────────────

#[test]
fn lookup_whitespace_ratio_mentions_blank() {
    let text = lookup("whitespace_ratio").unwrap();
    assert!(
        text.to_lowercase().contains("blank"),
        "whitespace_ratio summary should mention blank lines"
    );
}

#[test]
fn lookup_test_density_mentions_test_files() {
    let text = lookup("test_density").unwrap();
    assert!(
        text.to_lowercase().contains("test"),
        "test_density should mention tests"
    );
}

#[test]
fn lookup_todo_density_mentions_markers() {
    let text = lookup("todo_density").unwrap();
    let lower = text.to_lowercase();
    assert!(
        lower.contains("todo") || lower.contains("fixme") || lower.contains("marker"),
        "todo_density should mention markers"
    );
}

#[test]
fn lookup_polyglot_entropy_mentions_language() {
    let text = lookup("polyglot_entropy").unwrap();
    let lower = text.to_lowercase();
    assert!(
        lower.contains("language") || lower.contains("distribution"),
        "polyglot_entropy should mention language distribution"
    );
}

#[test]
fn lookup_bus_factor_mentions_author() {
    let text = lookup("bus_factor").unwrap();
    let lower = text.to_lowercase();
    assert!(
        lower.contains("author") || lower.contains("ownership"),
        "bus_factor should mention authorship"
    );
}

#[test]
fn lookup_freshness_mentions_recency_or_stale() {
    let text = lookup("freshness").unwrap();
    let lower = text.to_lowercase();
    assert!(
        lower.contains("recen") || lower.contains("stale") || lower.contains("change"),
        "freshness should mention recency or staleness"
    );
}

#[test]
fn lookup_coupling_mentions_modules_or_commits() {
    let text = lookup("coupling").unwrap();
    let lower = text.to_lowercase();
    assert!(
        lower.contains("module") || lower.contains("commit") || lower.contains("changed"),
        "coupling should mention modules or commits"
    );
}

#[test]
fn lookup_predictive_churn_mentions_velocity() {
    let text = lookup("predictive_churn").unwrap();
    let lower = text.to_lowercase();
    assert!(
        lower.contains("velocity") || lower.contains("trend") || lower.contains("churn"),
        "predictive_churn should mention velocity or trend"
    );
}

#[test]
fn lookup_duplicate_waste_mentions_redundant() {
    let text = lookup("duplicate_waste").unwrap();
    let lower = text.to_lowercase();
    assert!(
        lower.contains("redundant") || lower.contains("duplicate"),
        "duplicate_waste should mention redundancy"
    );
}

#[test]
fn lookup_imports_mentions_dependency() {
    let text = lookup("imports").unwrap();
    let lower = text.to_lowercase();
    assert!(
        lower.contains("dependency") || lower.contains("import"),
        "imports should mention dependency or import"
    );
}

#[test]
fn lookup_entropy_suspects_mentions_entropy() {
    let text = lookup("entropy_suspects").unwrap();
    let lower = text.to_lowercase();
    assert!(
        lower.contains("entropy") || lower.contains("packed") || lower.contains("binary"),
        "entropy_suspects should mention entropy or packed"
    );
}

#[test]
fn lookup_license_radar_mentions_spdx_or_license() {
    let text = lookup("license_radar").unwrap();
    let lower = text.to_lowercase();
    assert!(
        lower.contains("spdx") || lower.contains("license"),
        "license_radar should mention SPDX or license"
    );
}

#[test]
fn lookup_archetype_mentions_repository_type() {
    let text = lookup("archetype").unwrap();
    let lower = text.to_lowercase();
    assert!(
        lower.contains("repository") || lower.contains("type") || lower.contains("inference"),
        "archetype should mention repository type"
    );
}

#[test]
fn lookup_context_window_fit_mentions_token() {
    let text = lookup("context_window_fit").unwrap();
    let lower = text.to_lowercase();
    assert!(
        lower.contains("token") || lower.contains("context") || lower.contains("window"),
        "context_window_fit should mention token or context"
    );
}

#[test]
fn lookup_maintainability_index_mentions_maintainability() {
    let text = lookup("maintainability_index").unwrap();
    let lower = text.to_lowercase();
    assert!(
        lower.contains("maintainability") || lower.contains("sei"),
        "maintainability_index should mention maintainability"
    );
}

#[test]
fn lookup_technical_debt_ratio_mentions_debt() {
    let text = lookup("technical_debt_ratio").unwrap();
    let lower = text.to_lowercase();
    assert!(
        lower.contains("debt") || lower.contains("complexity"),
        "technical_debt_ratio should mention debt"
    );
}

// ── lookup: separator and normalization edge cases ──────────────────

#[test]
fn lookup_dot_hyphen_space_all_normalize_identically() {
    let via_underscore = lookup("doc_density").unwrap();
    let via_dot = lookup("doc.density").unwrap();
    let via_hyphen = lookup("doc-density").unwrap();
    let via_space = lookup("doc density").unwrap();
    assert_eq!(via_underscore, via_dot);
    assert_eq!(via_underscore, via_hyphen);
    assert_eq!(via_underscore, via_space);
}

#[test]
fn lookup_mixed_separator_chains_resolve() {
    // e.g. "avg-cyclomatic" with hyphen
    assert!(lookup("avg-cyclomatic").is_some());
    assert!(lookup("avg.cyclomatic").is_some());
    assert!(lookup("avg cyclomatic").is_some());
}

#[test]
fn lookup_triple_word_keys_normalize_across_separators() {
    let canonical = lookup("code_age_distribution").unwrap();
    assert_eq!(lookup("code-age-distribution").unwrap(), canonical);
    assert_eq!(lookup("code.age.distribution").unwrap(), canonical);
    assert_eq!(lookup("code age distribution").unwrap(), canonical);
}

#[test]
fn lookup_case_insensitive_for_all_canonical_keys() {
    let keys = [
        "doc_density",
        "gini",
        "halstead",
        "archetype",
        "imports",
        "freshness",
    ];
    for key in keys {
        let lower = lookup(key).unwrap();
        let upper = lookup(&key.to_uppercase()).unwrap();
        assert_eq!(lower, upper, "case mismatch for {key}");
    }
}

#[test]
fn lookup_with_leading_and_trailing_spaces_resolves() {
    assert_eq!(
        lookup("  gini  ").unwrap(),
        lookup("gini").unwrap(),
        "trimming should normalize padded key"
    );
}

#[test]
fn lookup_with_interior_tab_does_not_resolve() {
    // Tab is not a separator replacement
    assert!(lookup("doc\tdensity").is_none());
}

// ── lookup: output format invariants ────────────────────────────────

#[test]
fn lookup_output_starts_with_canonical_key_for_all_entries() {
    let all = [
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
    for key in all {
        let text = lookup(key).unwrap();
        assert!(
            text.starts_with(&format!("{key}:")),
            "lookup({key}) should start with '{key}:', got: {text}"
        );
    }
}

#[test]
fn lookup_summary_length_is_reasonable_for_every_key() {
    let all = [
        "doc_density",
        "whitespace_ratio",
        "verbosity",
        "gini",
        "hotspots",
        "bus_factor",
        "coupling",
        "imports",
        "archetype",
    ];
    for key in all {
        let text = lookup(key).unwrap();
        let summary = text.split_once(": ").unwrap().1;
        assert!(
            summary.len() >= 10,
            "summary for '{key}' is too short: {summary}"
        );
        assert!(
            summary.len() <= 200,
            "summary for '{key}' is too long ({} chars)",
            summary.len()
        );
    }
}

#[test]
fn lookup_output_is_ascii_for_all_canonical_keys() {
    let all = ["doc_density", "gini", "halstead", "archetype", "imports"];
    for key in all {
        let text = lookup(key).unwrap();
        assert!(
            text.is_ascii(),
            "lookup({key}) output should be ASCII: {text}"
        );
    }
}

// ── catalog: determinism and stability ──────────────────────────────

#[test]
fn catalog_is_deterministic_across_100_calls() {
    let baseline = catalog();
    for i in 0..100 {
        assert_eq!(catalog(), baseline, "catalog() diverged on call {i}");
    }
}

#[test]
fn catalog_keys_are_strictly_alphabetical() {
    let text = catalog();
    let keys: Vec<&str> = text
        .lines()
        .skip(1)
        .filter_map(|l| l.strip_prefix("- "))
        .collect();
    for window in keys.windows(2) {
        assert!(
            window[0] < window[1],
            "catalog order violation: '{}' should come before '{}'",
            window[0],
            window[1]
        );
    }
}

#[test]
fn catalog_does_not_contain_duplicates() {
    let text = catalog();
    let keys: Vec<&str> = text
        .lines()
        .skip(1)
        .filter_map(|l| l.strip_prefix("- "))
        .collect();
    let set: std::collections::BTreeSet<&str> = keys.iter().copied().collect();
    assert_eq!(keys.len(), set.len(), "catalog has duplicate entries");
}

#[test]
fn catalog_every_line_after_header_starts_with_dash_space() {
    let text = catalog();
    for (i, line) in text.lines().enumerate().skip(1) {
        assert!(
            line.starts_with("- "),
            "catalog line {i} should start with '- ', got: {line}"
        );
    }
}

#[test]
fn catalog_contains_no_blank_lines() {
    for (i, line) in catalog().lines().enumerate() {
        assert!(
            !line.is_empty(),
            "catalog should not have blank line at index {i}"
        );
    }
}

#[test]
fn catalog_ends_with_trailing_newline() {
    assert!(catalog().ends_with('\n'));
}

// ── Cross-cutting: alias ↔ canonical round-trip ─────────────────────

#[test]
fn alias_lookup_always_returns_canonical_key_prefix() {
    let pairs = [
        ("docs", "doc_density"),
        ("mi", "maintainability_index"),
        ("churn", "predictive_churn"),
        ("dup", "duplicate_waste"),
        ("entropy", "entropy_suspects"),
        ("license", "license_radar"),
        ("staleness", "freshness"),
        ("ownership", "bus_factor"),
    ];
    for (alias, canonical) in pairs {
        let text = lookup(alias).unwrap();
        assert!(
            text.starts_with(&format!("{canonical}:")),
            "alias '{alias}' should resolve to canonical '{canonical}', got: {text}"
        );
    }
}

#[test]
fn all_catalog_keys_round_trip_through_lookup() {
    let text = catalog();
    for line in text.lines().skip(1) {
        if let Some(key) = line.strip_prefix("- ") {
            let result = lookup(key);
            assert!(result.is_some(), "catalog key '{key}' must be lookupable");
            let explanation = result.unwrap();
            assert!(
                explanation.starts_with(&format!("{key}:")),
                "lookup of catalog key '{key}' should start with '{key}:'"
            );
        }
    }
}
