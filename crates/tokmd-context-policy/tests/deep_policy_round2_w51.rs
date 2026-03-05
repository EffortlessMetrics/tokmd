//! Deep round-2 tests for tokmd-context-policy (W51).

use tokmd_context_policy::{
    DEFAULT_DENSE_THRESHOLD, DEFAULT_MAX_FILE_PCT, DEFAULT_MAX_FILE_TOKENS, assign_policy,
    classify_file, compute_file_cap, is_spine_file, smart_exclude_reason,
};
use tokmd_types::{FileClassification, InclusionPolicy};

// ---------------------------------------------------------------------------
// File classification tests
// ---------------------------------------------------------------------------

#[test]
fn classify_source_file_returns_empty() {
    let classes = classify_file("src/main.rs", 200, 50, DEFAULT_DENSE_THRESHOLD);
    assert!(
        classes.is_empty(),
        "plain source file should have no classifications"
    );
}

#[test]
fn classify_config_lockfile() {
    let classes = classify_file("Cargo.lock", 5000, 400, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Lockfile));
}

#[test]
fn classify_docs_file_has_no_special_class() {
    let classes = classify_file("docs/tutorial.md", 300, 80, DEFAULT_DENSE_THRESHOLD);
    assert!(
        classes.is_empty(),
        "docs markdown should have no special classification"
    );
}

#[test]
fn classify_test_fixture_dir() {
    let classes = classify_file(
        "fixtures/golden/output.json",
        1000,
        50,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Fixture));
}

