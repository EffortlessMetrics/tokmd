//! Edge-case tests for analysis_explain module.

use crate::analysis_explain::{catalog, lookup};

// ── Null bytes and control characters ───────────────────────────────

#[test]
fn lookup_with_null_byte_returns_none() {
    assert!(lookup("doc\0density").is_none());
    assert!(lookup("\0").is_none());
}

#[test]
fn lookup_with_carriage_return_returns_none() {
    assert!(lookup("doc\rdensity").is_none());
}

#[test]
fn lookup_with_vertical_tab_returns_none() {
    assert!(lookup("doc\x0Bdensity").is_none());
}

#[test]
fn lookup_with_form_feed_returns_none() {
    assert!(lookup("doc\x0Cdensity").is_none());
}

// ── Prefix/suffix substrings of canonical keys ──────────────────────

#[test]
fn lookup_prefix_of_canonical_key_returns_none() {
    assert!(lookup("doc_").is_none());
    assert!(lookup("doc").is_none());
    assert!(lookup("whitespace_").is_none());
    assert!(lookup("avg_").is_none());
    assert!(lookup("max_").is_none());
    assert!(lookup("predict").is_none());
}

#[test]
fn lookup_suffix_of_canonical_key_returns_none() {
    assert!(lookup("density").is_none());
    assert!(lookup("ratio").is_none());
    assert!(lookup("factor").is_none());
    assert!(lookup("suspects").is_none());
    assert!(lookup("radar").is_none());
    assert!(lookup("depth").is_none());
}

#[test]
fn lookup_canonical_with_appended_chars_returns_none() {
    assert!(lookup("doc_density_").is_none());
    assert!(lookup("gini_extra").is_none());
    assert!(lookup("halsteadx").is_none());
    assert!(lookup("freshness2").is_none());
}

#[test]
fn lookup_canonical_with_prepended_chars_returns_none() {
    assert!(lookup("xdoc_density").is_none());
    assert!(lookup("_gini").is_none());
    assert!(lookup("0halstead").is_none());
}

// ── Emoji and non-ASCII ─────────────────────────────────────────────

#[test]
fn lookup_with_emoji_returns_none() {
    assert!(lookup("🔥").is_none());
    assert!(lookup("doc_density🔥").is_none());
    assert!(lookup("🔥doc_density").is_none());
}

#[test]
fn lookup_with_cjk_characters_returns_none() {
    assert!(lookup("文档密度").is_none());
}

#[test]
fn lookup_with_accented_chars_returns_none() {
    assert!(lookup("dóc_dénsity").is_none());
    assert!(lookup("naïve").is_none());
}

// ── Numeric and special inputs ──────────────────────────────────────

#[test]
fn lookup_purely_numeric_returns_none() {
    assert!(lookup("12345").is_none());
    assert!(lookup("0").is_none());
}

#[test]
fn lookup_with_special_regex_chars_returns_none() {
    assert!(lookup("doc.*density").is_none());
    assert!(lookup("doc|density").is_none());
    assert!(lookup("doc(density)").is_none());
    assert!(lookup("[doc_density]").is_none());
}

#[test]
fn lookup_with_path_separators_returns_none() {
    assert!(lookup("doc/density").is_none());
    assert!(lookup("doc\\density").is_none());
}

// ── Empty and whitespace-only ───────────────────────────────────────

#[test]
fn lookup_empty_string_returns_none() {
    assert!(lookup("").is_none());
}

#[test]
fn lookup_various_whitespace_only_returns_none() {
    assert!(lookup(" ").is_none());
    assert!(lookup("  ").is_none());
    assert!(lookup("\t").is_none());
    assert!(lookup("\n").is_none());
    assert!(lookup(" \t \n ").is_none());
}

// ── Large input stress ──────────────────────────────────────────────

#[test]
fn lookup_extremely_long_input_returns_none() {
    let long = "a".repeat(100_000);
    assert!(lookup(&long).is_none());
}

#[test]
fn lookup_long_valid_key_padded_returns_none() {
    let padded = format!("{}doc_density{}", "x".repeat(1000), "y".repeat(1000));
    assert!(lookup(&padded).is_none());
}

// ── Catalog structural edge cases ───────────────────────────────────

#[test]
fn catalog_ends_with_newline() {
    let text = catalog();
    assert!(
        text.ends_with('\n'),
        "catalog should end with trailing newline"
    );
}

