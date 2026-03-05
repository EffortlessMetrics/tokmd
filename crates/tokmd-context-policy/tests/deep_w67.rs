//! Deep tests for tokmd-context-policy (w67).
//!
//! Covers: smart excludes, file classification, inclusion policy assignment,
//! token budget / file cap, spine file detection, determinism invariants,
//! and property-based tests.

use tokmd_context_policy::{
    DEFAULT_DENSE_THRESHOLD, DEFAULT_MAX_FILE_PCT, DEFAULT_MAX_FILE_TOKENS, assign_policy,
    classify_file, compute_file_cap, is_spine_file, smart_exclude_reason,
};
use tokmd_types::{FileClassification, InclusionPolicy};

// ===========================================================================
// Helpers
// ===========================================================================

/// Full pipeline: classify → cap → assign.
fn pipeline(path: &str, tokens: usize, lines: usize, budget: usize) -> InclusionPolicy {
    let classes = classify_file(path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
    let cap = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, None);
    let (policy, _) = assign_policy(tokens, cap, &classes);
    policy
}

// ===========================================================================
// 1. smart_exclude_reason
// ===========================================================================

#[test]
fn smart_exclude_lockfile_cargo() {
    assert_eq!(smart_exclude_reason("Cargo.lock"), Some("lockfile"));
}

#[test]
fn smart_exclude_lockfile_npm() {
    assert_eq!(smart_exclude_reason("package-lock.json"), Some("lockfile"));
}

#[test]
fn smart_exclude_lockfile_yarn() {
    assert_eq!(smart_exclude_reason("yarn.lock"), Some("lockfile"));
}

#[test]
fn smart_exclude_lockfile_nested_path() {
    assert_eq!(
        smart_exclude_reason("some/deep/path/Gemfile.lock"),
        Some("lockfile")
    );
}

#[test]
fn smart_exclude_minified_js() {
    assert_eq!(smart_exclude_reason("app.min.js"), Some("minified"));
}

#[test]
fn smart_exclude_minified_css() {
    assert_eq!(
        smart_exclude_reason("dist/styles.min.css"),
        Some("minified")
    );
}

#[test]
fn smart_exclude_sourcemap_js() {
    assert_eq!(smart_exclude_reason("bundle.js.map"), Some("sourcemap"));
}

#[test]
fn smart_exclude_sourcemap_css() {
    assert_eq!(smart_exclude_reason("theme.css.map"), Some("sourcemap"));
}

#[test]
fn smart_exclude_none_for_source() {
    assert_eq!(smart_exclude_reason("src/main.rs"), None);
    assert_eq!(smart_exclude_reason("lib/util.py"), None);
    assert_eq!(smart_exclude_reason("index.js"), None);
}

// ===========================================================================
// 2. classify_file
// ===========================================================================

#[test]
fn classify_lockfile() {
    let classes = classify_file("Cargo.lock", 1000, 500, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Lockfile));
}

#[test]
fn classify_minified() {
    let classes = classify_file("app.min.js", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Minified));
}

#[test]
fn classify_sourcemap() {
    let classes = classify_file("bundle.js.map", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Sourcemap));
}

