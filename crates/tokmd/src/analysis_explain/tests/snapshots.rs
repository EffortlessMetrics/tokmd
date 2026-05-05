// Snapshot tests for analysis_explain module using insta

use crate::analysis_explain::{catalog, lookup};

fn assert_lookup_snapshot(metric: &str) {
    let snapshot_name = format!("lookup_{metric}");
    let text = lookup(metric).unwrap_or_else(|| panic!("missing metric: {metric}"));
    insta::assert_snapshot!(snapshot_name, text);
}

#[test]
fn snapshot_catalog_full() {
    let text = catalog();
    insta::assert_snapshot!("catalog_full", text);
}

#[test]
fn snapshot_lookup_metrics() {
    let metrics = [
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

    for metric in metrics {
        assert_lookup_snapshot(metric);
    }
}

// Alias lookups should produce identical output to canonical lookups
#[test]
fn snapshot_alias_mi_matches_maintainability_index() {
    let canonical = lookup("maintainability_index").unwrap();
    let via_alias = lookup("mi").unwrap();
    assert_eq!(canonical, via_alias);
}

#[test]
fn snapshot_alias_churn_matches_predictive_churn() {
    let canonical = lookup("predictive_churn").unwrap();
    let via_alias = lookup("churn").unwrap();
    assert_eq!(canonical, via_alias);
}

#[test]
fn snapshot_alias_entropy_matches_entropy_suspects() {
    let canonical = lookup("entropy_suspects").unwrap();
    let via_alias = lookup("entropy").unwrap();
    assert_eq!(canonical, via_alias);
}
