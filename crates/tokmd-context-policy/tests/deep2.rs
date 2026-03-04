//! Deep integration tests (batch 2) for context-policy crate.
//!
//! Covers: advanced smart-exclude edge cases, classification interaction matrix,
//! dense blob precision, file cap arithmetic saturation, assign_policy with
//! mixed skip/non-skip classes, spine file path normalization, and full
//! pipeline determinism under permutation.

use tokmd_context_policy::{
    DEFAULT_DENSE_THRESHOLD, DEFAULT_MAX_FILE_PCT, DEFAULT_MAX_FILE_TOKENS, assign_policy,
    classify_file, compute_file_cap, is_spine_file, smart_exclude_reason,
};
use tokmd_types::{FileClassification, InclusionPolicy};

// ===========================================================================
// 1. Smart exclude — advanced edge cases
// ===========================================================================

#[test]
fn smart_exclude_composer_lock_detected() {
    assert_eq!(smart_exclude_reason("composer.lock"), Some("lockfile"));
}

#[test]
fn smart_exclude_gemfile_lock_detected() {
    assert_eq!(smart_exclude_reason("Gemfile.lock"), Some("lockfile"));
}

#[test]
fn smart_exclude_go_sum_detected() {
    assert_eq!(smart_exclude_reason("go.sum"), Some("lockfile"));
}

#[test]
fn smart_exclude_pipfile_lock_detected() {
    assert_eq!(smart_exclude_reason("Pipfile.lock"), Some("lockfile"));
}

#[test]
fn smart_exclude_poetry_lock_detected() {
    assert_eq!(smart_exclude_reason("poetry.lock"), Some("lockfile"));
}

#[test]
fn smart_exclude_lockfile_in_deep_monorepo_path() {
    assert_eq!(
        smart_exclude_reason("apps/frontend/packages/ui/package-lock.json"),
        Some("lockfile")
    );
}

#[test]
fn smart_exclude_minified_js_deep_path() {
    assert_eq!(
        smart_exclude_reason("dist/assets/js/vendor.min.js"),
        Some("minified")
    );
}

#[test]
fn smart_exclude_non_map_extension_not_matched() {
    // .map alone without .js or .css prefix
    assert_eq!(smart_exclude_reason("data.map"), None);
}

#[test]
fn smart_exclude_dot_min_in_directory_name_not_matched() {
    // The file itself is not minified; directory has ".min" in name
    assert_eq!(smart_exclude_reason("dist.min/app.js"), None);
}

#[test]
fn smart_exclude_case_sensitivity_lockfile() {
    // Lockfile names are case-sensitive
    assert_eq!(smart_exclude_reason("cargo.lock"), None);
    assert_eq!(smart_exclude_reason("CARGO.LOCK"), None);
}

#[test]
fn smart_exclude_path_with_only_slashes() {
    assert_eq!(smart_exclude_reason("/"), None);
    assert_eq!(smart_exclude_reason("///"), None);
}

// ===========================================================================
// 2. Classification interaction matrix
// ===========================================================================

#[test]
fn classify_lockfile_and_sourcemap_simultaneously() {
    // A file named "Cargo.lock" can't simultaneously be a sourcemap,
    // but verify classify doesn't double-tag when it shouldn't.
    let classes = classify_file("Cargo.lock", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Lockfile));
    assert!(!classes.contains(&FileClassification::Sourcemap));
}

#[test]
fn classify_fixture_plus_dense_blob() {
    let classes = classify_file("testdata/huge.json", 10_000, 1, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn classify_vendored_plus_lockfile() {
    let classes = classify_file(
        "vendor/dep/package-lock.json",
        500,
        100,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::Lockfile));
}

#[test]
fn classify_node_modules_lockfile() {
    let classes = classify_file(
        "node_modules/pkg/yarn.lock",
        500,
        100,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::Lockfile));
}

#[test]
fn classify_generated_inside_golden_dir() {
    let classes = classify_file("golden/output.pb.go", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_minified_in_vendor() {
    let classes = classify_file(
        "vendor/jquery/jquery.min.js",
        80_000,
        1,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::Minified));
    assert!(classes.contains(&FileClassification::DataBlob));
    assert_eq!(classes.len(), 3);
}

#[test]
fn classify_sourcemap_in_fixtures() {
    let classes = classify_file("fixtures/output.css.map", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));
    assert!(classes.contains(&FileClassification::Sourcemap));
}

#[test]
fn classify_snapshot_directory() {
    let classes = classify_file(
        "__snapshots__/Login.test.tsx.snap",
        500,
        100,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Fixture));
}