#[test]
fn classify_generated_protobuf_go() {
    let classes = classify_file("api.pb.go", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_generated_protobuf_rs() {
    let classes = classify_file("api.pb.rs", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_generated_python_pb2() {
    let classes = classify_file("service_pb2.py", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_generated_node_types_json() {
    let classes = classify_file("node-types.json", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_vendored_directory() {
    let classes = classify_file("vendor/lib/foo.go", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Vendored));
}

#[test]
fn classify_node_modules_vendored() {
    let classes = classify_file(
        "node_modules/lodash/index.js",
        500,
        100,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Vendored));
}

#[test]
fn classify_fixture_directory() {
    let classes = classify_file("fixtures/sample.json", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));
}

#[test]
fn classify_testdata_fixture() {
    let classes = classify_file("testdata/input.txt", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));
}

#[test]
fn classify_data_blob_high_density() {
    // 10_000 tokens / 10 lines = 1000 tokens/line >> threshold
    let classes = classify_file("data.bin", 10_000, 10, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn classify_no_data_blob_low_density() {
    // 100 tokens / 100 lines = 1 token/line << threshold
    let classes = classify_file("src/main.rs", 100, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(!classes.contains(&FileClassification::DataBlob));
}

#[test]
fn classify_plain_source_file_empty() {
    let classes = classify_file("src/lib.rs", 100, 50, DEFAULT_DENSE_THRESHOLD);
    assert!(
        classes.is_empty(),
        "plain source should have no classifications"
    );
}

#[test]
fn classify_sorted_and_deduped() {
    // A file in vendor/ that is also generated: both should appear, sorted, no dups
    let classes = classify_file("vendor/generated.pb.go", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
    assert!(classes.contains(&FileClassification::Vendored));
    // Verify sorted order
    for pair in classes.windows(2) {
        assert!(pair[0] <= pair[1], "classifications must be sorted");
    }
}

// ===========================================================================
// 3. is_spine_file
// ===========================================================================

#[test]
fn spine_readme_md() {
    assert!(is_spine_file("README.md"));
}

#[test]
fn spine_cargo_toml() {
    assert!(is_spine_file("Cargo.toml"));
}

#[test]
fn spine_package_json() {
    assert!(is_spine_file("package.json"));
}

#[test]
fn spine_docs_architecture() {
    assert!(is_spine_file("docs/architecture.md"));
}

#[test]
fn spine_nested_docs_architecture() {
    assert!(is_spine_file("project/docs/architecture.md"));
}

#[test]
fn spine_not_random_source() {
    assert!(!is_spine_file("src/main.rs"));
    assert!(!is_spine_file("lib/util.py"));
}

// ===========================================================================
// 4. compute_file_cap
// ===========================================================================

#[test]
fn file_cap_respects_pct() {
    // budget=100_000, 15% = 15_000 -> below hard cap 16_000
    let cap = compute_file_cap(100_000, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, 15_000);
}

#[test]
fn file_cap_respects_hard_cap() {
    // budget=1_000_000, 15% = 150_000 -> above hard cap 16_000
    let cap = compute_file_cap(1_000_000, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, DEFAULT_MAX_FILE_TOKENS);
}

#[test]
fn file_cap_custom_hard_cap() {
    let cap = compute_file_cap(1_000_000, DEFAULT_MAX_FILE_PCT, Some(5_000));
    assert_eq!(cap, 5_000);
}

#[test]
fn file_cap_usize_max_budget() {
    let cap = compute_file_cap(usize::MAX, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, usize::MAX);
}

#[test]
fn file_cap_zero_budget() {
    let cap = compute_file_cap(0, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, 0);
}

// ===========================================================================
// 5. assign_policy
// ===========================================================================

#[test]
fn assign_full_when_under_cap() {
    let (policy, reason) = assign_policy(100, 1_000, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

#[test]
fn assign_head_tail_when_over_cap_plain() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
    assert!(reason.is_some());
    assert!(reason.unwrap().contains("head+tail"));
}

#[test]
fn assign_skip_generated_over_cap() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::Generated]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.unwrap().contains("generated"));
}

#[test]
fn assign_skip_vendored_over_cap() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::Vendored]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.unwrap().contains("vendored"));
}

#[test]
fn assign_skip_data_blob_over_cap() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::DataBlob]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.unwrap().contains("data_blob"));
}

#[test]
fn assign_head_tail_lockfile_over_cap() {
    // Lockfile is NOT in the skip list, so large lockfiles get HeadTail
    let (policy, _) = assign_policy(20_000, 16_000, &[FileClassification::Lockfile]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

#[test]
fn assign_full_at_exact_cap() {
    let (policy, _) = assign_policy(16_000, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
}

// ===========================================================================
// 6. Full pipeline integration
// ===========================================================================

#[test]
fn pipeline_small_source_included() {
    assert_eq!(
        pipeline("src/lib.rs", 500, 100, 128_000),
        InclusionPolicy::Full
    );
}

#[test]
fn pipeline_large_generated_skipped() {
    // Generated + large → Skip
    assert_eq!(
        pipeline("parser.pb.go", 50_000, 5_000, 128_000),
        InclusionPolicy::Skip
    );
}

#[test]
fn pipeline_large_plain_head_tail() {
    assert_eq!(
        pipeline("src/huge.rs", 50_000, 5_000, 128_000),
        InclusionPolicy::HeadTail
    );
}

// ===========================================================================
// 7. Property tests: determinism
// ===========================================================================

mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn classify_is_deterministic(
            path in "[a-z/]{1,30}\\.(rs|py|js|go)",
            tokens in 0usize..100_000,
            lines in 1usize..10_000,
        ) {
            let a = classify_file(&path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
            let b = classify_file(&path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
            prop_assert_eq!(a, b, "classification must be deterministic");
        }

        #[test]
        fn classify_always_sorted(
            path in "[a-z/]{1,30}\\.(rs|py|js|go)",
            tokens in 0usize..100_000,
            lines in 1usize..10_000,
        ) {
            let classes = classify_file(&path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
            for pair in classes.windows(2) {
                prop_assert!(pair[0] <= pair[1], "classifications must be sorted");
            }
        }

        #[test]
        fn assign_policy_deterministic(
            tokens in 0usize..100_000,
            cap in 1usize..50_000,
        ) {
            let (a, _) = assign_policy(tokens, cap, &[]);
            let (b, _) = assign_policy(tokens, cap, &[]);
            prop_assert_eq!(a, b, "assign_policy must be deterministic");
        }

        #[test]
        fn compute_file_cap_deterministic(
            budget in 0usize..10_000_000,
            pct in 0.01f64..1.0,
        ) {
            let a = compute_file_cap(budget, pct, None);
            let b = compute_file_cap(budget, pct, None);
            prop_assert_eq!(a, b);
        }
    }
}
