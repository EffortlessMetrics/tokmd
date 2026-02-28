// Snapshot tests for tokmd-analysis-explain using insta

use tokmd_analysis_explain::{catalog, lookup};

#[test]
fn snapshot_catalog_full() {
    let text = catalog();
    insta::assert_snapshot!("catalog_full", text);
}

#[test]
fn snapshot_lookup_doc_density() {
    insta::assert_snapshot!("lookup_doc_density", lookup("doc_density").unwrap());
}

#[test]
fn snapshot_lookup_whitespace_ratio() {
    insta::assert_snapshot!(
        "lookup_whitespace_ratio",
        lookup("whitespace_ratio").unwrap()
    );
}

#[test]
fn snapshot_lookup_verbosity() {
    insta::assert_snapshot!("lookup_verbosity", lookup("verbosity").unwrap());
}

#[test]
fn snapshot_lookup_test_density() {
    insta::assert_snapshot!("lookup_test_density", lookup("test_density").unwrap());
}

#[test]
fn snapshot_lookup_todo_density() {
    insta::assert_snapshot!("lookup_todo_density", lookup("todo_density").unwrap());
}

#[test]
fn snapshot_lookup_polyglot_entropy() {
    insta::assert_snapshot!(
        "lookup_polyglot_entropy",
        lookup("polyglot_entropy").unwrap()
    );
}

#[test]
fn snapshot_lookup_gini() {
    insta::assert_snapshot!("lookup_gini", lookup("gini").unwrap());
}

#[test]
fn snapshot_lookup_avg_cyclomatic() {
    insta::assert_snapshot!("lookup_avg_cyclomatic", lookup("avg_cyclomatic").unwrap());
}

#[test]
fn snapshot_lookup_max_cyclomatic() {
    insta::assert_snapshot!("lookup_max_cyclomatic", lookup("max_cyclomatic").unwrap());
}

#[test]
fn snapshot_lookup_avg_cognitive() {
    insta::assert_snapshot!("lookup_avg_cognitive", lookup("avg_cognitive").unwrap());
}

#[test]
fn snapshot_lookup_max_nesting_depth() {
    insta::assert_snapshot!(
        "lookup_max_nesting_depth",
        lookup("max_nesting_depth").unwrap()
    );
}

#[test]
fn snapshot_lookup_maintainability_index() {
    insta::assert_snapshot!(
        "lookup_maintainability_index",
        lookup("maintainability_index").unwrap()
    );
}

#[test]
fn snapshot_lookup_technical_debt_ratio() {
    insta::assert_snapshot!(
        "lookup_technical_debt_ratio",
        lookup("technical_debt_ratio").unwrap()
    );
}

#[test]
fn snapshot_lookup_halstead() {
    insta::assert_snapshot!("lookup_halstead", lookup("halstead").unwrap());
}

#[test]
fn snapshot_lookup_complexity_histogram() {
    insta::assert_snapshot!(
        "lookup_complexity_histogram",
        lookup("complexity_histogram").unwrap()
    );
}

#[test]
fn snapshot_lookup_hotspots() {
    insta::assert_snapshot!("lookup_hotspots", lookup("hotspots").unwrap());
}

#[test]
fn snapshot_lookup_bus_factor() {
    insta::assert_snapshot!("lookup_bus_factor", lookup("bus_factor").unwrap());
}

#[test]
fn snapshot_lookup_freshness() {
    insta::assert_snapshot!("lookup_freshness", lookup("freshness").unwrap());
}

#[test]
fn snapshot_lookup_code_age_distribution() {
    insta::assert_snapshot!(
        "lookup_code_age_distribution",
        lookup("code_age_distribution").unwrap()
    );
}

#[test]
fn snapshot_lookup_coupling() {
    insta::assert_snapshot!("lookup_coupling", lookup("coupling").unwrap());
}

#[test]
fn snapshot_lookup_predictive_churn() {
    insta::assert_snapshot!(
        "lookup_predictive_churn",
        lookup("predictive_churn").unwrap()
    );
}

#[test]
fn snapshot_lookup_duplicate_waste() {
    insta::assert_snapshot!("lookup_duplicate_waste", lookup("duplicate_waste").unwrap());
}

#[test]
fn snapshot_lookup_duplication_density() {
    insta::assert_snapshot!(
        "lookup_duplication_density",
        lookup("duplication_density").unwrap()
    );
}

#[test]
fn snapshot_lookup_imports() {
    insta::assert_snapshot!("lookup_imports", lookup("imports").unwrap());
}

#[test]
fn snapshot_lookup_entropy_suspects() {
    insta::assert_snapshot!(
        "lookup_entropy_suspects",
        lookup("entropy_suspects").unwrap()
    );
}

#[test]
fn snapshot_lookup_license_radar() {
    insta::assert_snapshot!("lookup_license_radar", lookup("license_radar").unwrap());
}

#[test]
fn snapshot_lookup_archetype() {
    insta::assert_snapshot!("lookup_archetype", lookup("archetype").unwrap());
}

#[test]
fn snapshot_lookup_context_window_fit() {
    insta::assert_snapshot!(
        "lookup_context_window_fit",
        lookup("context_window_fit").unwrap()
    );
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