#[test]
fn classify_generated_protobuf() {
    let classes = classify_file("proto/api.pb.rs", 10_000, 500, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_generated_node_types() {
    let classes = classify_file(
        "grammar/node-types.json",
        50_000,
        5,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Generated));
    assert!(classes.contains(&FileClassification::DataBlob));
}

// ---------------------------------------------------------------------------
// Smart exclude pattern tests
// ---------------------------------------------------------------------------

#[test]
fn smart_exclude_detects_all_lockfile_variants() {
    let lockfiles = [
        "Cargo.lock",
        "package-lock.json",
        "pnpm-lock.yaml",
        "yarn.lock",
        "poetry.lock",
        "Pipfile.lock",
        "go.sum",
        "composer.lock",
        "Gemfile.lock",
    ];
    for lockfile in lockfiles {
        assert_eq!(
            smart_exclude_reason(lockfile),
            Some("lockfile"),
            "{lockfile} should be excluded as lockfile"
        );
    }
}

#[test]
fn smart_exclude_detects_minified() {
    assert_eq!(smart_exclude_reason("dist/app.min.js"), Some("minified"));
    assert_eq!(smart_exclude_reason("styles.min.css"), Some("minified"));
}

#[test]
fn smart_exclude_detects_sourcemaps() {
    assert_eq!(
        smart_exclude_reason("dist/bundle.js.map"),
        Some("sourcemap")
    );
    assert_eq!(smart_exclude_reason("styles.css.map"), Some("sourcemap"));
}

#[test]
fn smart_exclude_ignores_regular_files() {
    assert_eq!(smart_exclude_reason("src/main.rs"), None);
    assert_eq!(smart_exclude_reason("README.md"), None);
    assert_eq!(smart_exclude_reason("Cargo.toml"), None);
}

#[test]
fn smart_exclude_lockfile_in_nested_path() {
    assert_eq!(
        smart_exclude_reason("packages/web/package-lock.json"),
        Some("lockfile"),
    );
}

// ---------------------------------------------------------------------------
// Inclusion policy tests
// ---------------------------------------------------------------------------

#[test]
fn policy_full_when_under_cap() {
    let cap = compute_file_cap(100_000, DEFAULT_MAX_FILE_PCT, None);
    let (policy, reason) = assign_policy(100, cap, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

#[test]
fn policy_head_tail_when_over_cap_plain_file() {
    let cap = 1_000;
    let (policy, reason) = assign_policy(5_000, cap, &[]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
    assert!(reason.as_deref().unwrap().contains("head+tail"));
}

#[test]
fn policy_skip_for_generated_over_cap() {
    let cap = 1_000;
    let (policy, reason) = assign_policy(5_000, cap, &[FileClassification::Generated]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.as_deref().unwrap().contains("generated"));
}

#[test]
fn policy_skip_for_vendored_over_cap() {
    let cap = 1_000;
    let (policy, reason) = assign_policy(5_000, cap, &[FileClassification::Vendored]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.as_deref().unwrap().contains("vendored"));
}

#[test]
fn policy_skip_for_data_blob_over_cap() {
    let cap = 1_000;
    let (policy, reason) = assign_policy(5_000, cap, &[FileClassification::DataBlob]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.as_deref().unwrap().contains("data_blob"));
}

// ---------------------------------------------------------------------------
// Token budget allocation tests
// ---------------------------------------------------------------------------

#[test]
fn file_cap_respects_percentage_limit() {
    let cap = compute_file_cap(100_000, 0.10, None);
    assert_eq!(cap, 10_000, "10% of 100k = 10k");
}

#[test]
fn file_cap_respects_hard_cap() {
    let cap = compute_file_cap(1_000_000, 0.50, Some(20_000));
    assert_eq!(cap, 20_000, "hard cap should win over percentage");
}

#[test]
fn file_cap_unlimited_budget_returns_max() {
    let cap = compute_file_cap(usize::MAX, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, usize::MAX);
}

#[test]
fn file_cap_zero_budget_returns_zero() {
    let cap = compute_file_cap(0, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, 0);
}

#[test]
fn file_cap_defaults_match_constants() {
    let cap = compute_file_cap(200_000, DEFAULT_MAX_FILE_PCT, None);
    // 200_000 * 0.15 = 30_000, but hard cap is 16_000
    assert_eq!(cap, DEFAULT_MAX_FILE_TOKENS);
}

// ---------------------------------------------------------------------------
// Priority / spine file tests
// ---------------------------------------------------------------------------

#[test]
fn spine_detects_readme_variants() {
    assert!(is_spine_file("README.md"));
    assert!(is_spine_file("README"));
    assert!(is_spine_file("README.rst"));
    assert!(is_spine_file("README.txt"));
}

#[test]
fn spine_detects_config_manifests() {
    assert!(is_spine_file("Cargo.toml"));
    assert!(is_spine_file("package.json"));
    assert!(is_spine_file("pyproject.toml"));
    assert!(is_spine_file("go.mod"));
}

#[test]
fn spine_detects_docs_architecture() {
    assert!(is_spine_file("docs/architecture.md"));
    assert!(is_spine_file("nested/docs/architecture.md"));
}

#[test]
fn spine_rejects_random_source() {
    assert!(!is_spine_file("src/main.rs"));
    assert!(!is_spine_file("lib/utils.py"));
    assert!(!is_spine_file("index.js"));
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn policy_with_empty_classifications_under_cap() {
    let (policy, reason) = assign_policy(50, 1000, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

#[test]
fn classify_empty_path() {
    let classes = classify_file("", 0, 0, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.is_empty());
}

#[test]
fn classify_vendored_node_modules() {
    let classes = classify_file(
        "node_modules/lodash/index.js",
        500,
        100,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Vendored));
}

#[test]
fn classify_vendored_third_party() {
    let classes = classify_file(
        "third_party/protobuf/src/lib.rs",
        500,
        100,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Vendored));
}

#[test]
fn classify_testdata_fixture() {
    let classes = classify_file("testdata/sample.json", 200, 20, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));
}

#[test]
fn classify_snapshots_fixture() {
    let classes = classify_file(
        "__snapshots__/component.snap",
        300,
        30,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Fixture));
}

#[test]
fn multiple_classifications_sorted_and_deduped() {
    // A vendored generated file that is also dense
    let classes = classify_file(
        "vendor/proto/api.pb.go",
        100_000,
        10,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::Generated));
    assert!(classes.contains(&FileClassification::DataBlob));
    // Verify sorted (no duplicates, ascending order)
    let mut sorted = classes.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(classes, sorted);
}

#[test]
fn spine_with_backslash_paths_normalized() {
    assert!(is_spine_file("docs\\architecture.md"));
}