#[test]
fn classify_no_duplicates_in_output() {
    // Force a path that could match multiple patterns
    let classes = classify_file(
        "vendor/third_party/node_modules/pkg.min.js",
        100_000,
        1,
        DEFAULT_DENSE_THRESHOLD,
    );
    // Vendored should appear exactly once even though multiple vendor patterns match
    let vendored_count = classes
        .iter()
        .filter(|c| **c == FileClassification::Vendored)
        .count();
    assert_eq!(vendored_count, 1, "Vendored should be deduped");
}

// ===========================================================================
// 3. Dense blob precision
// ===========================================================================

#[test]
fn dense_blob_exact_ratio_boundary_with_many_lines() {
    // 5000 tokens / 100 lines = 50.0 → NOT > 50.0
    let classes = classify_file("data.csv", 5000, 100, 50.0);
    assert!(!classes.contains(&FileClassification::DataBlob));
}

#[test]
fn dense_blob_one_above_with_many_lines() {
    // 5001 tokens / 100 lines = 50.01 > 50.0
    let classes = classify_file("data.csv", 5001, 100, 50.0);
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn dense_blob_with_very_low_threshold() {
    // threshold = 1.0, 2 tokens / 1 line = 2.0 > 1.0
    let classes = classify_file("file.txt", 2, 1, 1.0);
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn dense_blob_with_very_high_threshold() {
    // threshold = 10000.0, 9999 tokens / 1 line = 9999.0 → NOT flagged
    let classes = classify_file("file.txt", 9999, 1, 10_000.0);
    assert!(!classes.contains(&FileClassification::DataBlob));
}

#[test]
fn dense_blob_large_file_normal_density() {
    // 100_000 tokens / 10_000 lines = 10.0 → NOT > 50.0
    let classes = classify_file("big_source.rs", 100_000, 10_000, DEFAULT_DENSE_THRESHOLD);
    assert!(!classes.contains(&FileClassification::DataBlob));
}

#[test]
fn dense_blob_single_line_file() {
    // 51 tokens / 1 line = 51.0 > 50.0
    let classes = classify_file("oneliner.js", 51, 1, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::DataBlob));
}

// ===========================================================================
// 4. File cap arithmetic — saturation and edge cases
// ===========================================================================

#[test]
fn file_cap_pct_zero_returns_zero() {
    let cap = compute_file_cap(100_000, 0.0, Some(16_000));
    assert_eq!(cap, 0);
}

#[test]
fn file_cap_hard_cap_zero_always_zero() {
    let cap = compute_file_cap(100_000, 0.15, Some(0));
    assert_eq!(cap, 0);
}

#[test]
fn file_cap_both_zero() {
    let cap = compute_file_cap(0, 0.0, Some(0));
    assert_eq!(cap, 0);
}

#[test]
fn file_cap_large_hard_cap_pct_dominates() {
    // budget=100_000, pct=0.10 → 10_000, hard_cap=1_000_000 → 10_000
    let cap = compute_file_cap(100_000, 0.10, Some(1_000_000));
    assert_eq!(cap, 10_000);
}

#[test]
fn file_cap_exact_pct_equals_hard_cap() {
    // budget=100_000, pct=0.16 → 16_000, hard_cap=16_000 → 16_000
    let cap = compute_file_cap(100_000, 0.16, Some(16_000));
    assert_eq!(cap, 16_000);
}

#[test]
fn file_cap_usize_max_budget_short_circuits() {
    // With usize::MAX budget, should return usize::MAX regardless of pct/hard_cap
    let cap = compute_file_cap(usize::MAX, 0.01, Some(100));
    assert_eq!(cap, usize::MAX);
}

#[test]
fn file_cap_default_constants_produce_expected_cap_at_128k() {
    let cap = compute_file_cap(128_000, DEFAULT_MAX_FILE_PCT, Some(DEFAULT_MAX_FILE_TOKENS));
    // 128_000 * 0.15 = 19_200, min(19_200, 16_000) = 16_000
    assert_eq!(cap, 16_000);
}

#[test]
fn file_cap_small_budget_100_tokens() {
    let cap = compute_file_cap(100, DEFAULT_MAX_FILE_PCT, None);
    // 100 * 0.15 = 15, min(15, 16_000) = 15
    assert_eq!(cap, 15);
}

// ===========================================================================
// 5. assign_policy — mixed classification edge cases
// ===========================================================================

#[test]
fn assign_policy_empty_classifications_under_cap_is_full() {
    let (policy, reason) = assign_policy(1000, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

#[test]
fn assign_policy_data_blob_alone_over_cap_skips() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::DataBlob]);
    assert_eq!(policy, InclusionPolicy::Skip);
    let r = reason.unwrap();
    assert!(r.contains("data_blob"));
}

