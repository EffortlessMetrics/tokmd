//! Wave-57 depth tests for `tokmd-analysis-explain`.
//!
//! Covers:
//! - Explanation lookup for various metrics
//! - Formatting of explanations
//! - Catalog completeness
//! - Case-insensitive and separator-normalized lookup
//! - Unknown metric names

use tokmd_analysis_explain::{catalog, lookup};

// =============================================================================
// 1. Canonical key lookup
// =============================================================================

#[test]
fn lookup_all_canonical_keys_resolve() {
    let cat = catalog();
    let keys: Vec<&str> = cat
        .lines()
        .skip(1)
        .filter_map(|line| line.strip_prefix("- "))
        .collect();
    for key in &keys {
        assert!(lookup(key).is_some(), "Canonical key '{key}' must resolve");
    }
}

#[test]
fn lookup_returns_canonical_key_as_prefix() {
    let result = lookup("doc_density").unwrap();
    assert!(
        result.starts_with("doc_density:"),
        "Result must start with canonical key"
    );
}

#[test]
fn lookup_result_contains_nonempty_description() {
    let result = lookup("gini").unwrap();
    let parts: Vec<&str> = result.splitn(2, ": ").collect();
    assert_eq!(parts.len(), 2);
    assert!(
        !parts[1].is_empty(),
        "Description after colon must not be empty"
    );
}

// =============================================================================
// 2. Alias resolution
// =============================================================================

#[test]
fn lookup_alias_resolves_to_canonical() {
    let via_alias = lookup("documentation_density").unwrap();
    let via_canonical = lookup("doc_density").unwrap();
    assert_eq!(via_alias, via_canonical);
}

#[test]
fn lookup_multiple_aliases_for_same_key() {
    // doc_density has aliases: documentation_density, docs
    let a = lookup("documentation_density").unwrap();
    let b = lookup("docs").unwrap();
    assert_eq!(a, b);
}

#[test]
fn lookup_mi_alias_resolves_to_maintainability_index() {
    let result = lookup("mi").unwrap();
    assert!(result.starts_with("maintainability_index:"));
}

#[test]
fn lookup_churn_alias_resolves_to_predictive_churn() {
    let result = lookup("churn").unwrap();
    assert!(result.starts_with("predictive_churn:"));
}

// =============================================================================
// 3. Case-insensitive and separator normalization
// =============================================================================

#[test]
fn lookup_uppercase_key() {
    let result = lookup("DOC_DENSITY").unwrap();
    assert!(result.starts_with("doc_density:"));
}

#[test]
fn lookup_mixed_case_key() {
    let result = lookup("Doc_Density").unwrap();
    assert!(result.starts_with("doc_density:"));
}

#[test]
fn lookup_hyphen_separator() {
    let result = lookup("doc-density").unwrap();
    assert!(result.starts_with("doc_density:"));
}

#[test]
fn lookup_dot_separator() {
    let result = lookup("doc.density").unwrap();
    assert!(result.starts_with("doc_density:"));
}

#[test]
fn lookup_space_separator() {
    let result = lookup("doc density").unwrap();
    assert!(result.starts_with("doc_density:"));
}

#[test]
fn lookup_leading_trailing_whitespace_trimmed() {
    let result = lookup("  gini  ").unwrap();
    assert!(result.starts_with("gini:"));
}

// =============================================================================
// 4. Unknown metric names
// =============================================================================

#[test]
fn lookup_unknown_key_returns_none() {
    assert!(lookup("nonexistent_metric").is_none());
}

#[test]
fn lookup_empty_string_returns_none() {
    assert!(lookup("").is_none());
}

#[test]
fn lookup_whitespace_only_returns_none() {
    assert!(lookup("   ").is_none());
}

// =============================================================================
// 5. Catalog completeness and format
// =============================================================================

#[test]
fn catalog_starts_with_header_line() {
    let cat = catalog();
    let first_line = cat.lines().next().unwrap();
    assert_eq!(first_line, "Available metric/finding keys:");
}

#[test]
fn catalog_has_at_least_25_entries() {
    let cat = catalog();
    let count = cat.lines().filter(|l| l.starts_with("- ")).count();
    assert!(
        count >= 25,
        "Catalog should have at least 25 entries, got {count}"
    );
}

#[test]
fn catalog_is_sorted_ascending() {
    let cat = catalog();
    let keys: Vec<&str> = cat
        .lines()
        .skip(1)
        .filter_map(|line| line.strip_prefix("- "))
        .collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(keys, sorted, "Catalog keys must be sorted");
}

#[test]
fn catalog_has_no_duplicate_entries() {
    let cat = catalog();
    let keys: Vec<&str> = cat
        .lines()
        .skip(1)
        .filter_map(|line| line.strip_prefix("- "))
        .collect();
    let unique: BTreeSet<&str> = keys.iter().copied().collect();
    assert_eq!(keys.len(), unique.len(), "Catalog must have no duplicates");
}

#[test]
fn catalog_ends_with_newline() {
    let cat = catalog();
    assert!(cat.ends_with('\n'));
}

#[test]
fn catalog_is_deterministic() {
    let c1 = catalog();
    let c2 = catalog();
    assert_eq!(c1, c2);
}

use std::collections::BTreeSet;
