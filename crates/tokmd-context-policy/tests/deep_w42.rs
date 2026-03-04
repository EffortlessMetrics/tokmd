//! Wave 42 deep tests for context-policy crate.
//!
//! Covers smart-exclude matching, file classification, inclusion policy,
//! budget allocation, priority ordering, and edge cases.

use tokmd_context_policy::*;
use tokmd_types::{FileClassification, InclusionPolicy};

// ── Smart-exclude: lockfiles ──────────────────────────────────────────

#[test]
fn smart_exclude_detects_cargo_lock() {
    assert_eq!(smart_exclude_reason("Cargo.lock"), Some("lockfile"));
}

#[test]
fn smart_exclude_detects_package_lock_json() {
    assert_eq!(smart_exclude_reason("package-lock.json"), Some("lockfile"));
}

#[test]
fn smart_exclude_detects_yarn_lock() {
    assert_eq!(smart_exclude_reason("yarn.lock"), Some("lockfile"));
}

#[test]
fn smart_exclude_detects_poetry_lock() {
    assert_eq!(smart_exclude_reason("poetry.lock"), Some("lockfile"));
}

#[test]
fn smart_exclude_detects_go_sum() {
    assert_eq!(smart_exclude_reason("go.sum"), Some("lockfile"));
}

#[test]
fn smart_exclude_detects_lockfile_in_nested_path() {
    assert_eq!(
        smart_exclude_reason("some/deep/path/Cargo.lock"),
        Some("lockfile")
    );
}

// ── Smart-exclude: minified / sourcemaps ──────────────────────────────

#[test]
fn smart_exclude_detects_minified_js() {
    assert_eq!(smart_exclude_reason("dist/app.min.js"), Some("minified"));
}

#[test]
fn smart_exclude_detects_minified_css() {
    assert_eq!(
        smart_exclude_reason("assets/style.min.css"),
        Some("minified")
    );
}

#[test]
fn smart_exclude_detects_js_sourcemap() {
    assert_eq!(
        smart_exclude_reason("dist/bundle.js.map"),
        Some("sourcemap")
    );
}

#[test]
fn smart_exclude_detects_css_sourcemap() {
    assert_eq!(
        smart_exclude_reason("dist/style.css.map"),
        Some("sourcemap")
    );
}

#[test]
fn smart_exclude_returns_none_for_normal_file() {
    assert_eq!(smart_exclude_reason("src/main.rs"), None);
}

#[test]
fn smart_exclude_returns_none_for_regular_js() {
    assert_eq!(smart_exclude_reason("src/app.js"), None);
}

// ── Spine file detection ──────────────────────────────────────────────

#[test]
fn spine_matches_readme_md() {
    assert!(is_spine_file("README.md"));
}

#[test]
fn spine_matches_readme_plain() {
    assert!(is_spine_file("README"));
}

#[test]
fn spine_matches_cargo_toml() {
    assert!(is_spine_file("Cargo.toml"));
}

#[test]
fn spine_matches_nested_architecture_doc() {
    assert!(is_spine_file("myproject/docs/architecture.md"));
}

#[test]
fn spine_matches_package_json() {
    assert!(is_spine_file("package.json"));
}

#[test]
fn spine_matches_pyproject_toml() {
    assert!(is_spine_file("pyproject.toml"));
}

#[test]
fn spine_does_not_match_random_source_file() {
    assert!(!is_spine_file("src/lib.rs"));
}

#[test]
fn spine_does_not_match_partial_name() {
    assert!(!is_spine_file("src/README.md.bak"));
}

// ── File classification ───────────────────────────────────────────────

#[test]
fn classify_lockfile() {
    let classes = classify_file("Cargo.lock", 100, 50, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Lockfile));
}

#[test]
fn classify_minified_js() {
    let classes = classify_file("dist/app.min.js", 100, 50, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Minified));
}

#[test]
fn classify_sourcemap() {
    let classes = classify_file("dist/app.js.map", 100, 50, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Sourcemap));
}

