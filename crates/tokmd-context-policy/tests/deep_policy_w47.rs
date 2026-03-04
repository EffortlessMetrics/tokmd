//! W47 deep tests for `tokmd-context-policy`.
//!
//! Covers: smart excludes (binary, lockfile, node_modules), classification
//! (source vs config vs docs vs generated), inclusion policy (Full, HeadTail,
//! Skip with reasons), token budget constraints, priority ordering, and
//! property-based invariants.

use proptest::prelude::*;
use tokmd_context_policy::{
    DEFAULT_DENSE_THRESHOLD, DEFAULT_MAX_FILE_PCT, DEFAULT_MAX_FILE_TOKENS, assign_policy,
    classify_file, compute_file_cap, is_spine_file, smart_exclude_reason,
};
use tokmd_types::{FileClassification, InclusionPolicy};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run the full policy pipeline and return (policy, reason).
fn pipeline_full(
    path: &str,
    tokens: usize,
    lines: usize,
    budget: usize,
) -> (InclusionPolicy, Option<String>) {
    let classes = classify_file(path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
    let cap = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, None);
    assign_policy(tokens, cap, &classes)
}

fn pipeline(path: &str, tokens: usize, lines: usize, budget: usize) -> InclusionPolicy {
    pipeline_full(path, tokens, lines, budget).0
}

// ===========================================================================
// 1. Smart excludes: binary files, lockfiles, node_modules
// ===========================================================================

#[test]
fn smart_exclude_all_lockfiles() {
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
        assert_eq!(
            smart_exclude_reason(lf),
            Some("lockfile"),
            "expected lockfile for {lf}"
        );
    }
}

#[test]
fn smart_exclude_lockfiles_nested_in_directories() {
    assert_eq!(
        smart_exclude_reason("deep/nested/Cargo.lock"),
        Some("lockfile")
    );
    assert_eq!(smart_exclude_reason("apps/web/yarn.lock"), Some("lockfile"));
}

#[test]
fn smart_exclude_minified_js_and_css() {
    assert_eq!(smart_exclude_reason("dist/bundle.min.js"), Some("minified"));
    assert_eq!(
        smart_exclude_reason("static/styles.min.css"),
        Some("minified")
    );
}

#[test]
fn smart_exclude_sourcemaps() {
    assert_eq!(smart_exclude_reason("dist/app.js.map"), Some("sourcemap"));
    assert_eq!(
        smart_exclude_reason("build/style.css.map"),
        Some("sourcemap")
    );
}

#[test]
fn smart_exclude_none_for_normal_source_files() {
    let normals = [
        "src/main.rs",
        "lib/utils.py",
        "app/controller.go",
        "README.md",
        "Cargo.toml",
        "node_modules/react/index.js",
    ];
    // Note: node_modules is classified by classify_file, not smart_exclude_reason
    for path in normals {
        let reason = smart_exclude_reason(path);
        if reason.is_some() {
            panic!("unexpected exclude reason for {path}: {reason:?}");
        }
    }
}

// ===========================================================================
// 2. Classification: source vs config vs docs vs generated
// ===========================================================================

#[test]
fn classify_source_file_no_classifications() {
    let classes = classify_file("src/main.rs", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(
        classes.is_empty(),
        "plain source file should have no classifications"
    );
}

#[test]
fn classify_lockfile_detected() {
    let classes = classify_file("Cargo.lock", 5000, 200, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Lockfile));
}

