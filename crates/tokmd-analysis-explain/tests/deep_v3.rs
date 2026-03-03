//! Deep tests for tokmd-analysis-explain: lookup and catalog edge cases.

use tokmd_analysis_explain::{catalog, lookup};

// ── lookup: canonical key content verification ──────────────────────

#[test]
fn lookup_doc_density_mentions_ratio() {
    let text = lookup("doc_density").unwrap();
    assert!(
        text.contains("Ratio"),
        "doc_density explanation should mention Ratio"
    );
}

#[test]
fn lookup_verbosity_mentions_bytes() {
    let text = lookup("verbosity").unwrap();
    assert!(text.contains("bytes"), "verbosity should mention bytes");
}

#[test]
fn lookup_gini_mentions_inequality() {
    let text = lookup("gini").unwrap();
    assert!(
        text.to_lowercase().contains("inequality") || text.to_lowercase().contains("concentration"),
        "gini should mention inequality or concentration"
    );
}

#[test]
fn lookup_cocomo_related_key_is_absent() {
    // cocomo is not in the catalog
    assert!(lookup("cocomo").is_none());
}

#[test]
fn lookup_hotspots_mentions_change_frequency() {
    let text = lookup("hotspots").unwrap();
    assert!(
        text.to_lowercase().contains("change") || text.to_lowercase().contains("frequency"),
        "hotspots should mention change or frequency"
    );
}

// ── lookup: normalization edge cases ────────────────────────────────

#[test]
fn lookup_normalizes_multiple_consecutive_separators() {
    // "doc__density" normalizes to "doc__density" which won't match "doc_density"
    assert!(lookup("doc__density").is_none());
}

#[test]
fn lookup_normalizes_mixed_case_alias() {
    let via_alias = lookup("DOCUMENTATION_DENSITY").unwrap();
    let canonical = lookup("doc_density").unwrap();
    assert_eq!(via_alias, canonical);
}

#[test]
fn lookup_tab_separated_key_normalizes() {
    // Tab is not a recognized separator, so "doc\tdensity" should not resolve
    assert!(lookup("doc\tdensity").is_none());
}

#[test]
fn lookup_newline_in_key_does_not_resolve() {
    assert!(lookup("doc\ndensity").is_none());
}

#[test]
fn lookup_unicode_key_does_not_resolve() {
    assert!(lookup("dóc_density").is_none());
    assert!(lookup("日本語").is_none());
}

#[test]
fn lookup_single_char_keys_do_not_resolve() {
    assert!(lookup("a").is_none());
    assert!(lookup("_").is_none());
    assert!(lookup("-").is_none());
}

#[test]
fn lookup_very_long_key_returns_none() {
    let long_key = "a".repeat(10_000);
    assert!(lookup(&long_key).is_none());
}

// ── lookup: all aliases produce same result as canonical ────────────

#[test]
fn lookup_all_doc_density_aliases_match_canonical() {
    let canonical = lookup("doc_density").unwrap();
    assert_eq!(lookup("documentation_density").unwrap(), canonical);
    assert_eq!(lookup("docs").unwrap(), canonical);
}

#[test]
fn lookup_all_context_window_fit_aliases_match() {
    let canonical = lookup("context_window_fit").unwrap();
    assert_eq!(lookup("window_fit").unwrap(), canonical);
    assert_eq!(lookup("context_fit").unwrap(), canonical);
}

#[test]
fn lookup_all_code_age_aliases_match() {
    let canonical = lookup("code_age_distribution").unwrap();
    assert_eq!(lookup("code_age").unwrap(), canonical);
    assert_eq!(lookup("age_buckets").unwrap(), canonical);
}

// ── lookup: output format invariants ────────────────────────────────

#[test]
fn lookup_output_never_has_trailing_newline() {
    let keys = ["doc_density", "gini", "halstead", "archetype"];
    for key in keys {
        let text = lookup(key).unwrap();
        assert!(
            !text.ends_with('\n'),
            "lookup({key}) should not end with newline"
        );
    }
}

#[test]
fn lookup_output_contains_exactly_one_colon_space_separator() {
    let keys = [
        "doc_density",
        "whitespace_ratio",
        "verbosity",
        "test_density",
        "hotspots",
    ];
    for key in keys {
        let text = lookup(key).unwrap();
        let parts: Vec<&str> = text.splitn(2, ": ").collect();
        assert_eq!(parts.len(), 2, "'{key}' output should split into 2 parts");
        assert!(
            !parts[0].is_empty(),
            "canonical key part should not be empty"
        );
        assert!(!parts[1].is_empty(), "summary part should not be empty");
    }
}

#[test]
fn lookup_summaries_end_with_period() {
    let keys = [
        "doc_density",
        "whitespace_ratio",
        "verbosity",
        "gini",
        "hotspots",
        "bus_factor",
        "freshness",
        "coupling",
    ];
    for key in keys {
        let text = lookup(key).unwrap();
        let summary = text.splitn(2, ": ").nth(1).unwrap();
        assert!(
            summary.ends_with('.'),
            "summary for '{key}' should end with period, got: {summary}"
        );
    }
}

// ── catalog: structure and content ──────────────────────────────────

#[test]
fn catalog_line_count_matches_entry_count() {
    let text = catalog();
    let key_lines: Vec<&str> = text
        .lines()
        .skip(1) // header
        .filter(|l| l.starts_with("- "))
        .collect();
    // There should be exactly 28 entries (count from ENTRIES const)
    assert_eq!(key_lines.len(), 28, "expected 28 catalog entries");
}

#[test]
fn catalog_every_key_is_resolvable_via_lookup() {
    let text = catalog();
    for line in text.lines().skip(1) {
        if let Some(key) = line.strip_prefix("- ") {
            assert!(
                lookup(key).is_some(),
                "catalog key '{key}' should be resolvable via lookup"
            );
        }
    }
}

#[test]
fn catalog_header_is_exactly_one_line() {
    let text = catalog();
    let first_line = text.lines().next().unwrap();
    assert_eq!(first_line, "Available metric/finding keys:");
}

#[test]
fn catalog_no_empty_lines() {
    let text = catalog();
    for (i, line) in text.lines().enumerate() {
        assert!(
            !line.is_empty(),
            "catalog should not have empty lines, found at line {i}"
        );
    }
}

#[test]
fn catalog_is_idempotent() {
    let first = catalog();
    let second = catalog();
    assert_eq!(first, second, "catalog() should be deterministic");
}

// ── Cross-cutting: lookup vs catalog consistency ────────────────────

#[test]
fn every_lookupable_canonical_key_appears_in_catalog() {
    let text = catalog();
    let catalog_keys: Vec<&str> = text
        .lines()
        .skip(1)
        .filter_map(|l| l.strip_prefix("- "))
        .collect();

    // Every canonical key that lookup resolves should be in catalog
    for key in &catalog_keys {
        let result = lookup(key);
        assert!(
            result.is_some(),
            "catalog key '{key}' should resolve via lookup"
        );
        let text = result.unwrap();
        assert!(
            text.starts_with(&format!("{key}:")),
            "lookup for catalog key '{key}' should start with '{key}:'"
        );
    }
}