#[test]
fn classify_generated_protobuf_go() {
    let classes = classify_file("api/service.pb.go", 200, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_generated_protobuf_rs() {
    let classes = classify_file("src/proto/types.pb.rs", 200, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_generated_python_pb2() {
    let classes = classify_file("gen/service_pb2.py", 200, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_generated_node_types_json() {
    let classes = classify_file("src/node-types.json", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_vendored_dir() {
    let classes = classify_file("vendor/lib/foo.go", 100, 50, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Vendored));
}

#[test]
fn classify_third_party_dir() {
    let classes = classify_file(
        "third_party/zlib/inflate.c",
        100,
        50,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Vendored));
}

#[test]
fn classify_node_modules() {
    let classes = classify_file(
        "node_modules/lodash/index.js",
        100,
        50,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Vendored));
}

#[test]
fn classify_fixture_dir() {
    let classes = classify_file("fixtures/sample.json", 100, 50, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));
}

#[test]
fn classify_testdata_dir() {
    let classes = classify_file("testdata/input.txt", 100, 50, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));
}

#[test]
fn classify_snapshots_dir() {
    let classes = classify_file("__snapshots__/foo.snap", 100, 50, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));
}

#[test]
fn classify_dense_blob() {
    // 10_000 tokens / 10 lines = 1000 tokens per line (>> 50 threshold)
    let classes = classify_file("data/big.json", 10_000, 10, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn classify_normal_file_returns_empty() {
    let classes = classify_file("src/main.rs", 200, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(
        classes.is_empty(),
        "expected no classifications for normal code file"
    );
}

#[test]
fn classify_multiple_labels_generated_and_blob() {
    // node-types.json that is also extremely dense
    let classes = classify_file("src/node-types.json", 50_000, 5, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn classify_zero_lines_avoids_division_by_zero() {
    // 0 lines should be treated as 1 to avoid panic
    let classes = classify_file("empty.rs", 100, 0, DEFAULT_DENSE_THRESHOLD);
    // 100 tokens / 1 effective line = 100, which exceeds 50
    assert!(classes.contains(&FileClassification::DataBlob));
}

// ── File cap computation ──────────────────────────────────────────────

#[test]
fn file_cap_respects_percentage() {
    // 100_000 budget * 0.15 = 15_000, hard cap 16_000 → min(15_000, 16_000) = 15_000
    let cap = compute_file_cap(100_000, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, 15_000);
}

#[test]
fn file_cap_respects_hard_cap() {
    // 200_000 budget * 0.15 = 30_000, hard cap 16_000 → min(30_000, 16_000) = 16_000
    let cap = compute_file_cap(200_000, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, DEFAULT_MAX_FILE_TOKENS);
}

#[test]
fn file_cap_custom_hard_cap() {
    let cap = compute_file_cap(100_000, DEFAULT_MAX_FILE_PCT, Some(10_000));
    assert_eq!(cap, 10_000);
}

#[test]
fn file_cap_unlimited_budget_returns_max() {
    let cap = compute_file_cap(usize::MAX, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, usize::MAX);
}

#[test]
fn file_cap_zero_budget() {
    let cap = compute_file_cap(0, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, 0);
}

// ── Inclusion policy assignment ───────────────────────────────────────

#[test]
fn policy_full_when_under_cap() {
    let (policy, reason) = assign_policy(5_000, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

#[test]
fn policy_full_when_exactly_at_cap() {
    let (policy, reason) = assign_policy(16_000, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

#[test]
fn policy_head_tail_when_over_cap_normal_file() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
    assert!(reason.is_some());
    assert!(reason.unwrap().contains("head+tail"));
}

#[test]
fn policy_skip_generated_over_cap() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::Generated]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.unwrap().contains("generated"));
}

#[test]
fn policy_skip_vendored_over_cap() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::Vendored]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.unwrap().contains("vendored"));
}

#[test]
fn policy_skip_data_blob_over_cap() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::DataBlob]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.unwrap().contains("data_blob"));
}

#[test]
fn policy_head_tail_lockfile_over_cap() {
    // Lockfiles are NOT in the skip list, so they get HeadTail
    let (policy, _) = assign_policy(20_000, 16_000, &[FileClassification::Lockfile]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

#[test]
fn policy_head_tail_fixture_over_cap() {
    // Fixtures are NOT in the skip list
    let (policy, _) = assign_policy(20_000, 16_000, &[FileClassification::Fixture]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

#[test]
fn policy_skip_reason_contains_token_counts() {
    let (_, reason) = assign_policy(25_000, 16_000, &[FileClassification::Generated]);
    let r = reason.unwrap();
    assert!(
        r.contains("25000"),
        "reason should contain token count: {r}"
    );
    assert!(r.contains("16000"), "reason should contain cap: {r}");
}

#[test]
fn policy_multiple_classifications_combined_in_reason() {
    let (policy, reason) = assign_policy(
        20_000,
        16_000,
        &[FileClassification::Generated, FileClassification::DataBlob],
    );
    assert_eq!(policy, InclusionPolicy::Skip);
    let r = reason.unwrap();
    assert!(r.contains("generated"));
    assert!(r.contains("data_blob"));
}

// ── Edge cases ────────────────────────────────────────────────────────

#[test]
fn classify_backslash_paths_normalized() {
    // Windows-style path should still match vendor/
    let classes = classify_file("vendor\\lib\\foo.go", 100, 50, DEFAULT_DENSE_THRESHOLD);
    assert!(
        classes.contains(&FileClassification::Vendored),
        "backslash paths should be normalized: {:?}",
        classes
    );
}

#[test]
fn spine_backslash_path_normalized() {
    assert!(is_spine_file("project\\README.md"));
}

#[test]
fn smart_exclude_empty_string() {
    assert_eq!(smart_exclude_reason(""), None);
}

#[test]
fn classify_empty_path() {
    let classes = classify_file("", 0, 0, DEFAULT_DENSE_THRESHOLD);
    // Empty path with 0 tokens / 1 effective line = 0 tokens per line
    assert!(!classes.contains(&FileClassification::DataBlob));
}

#[test]
fn classify_classes_are_sorted_and_deduped() {
    // A file that would hit multiple categories — verify sorting + dedup
    let classes = classify_file(
        "vendor/data/node-types.json",
        50_000,
        5,
        DEFAULT_DENSE_THRESHOLD,
    );
    // Should contain Vendored, Generated, DataBlob — all sorted
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::Generated));
    assert!(classes.contains(&FileClassification::DataBlob));
    // Verify sorted
    for w in classes.windows(2) {
        assert!(w[0] <= w[1], "classifications should be sorted");
    }
}

#[test]
fn file_cap_small_budget() {
    // Very small budget: 100 * 0.15 = 15 (floored), hard cap 16_000 → 15
    let cap = compute_file_cap(100, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, 15);
}

#[test]
fn policy_zero_tokens_under_any_cap() {
    let (policy, reason) = assign_policy(0, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}