#[test]
fn assign_policy_vendored_alone_over_cap_skips() {
    let (policy, _) = assign_policy(20_000, 16_000, &[FileClassification::Vendored]);
    assert_eq!(policy, InclusionPolicy::Skip);
}

#[test]
fn assign_policy_generated_alone_over_cap_skips() {
    let (policy, _) = assign_policy(20_000, 16_000, &[FileClassification::Generated]);
    assert_eq!(policy, InclusionPolicy::Skip);
}

#[test]
fn assign_policy_fixture_plus_data_blob_over_cap_skips() {
    // DataBlob is in skip list, so presence of DataBlob triggers skip
    let (policy, reason) = assign_policy(
        20_000,
        16_000,
        &[FileClassification::Fixture, FileClassification::DataBlob],
    );
    assert_eq!(policy, InclusionPolicy::Skip);
    let r = reason.unwrap();
    assert!(r.contains("fixture"));
    assert!(r.contains("data_blob"));
}

#[test]
fn assign_policy_lockfile_plus_minified_over_cap_head_tails() {
    // Neither lockfile nor minified are in skip list
    let (policy, _) = assign_policy(
        20_000,
        16_000,
        &[FileClassification::Lockfile, FileClassification::Minified],
    );
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

#[test]
fn assign_policy_reason_contains_token_counts() {
    let (_, reason) = assign_policy(25_000, 10_000, &[FileClassification::Generated]);
    let r = reason.unwrap();
    assert!(r.contains("25000"));
    assert!(r.contains("10000"));
}

#[test]
fn assign_policy_head_tail_reason_format() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::Fixture]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
    let r = reason.unwrap();
    assert!(r.contains("head+tail"));
}