#[test]
fn classify_minified_detected() {
    let classes = classify_file("dist/app.min.js", 50_000, 1, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Minified));
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn classify_generated_protobuf() {
    let classes = classify_file("api/service.pb.go", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_generated_dart_freezed() {
    let classes = classify_file("lib/model.freezed.dart", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_vendored_node_modules() {
    let classes = classify_file(
        "node_modules/react/index.js",
        500,
        100,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Vendored));
}

#[test]
fn classify_vendored_vendor_dir() {
    let classes = classify_file("vendor/sqlite/sqlite3.c", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Vendored));
}

#[test]
fn classify_fixture_directories() {
    let fixtures = [
        "tests/fixtures/sample.json",
        "testdata/input.txt",
        "test_data/config.yaml",
        "__snapshots__/Component.snap",
        "golden/expected.txt",
    ];
    for path in fixtures {
        let classes = classify_file(path, 500, 100, DEFAULT_DENSE_THRESHOLD);
        assert!(
            classes.contains(&FileClassification::Fixture),
            "expected Fixture for {path}, got {classes:?}"
        );
    }
}

#[test]
fn classify_data_blob_high_token_density() {
    let classes = classify_file("data/big.json", 100_000, 10, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn classify_no_data_blob_normal_density() {
    let classes = classify_file("src/lib.rs", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(!classes.contains(&FileClassification::DataBlob));
}

#[test]
fn classify_sourcemap_detected() {
    let classes = classify_file("dist/app.js.map", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Sourcemap));
}

#[test]
fn classify_multiple_classifications_on_one_file() {
    // vendor + minified + data blob
    let classes = classify_file(
        "vendor/lib/react.min.js",
        100_000,
        1,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::Minified));
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn classify_results_sorted_and_deduped() {
    let classes = classify_file(
        "vendor/lib/react.min.js",
        100_000,
        1,
        DEFAULT_DENSE_THRESHOLD,
    );
    let mut sorted = classes.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(classes, sorted);
}

// ===========================================================================
// 3. Inclusion policy: Full, HeadTail, Skip with reasons
// ===========================================================================

#[test]
fn policy_full_for_small_source_file() {
    assert_eq!(
        pipeline("src/lib.rs", 500, 100, 128_000),
        InclusionPolicy::Full
    );
}

#[test]
fn policy_head_tail_for_oversized_regular_file() {
    assert_eq!(
        pipeline("src/big_module.rs", 20_000, 2_000, 100_000),
        InclusionPolicy::HeadTail
    );
}

#[test]
fn policy_skip_for_generated_exceeding_cap() {
    assert_eq!(
        pipeline("api/service.pb.go", 40_000, 8_000, 100_000),
        InclusionPolicy::Skip
    );
}

#[test]
fn policy_skip_for_vendored_exceeding_cap() {
    assert_eq!(
        pipeline("vendor/lib/big.c", 40_000, 8_000, 100_000),
        InclusionPolicy::Skip
    );
}

#[test]
fn policy_skip_for_data_blob_exceeding_cap() {
    assert_eq!(
        pipeline("data/huge.json", 100_000, 5, 100_000),
        InclusionPolicy::Skip
    );
}

#[test]
fn policy_head_tail_for_fixture_exceeding_cap() {
    // Fixtures are NOT in the skip class
    assert_eq!(
        pipeline("tests/fixtures/big.json", 20_000, 5_000, 100_000),
        InclusionPolicy::HeadTail
    );
}

#[test]
fn policy_skip_reason_mentions_classification() {
    let (policy, reason) = pipeline_full("api/types.pb.go", 40_000, 8_000, 100_000);
    assert_eq!(policy, InclusionPolicy::Skip);
    let reason_str = reason.unwrap();
    assert!(
        reason_str.contains("generated"),
        "reason should mention 'generated': {reason_str}"
    );
}

#[test]
fn policy_head_tail_reason_mentions_cap() {
    let (policy, reason) = pipeline_full("src/huge.rs", 20_000, 2_000, 100_000);
    assert_eq!(policy, InclusionPolicy::HeadTail);
    let reason_str = reason.unwrap();
    assert!(
        reason_str.contains("head+tail"),
        "reason should mention 'head+tail': {reason_str}"
    );
}

#[test]
fn policy_full_has_no_reason() {
    let (policy, reason) = pipeline_full("src/lib.rs", 500, 100, 128_000);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

// ===========================================================================
// 4. Token budget: files selected within budget constraints
// ===========================================================================

#[test]
fn compute_file_cap_default_values() {
    let cap = compute_file_cap(128_000, DEFAULT_MAX_FILE_PCT, None);
    // 128_000 * 0.15 = 19_200, min(19_200, 16_000) = 16_000
    assert_eq!(cap, DEFAULT_MAX_FILE_TOKENS);
}

#[test]
fn compute_file_cap_small_budget_uses_pct() {
    let cap = compute_file_cap(10_000, DEFAULT_MAX_FILE_PCT, None);
    // 10_000 * 0.15 = 1_500, min(1_500, 16_000) = 1_500
    assert_eq!(cap, 1_500);
}

#[test]
fn compute_file_cap_with_explicit_hard_cap() {
    let cap = compute_file_cap(128_000, DEFAULT_MAX_FILE_PCT, Some(8_000));
    // 128_000 * 0.15 = 19_200, min(19_200, 8_000) = 8_000
    assert_eq!(cap, 8_000);
}

#[test]
fn compute_file_cap_unbounded_budget() {
    let cap = compute_file_cap(usize::MAX, DEFAULT_MAX_FILE_PCT, Some(16_000));
    assert_eq!(cap, usize::MAX);
}

#[test]
fn zero_budget_yields_zero_cap() {
    let cap = compute_file_cap(0, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, 0);
}

#[test]
fn budget_exactly_one_file() {
    // Budget equals exactly the file's tokens → should be Full
    let tokens = 500;
    let budget = tokens; // cap = 500 * 0.15 = 75
    let policy = pipeline("src/lib.rs", tokens, 100, budget);
    // cap < tokens → HeadTail for regular file
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

#[test]
fn budget_large_enough_for_file() {
    // Budget large enough that file fits under cap
    let tokens = 500;
    let budget = 100_000;
    let policy = pipeline("src/lib.rs", tokens, 100, budget);
    assert_eq!(policy, InclusionPolicy::Full);
}

// ===========================================================================
// 5. Priority ordering: spine files first
// ===========================================================================

#[test]
fn spine_readme_detected() {
    assert!(is_spine_file("README.md"));
    assert!(is_spine_file("README"));
    assert!(is_spine_file("README.rst"));
    assert!(is_spine_file("README.txt"));
}

#[test]
fn spine_project_manifests_detected() {
    assert!(is_spine_file("Cargo.toml"));
    assert!(is_spine_file("package.json"));
    assert!(is_spine_file("pyproject.toml"));
    assert!(is_spine_file("go.mod"));
}

#[test]
fn spine_docs_architecture_detected() {
    assert!(is_spine_file("docs/architecture.md"));
    assert!(is_spine_file("docs/design.md"));
    assert!(is_spine_file("nested/docs/architecture.md"));
}

#[test]
fn spine_config_files_detected() {
    assert!(is_spine_file("tokmd.toml"));
    assert!(is_spine_file("cockpit.toml"));
}

#[test]
fn non_spine_files_rejected() {
    assert!(!is_spine_file("src/main.rs"));
    assert!(!is_spine_file("lib/utils.py"));
    assert!(!is_spine_file("docs/random.md"));
}

// ===========================================================================
// 6. Edge cases
// ===========================================================================

#[test]
fn classify_zero_tokens_zero_lines() {
    let classes = classify_file("empty.rs", 0, 0, DEFAULT_DENSE_THRESHOLD);
    // 0 / max(0,1) = 0 which is <= threshold
    assert!(!classes.contains(&FileClassification::DataBlob));
}

#[test]
fn classify_empty_path() {
    let classes = classify_file("", 500, 100, DEFAULT_DENSE_THRESHOLD);
    // Empty path shouldn't match any patterns
    assert!(classes.is_empty());
}

#[test]
fn assign_policy_zero_cap_everything_oversized() {
    let (policy, reason) = assign_policy(1, 0, &[]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
    assert!(reason.is_some());
}

#[test]
fn assign_policy_zero_tokens_always_full() {
    let (policy, reason) = assign_policy(0, 0, &[FileClassification::Generated]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

#[test]
fn assign_policy_tokens_equal_cap_is_full() {
    let (policy, reason) = assign_policy(100, 100, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

// ===========================================================================
// 7. Property: excluded files never included, tokens never exceed budget
// ===========================================================================

proptest! {
    #[test]
    fn excluded_files_never_get_full_policy(
        tokens in 1usize..200_000,
        cap in 0usize..200_000,
        class in prop::sample::select(vec![
            FileClassification::Generated,
            FileClassification::DataBlob,
            FileClassification::Vendored,
        ]),
    ) {
        let (policy, _) = assign_policy(tokens, cap, &[class]);
        if tokens > cap {
            prop_assert_ne!(policy, InclusionPolicy::Full);
        }
    }

    #[test]
    fn total_tokens_capped_by_budget(
        budget in 1usize..1_000_000,
        pct in 0.01f64..1.0,
    ) {
        let cap = compute_file_cap(budget, pct, None);
        prop_assert!(cap <= budget, "cap {} > budget {}", cap, budget);
    }

    #[test]
    fn classify_is_always_sorted_deduped(
        tokens in 0usize..200_000,
        lines in 0usize..20_000,
    ) {
        let classes = classify_file("vendor/lib/react.min.js", tokens, lines, DEFAULT_DENSE_THRESHOLD);
        let mut sorted = classes.clone();
        sorted.sort();
        sorted.dedup();
        prop_assert_eq!(classes, sorted);
    }

    #[test]
    fn spine_detection_is_deterministic(path in "[a-zA-Z/._]{0,64}") {
        let a = is_spine_file(&path);
        let b = is_spine_file(&path);
        prop_assert_eq!(a, b);
    }

    #[test]
    fn smart_exclude_deterministic(path in "[a-zA-Z0-9/._ -]{0,128}") {
        let a = smart_exclude_reason(&path);
        let b = smart_exclude_reason(&path);
        prop_assert_eq!(a, b);
    }
}