#[test]
fn catalog_does_not_contain_alias_names() {
    let text = catalog();
    // Exhaustive check: aliases that are NOT also canonical keys
    let alias_only = [
        "documentation_density",
        "docs",
        "whitespace",
        "bytes_per_line",
        "tests",
        "todo",
        "fixme",
        "language_entropy",
        "polyglot",
        "distribution_gini",
        "cyclomatic",
        "cognitive",
        "nesting_depth",
        "mi",
        "debt_ratio",
        "technical_debt",
        "halstead_volume",
        "halstead_effort",
        "histogram",
        "git_hotspots",
        "ownership",
        "staleness",
        "code_age",
        "age_buckets",
        "module_coupling",
        "churn",
        "dup",
        "duplication",
        "dup_density",
        "import_graph",
        "entropy",
        "license",
        "project_archetype",
        "window_fit",
        "context_fit",
    ];
    for alias in alias_only {
        assert!(
            !text.contains(&format!("- {alias}\n")),
            "catalog must not list alias '{alias}'"
        );
    }
}

#[test]
fn catalog_line_format_is_consistent() {
    let text = catalog();
    for (i, line) in text.lines().enumerate() {
        if i == 0 {
            assert!(line.starts_with("Available"), "first line should be header");
        } else {
            assert!(
                line.starts_with("- "),
                "line {i} should start with '- ', got: {line}"
            );
        }
    }
}

// ── Cross-cutting: alias uniqueness ─────────────────────────────────

#[test]
fn no_alias_resolves_to_different_canonical_keys() {
    // All aliases for each canonical key should resolve to the same canonical key
    let alias_groups: &[(&str, &[&str])] = &[
        ("doc_density", &["documentation_density", "docs"]),
        ("whitespace_ratio", &["whitespace"]),
        ("verbosity", &["bytes_per_line"]),
        ("test_density", &["tests"]),
        ("todo_density", &["todo", "fixme"]),
        ("polyglot_entropy", &["language_entropy", "polyglot"]),
        ("gini", &["distribution_gini"]),
        ("avg_cyclomatic", &["cyclomatic"]),
        ("avg_cognitive", &["cognitive"]),
        ("max_nesting_depth", &["nesting_depth"]),
        ("maintainability_index", &["mi"]),
        ("technical_debt_ratio", &["debt_ratio", "technical_debt"]),
        ("halstead", &["halstead_volume", "halstead_effort"]),
        ("complexity_histogram", &["histogram"]),
        ("hotspots", &["git_hotspots"]),
        ("bus_factor", &["ownership"]),
        ("freshness", &["staleness"]),
        ("code_age_distribution", &["code_age", "age_buckets"]),
        ("coupling", &["module_coupling"]),
        ("predictive_churn", &["churn"]),
        ("duplicate_waste", &["dup", "duplication"]),
        ("duplication_density", &["dup_density"]),
        ("imports", &["import_graph"]),
        ("entropy_suspects", &["entropy"]),
        ("license_radar", &["license"]),
        ("archetype", &["project_archetype"]),
        ("context_window_fit", &["window_fit", "context_fit"]),
    ];

    for &(canonical, aliases) in alias_groups {
        let canonical_result = lookup(canonical).unwrap();
        for alias in aliases {
            let alias_result = lookup(alias).unwrap();
            assert_eq!(
                canonical_result, alias_result,
                "alias '{alias}' should produce same result as canonical '{canonical}'"
            );
        }
    }
}

// ── Regression: entries with empty alias lists ──────────────────────

#[test]
fn canonical_keys_without_aliases_still_resolve() {
    // max_cyclomatic has no aliases
    let result = lookup("max_cyclomatic");
    assert!(result.is_some());
    assert!(result.unwrap().starts_with("max_cyclomatic:"));
}

// ── Consistent summary content ──────────────────────────────────────

#[test]
fn all_summaries_end_with_period() {
    let all_keys = [
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
    for key in all_keys {
        let text = lookup(key).unwrap();
        let summary = text.split_once(": ").unwrap().1;
        assert!(
            summary.ends_with('.'),
            "summary for '{key}' should end with '.', got: {summary}"
        );
    }
}

#[test]
fn all_summaries_are_single_sentence() {
    // No summary should contain a newline
    let all_keys = [
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
    for key in all_keys {
        let text = lookup(key).unwrap();
        assert!(
            !text.contains('\n'),
            "explanation for '{key}' should be single-line"
        );
    }
}