#[test]
fn assign_policy_usize_max_cap_always_full() {
    let (policy, reason) = assign_policy(
        1_000_000,
        usize::MAX,
        &[FileClassification::Generated, FileClassification::DataBlob],
    );
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

#[test]
fn assign_policy_tokens_equal_one_cap_zero_generated_skips() {
    let (policy, _) = assign_policy(1, 0, &[FileClassification::Generated]);
    assert_eq!(policy, InclusionPolicy::Skip);
}

// ===========================================================================
// 6. Spine file — edge cases and normalization
// ===========================================================================

#[test]
fn spine_file_tokmd_toml_at_root() {
    assert!(is_spine_file("tokmd.toml"));
}

#[test]
fn spine_file_tokmd_toml_nested() {
    assert!(is_spine_file("project/tokmd.toml"));
}

#[test]
fn spine_file_package_json_deeply_nested() {
    assert!(is_spine_file("apps/frontend/package.json"));
}

#[test]
fn spine_file_contributing_md() {
    assert!(is_spine_file("CONTRIBUTING.md"));
}

#[test]
fn spine_file_roadmap_at_root() {
    assert!(is_spine_file("ROADMAP.md"));
}

#[test]
fn spine_file_docs_design_nested() {
    assert!(is_spine_file("my/project/docs/design.md"));
}

#[test]
fn spine_file_rejects_similar_but_different_names() {
    assert!(!is_spine_file("README.markdown"));
    assert!(!is_spine_file("Cargo.lock"));
    assert!(!is_spine_file("package-lock.json"));
    assert!(!is_spine_file("design.md")); // without docs/ prefix
}

#[test]
fn spine_file_windows_backslash_normalization() {
    assert!(is_spine_file("project\\Cargo.toml"));
    assert!(is_spine_file("my\\project\\docs\\architecture.md"));
}

// ===========================================================================
// 7. Full pipeline consistency tests
// ===========================================================================

fn full_pipeline(path: &str, tokens: usize, lines: usize, budget: usize) -> InclusionPolicy {
    let classes = classify_file(path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
    let cap = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, None);
    let (policy, _) = assign_policy(tokens, cap, &classes);
    policy
}

#[test]
fn pipeline_normal_file_under_budget_full() {
    assert_eq!(
        full_pipeline("src/utils.rs", 500, 100, 128_000),
        InclusionPolicy::Full
    );
}

#[test]
fn pipeline_large_normal_file_head_tail() {
    assert_eq!(
        full_pipeline("src/engine.rs", 20_000, 2_000, 128_000),
        InclusionPolicy::HeadTail
    );
}

#[test]
fn pipeline_generated_under_cap_full() {
    // Even generated files get Full if under cap
    assert_eq!(
        full_pipeline("api/types.pb.go", 1_000, 200, 128_000),
        InclusionPolicy::Full
    );
}

#[test]
fn pipeline_generated_over_cap_skip() {
    assert_eq!(
        full_pipeline("api/types.pb.go", 40_000, 8_000, 128_000),
        InclusionPolicy::Skip
    );
}

#[test]
fn pipeline_vendored_dense_blob_skip() {
    assert_eq!(
        full_pipeline("vendor/lib/big.min.js", 80_000, 1, 128_000),
        InclusionPolicy::Skip
    );
}

#[test]
fn pipeline_fixture_over_cap_head_tail() {
    // Fixture alone doesn't trigger skip
    assert_eq!(
        full_pipeline("testdata/large_input.txt", 25_000, 5_000, 128_000),
        InclusionPolicy::HeadTail
    );
}

#[test]
fn pipeline_zero_budget_zero_tokens_full() {
    // 0 tokens, cap=0, no classifications → 0 ≤ 0 → Full
    assert_eq!(full_pipeline("empty.rs", 0, 0, 0), InclusionPolicy::Full);
}

// ===========================================================================
// 8. Determinism under repeated invocations
// ===========================================================================

#[test]
fn classify_file_deterministic_10_iterations() {
    let path = "vendor/third_party/generated.pb.rs";
    let first = classify_file(path, 50_000, 100, DEFAULT_DENSE_THRESHOLD);
    for _ in 0..10 {
        let result = classify_file(path, 50_000, 100, DEFAULT_DENSE_THRESHOLD);
        assert_eq!(result, first, "classify_file must be deterministic");
    }
}

#[test]
fn smart_exclude_deterministic_all_lockfiles() {
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
    for lf in lockfiles {
        let a = smart_exclude_reason(lf);
        let b = smart_exclude_reason(lf);
        assert_eq!(a, b, "smart_exclude_reason must be deterministic for {lf}");
    }
}

#[test]
fn full_pipeline_deterministic_across_invocations() {
    let cases = [
        ("src/lib.rs", 500, 100, 128_000),
        ("api/types.pb.go", 40_000, 8_000, 100_000),
        ("vendor/react.min.js", 80_000, 1, 128_000),
        ("testdata/big.json", 25_000, 5_000, 50_000),
    ];
    for (path, tokens, lines, budget) in cases {
        let first = full_pipeline(path, tokens, lines, budget);
        let second = full_pipeline(path, tokens, lines, budget);
        assert_eq!(first, second, "pipeline must be deterministic for {path}");
    }
}

// ===========================================================================
// 9. Classification output sorting invariant
// ===========================================================================

#[test]
fn classify_output_always_sorted() {
    let test_cases = [
        "vendor/lib/react.min.js",
        "fixtures/generated.pb.go",
        "node_modules/@scope/yarn.lock",
        "third-party/data.js.map",
    ];
    for path in test_cases {
        let classes = classify_file(path, 100_000, 1, DEFAULT_DENSE_THRESHOLD);
        let mut sorted = classes.clone();
        sorted.sort();
        assert_eq!(classes, sorted, "classify output must be sorted for {path}");
    }
}

#[test]
fn classify_output_no_duplicates() {
    let test_cases = [
        "vendor/third_party/node_modules/gen.pb.go",
        "fixtures/testdata/golden/file.min.js",
    ];
    for path in test_cases {
        let classes = classify_file(path, 100_000, 1, DEFAULT_DENSE_THRESHOLD);
        let mut deduped = classes.clone();
        deduped.dedup();
        assert_eq!(
            classes, deduped,
            "classify output must have no duplicates for {path}"
        );
    }
}

// ===========================================================================
// 10. Default constant sanity
// ===========================================================================

#[test]
fn default_dense_threshold_is_positive() {
    const {
        assert!(DEFAULT_DENSE_THRESHOLD > 0.0);
    }
}

#[test]
fn default_max_file_pct_between_zero_and_one() {
    const {
        assert!(DEFAULT_MAX_FILE_PCT > 0.0);
    }
    const {
        assert!(DEFAULT_MAX_FILE_PCT < 1.0);
    }
}

#[test]
fn default_max_file_tokens_reasonable() {
    const {
        assert!(DEFAULT_MAX_FILE_TOKENS >= 1_000);
    }
    const {
        assert!(DEFAULT_MAX_FILE_TOKENS <= 1_000_000);
    }
}
