//! Deep tests for tokmd-analysis-explain (W69).
//!
//! Covers: lookup by canonical key, lookup by alias, normalization,
//! catalog determinism, unknown keys, edge cases.

use tokmd_analysis_explain::{catalog, lookup};

// ===================================================================
// 1. Canonical key lookups
// ===================================================================

#[test]
fn lookup_doc_density() {
    let result = lookup("doc_density").unwrap();
    assert!(result.starts_with("doc_density:"));
    assert!(result.contains("comment"));
}

#[test]
fn lookup_whitespace_ratio() {
    let result = lookup("whitespace_ratio").unwrap();
    assert!(result.starts_with("whitespace_ratio:"));
    assert!(result.contains("blank"));
}

#[test]
fn lookup_verbosity() {
    let result = lookup("verbosity").unwrap();
    assert!(result.starts_with("verbosity:"));
}

#[test]
fn lookup_test_density() {
    let result = lookup("test_density").unwrap();
    assert!(result.starts_with("test_density:"));
    assert!(result.contains("test"));
}

#[test]
fn lookup_todo_density() {
    let result = lookup("todo_density").unwrap();
    assert!(result.starts_with("todo_density:"));
}

#[test]
fn lookup_gini() {
    let result = lookup("gini").unwrap();
    assert!(result.starts_with("gini:"));
    assert!(result.contains("file"));
}

#[test]
fn lookup_hotspots() {
    let result = lookup("hotspots").unwrap();
    assert!(result.starts_with("hotspots:"));
}

#[test]
fn lookup_bus_factor() {
    let result = lookup("bus_factor").unwrap();
    assert!(result.starts_with("bus_factor:"));
    assert!(result.contains("author"));
}

#[test]
fn lookup_coupling() {
    let result = lookup("coupling").unwrap();
    assert!(result.starts_with("coupling:"));
}

#[test]
fn lookup_imports() {
    let result = lookup("imports").unwrap();
    assert!(result.starts_with("imports:"));
}

// ===================================================================
// 2. Alias lookups
// ===================================================================

#[test]
fn lookup_alias_docs() {
    let result = lookup("docs").unwrap();
    assert!(result.starts_with("doc_density:"));
}

#[test]
fn lookup_alias_todo() {
    let result = lookup("todo").unwrap();
    assert!(result.starts_with("todo_density:"));
}

#[test]
fn lookup_alias_churn() {
    let result = lookup("churn").unwrap();
    assert!(result.starts_with("predictive_churn:"));
}

// ===================================================================
// 3. Normalization
// ===================================================================

#[test]
fn lookup_normalizes_dashes() {
    let result = lookup("doc-density").unwrap();
    assert!(result.starts_with("doc_density:"));
}

#[test]
fn lookup_normalizes_spaces() {
    let result = lookup("doc density").unwrap();
    assert!(result.starts_with("doc_density:"));
}

#[test]
fn lookup_normalizes_case() {
    let result = lookup("DOC_DENSITY").unwrap();
    assert!(result.starts_with("doc_density:"));
}

#[test]
fn lookup_normalizes_dots() {
    let result = lookup("doc.density").unwrap();
    assert!(result.starts_with("doc_density:"));
}

#[test]
fn lookup_normalizes_leading_trailing_whitespace() {
    let result = lookup("  doc_density  ").unwrap();
    assert!(result.starts_with("doc_density:"));
}

// ===================================================================
// 4. Unknown keys
// ===================================================================

#[test]
fn lookup_unknown_key_returns_none() {
    assert!(lookup("nonexistent_metric").is_none());
}

#[test]
fn lookup_empty_string_returns_none() {
    assert!(lookup("").is_none());
}

// ===================================================================
// 5. Catalog
// ===================================================================

#[test]
fn catalog_starts_with_header() {
    let cat = catalog();
    assert!(cat.starts_with("Available metric/finding keys:"));
}

#[test]
fn catalog_contains_known_keys() {
    let cat = catalog();
    assert!(cat.contains("doc_density"));
    assert!(cat.contains("gini"));
    assert!(cat.contains("hotspots"));
    assert!(cat.contains("imports"));
}

#[test]
fn catalog_is_sorted() {
    let cat = catalog();
    let keys: Vec<&str> = cat
        .lines()
        .skip(1)
        .filter_map(|l| l.strip_prefix("- "))
        .collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(keys, sorted);
}

#[test]
fn catalog_has_no_duplicates() {
    let cat = catalog();
    let keys: Vec<&str> = cat
        .lines()
        .skip(1)
        .filter_map(|l| l.strip_prefix("- "))
        .collect();
    let mut deduped = keys.clone();
    deduped.dedup();
    assert_eq!(keys, deduped);
}

#[test]
fn catalog_is_deterministic() {
    let a = catalog();
    let b = catalog();
    assert_eq!(a, b, "catalog() must be deterministic");
}
