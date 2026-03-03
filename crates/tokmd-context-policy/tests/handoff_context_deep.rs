//! Deep tests for context-policy: smart excludes, classification, file cap,
//! inclusion policy assignment, and spine-file detection.

use tokmd_context_policy::{
    DEFAULT_DENSE_THRESHOLD, DEFAULT_MAX_FILE_PCT, DEFAULT_MAX_FILE_TOKENS, assign_policy,
    classify_file, compute_file_cap, is_spine_file, smart_exclude_reason,
};
use tokmd_types::{FileClassification, InclusionPolicy};

// =============================================================================
// smart_exclude_reason — lockfiles
// =============================================================================

#[test]
fn smart_exclude_all_lockfiles() {
    for lockfile in &[
        "Cargo.lock",
        "package-lock.json",
        "pnpm-lock.yaml",
        "yarn.lock",
        "poetry.lock",
        "Pipfile.lock",
        "go.sum",
        "composer.lock",
        "Gemfile.lock",
    ] {
        assert_eq!(
            smart_exclude_reason(lockfile),
            Some("lockfile"),
            "{lockfile} should be detected as lockfile"
        );
    }
}

#[test]
fn smart_exclude_lockfile_with_dir_prefix() {
    assert_eq!(smart_exclude_reason("path/to/Cargo.lock"), Some("lockfile"));
    assert_eq!(
        smart_exclude_reason("deep/nested/package-lock.json"),
        Some("lockfile")
    );
}

// =============================================================================
// smart_exclude_reason — minified / sourcemaps
// =============================================================================

#[test]
fn smart_exclude_minified_js_and_css() {
    assert_eq!(smart_exclude_reason("app.min.js"), Some("minified"));
    assert_eq!(smart_exclude_reason("styles.min.css"), Some("minified"));
    assert_eq!(smart_exclude_reason("dist/vendor.min.js"), Some("minified"));
}

#[test]
fn smart_exclude_sourcemaps() {
    assert_eq!(smart_exclude_reason("app.js.map"), Some("sourcemap"));
    assert_eq!(smart_exclude_reason("styles.css.map"), Some("sourcemap"));
}

#[test]
fn smart_exclude_normal_files_return_none() {
    assert_eq!(smart_exclude_reason("src/main.rs"), None);
    assert_eq!(smart_exclude_reason("README.md"), None);
    assert_eq!(smart_exclude_reason("Cargo.toml"), None);
    assert_eq!(smart_exclude_reason("app.js"), None);
}

// =============================================================================
// is_spine_file
// =============================================================================

#[test]
fn spine_file_readme_variants() {
    assert!(is_spine_file("README.md"));
    assert!(is_spine_file("README"));
    assert!(is_spine_file("README.rst"));
    assert!(is_spine_file("README.txt"));
}

#[test]
fn spine_file_project_manifests() {
    assert!(is_spine_file("Cargo.toml"));
    assert!(is_spine_file("package.json"));
    assert!(is_spine_file("pyproject.toml"));
    assert!(is_spine_file("go.mod"));
}

#[test]
fn spine_file_docs_paths() {
    assert!(is_spine_file("docs/architecture.md"));
    assert!(is_spine_file("docs/design.md"));
    assert!(is_spine_file("nested/docs/architecture.md"));
}

#[test]
fn spine_file_non_spine_returns_false() {
    assert!(!is_spine_file("src/main.rs"));
    assert!(!is_spine_file("tests/test_lib.rs"));
    assert!(!is_spine_file("random.txt"));
}

// =============================================================================
// classify_file — generated
// =============================================================================

#[test]
fn classify_generated_files() {
    let classes = classify_file("node-types.json", 1000, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));

    let classes = classify_file("schema.pb.rs", 5000, 200, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));

    let classes = classify_file("types_pb2.py", 3000, 150, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
}

// =============================================================================
// classify_file — vendored
// =============================================================================

#[test]
fn classify_vendored_dirs() {
    let classes = classify_file("vendor/dep/lib.rs", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Vendored));

    let classes = classify_file("third_party/lib.c", 800, 200, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Vendored));

    let classes = classify_file(
        "node_modules/pkg/index.js",
        300,
        50,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Vendored));
}

// =============================================================================
// classify_file — fixtures
// =============================================================================

#[test]
fn classify_fixture_dirs() {
    let classes = classify_file("fixtures/data.json", 1000, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));

    let classes = classify_file("testdata/input.txt", 500, 50, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));

    let classes = classify_file("__snapshots__/snap.json", 200, 20, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));
}

// =============================================================================
// classify_file — dense blobs
// =============================================================================

#[test]
fn classify_dense_blob() {
    // 50,000 tokens / 10 lines = 5,000 tokens/line >> threshold
    let classes = classify_file("data.json", 50_000, 10, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn classify_normal_density_not_blob() {
    // 1,000 tokens / 100 lines = 10 tokens/line < threshold
    let classes = classify_file("normal.rs", 1_000, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(!classes.contains(&FileClassification::DataBlob));
}

// =============================================================================
// classify_file — minified / sourcemaps / lockfiles
// =============================================================================

#[test]
fn classify_minified() {
    let classes = classify_file("app.min.js", 100_000, 1, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Minified));
}

#[test]
fn classify_sourcemap() {
    let classes = classify_file("app.js.map", 200_000, 1, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Sourcemap));
}

#[test]
fn classify_lockfile() {
    let classes = classify_file("Cargo.lock", 30_000, 5000, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Lockfile));
}

// =============================================================================
// classify_file — multiple classifications
// =============================================================================

#[test]
fn classify_multiple_classes_sorted_deduped() {
    // vendored + minified + data blob
    let classes = classify_file("vendor/lib.min.js", 100_000, 1, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::Minified));
    assert!(classes.contains(&FileClassification::DataBlob));
    // Verify sorted
    for i in 0..classes.len() - 1 {
        assert!(classes[i] <= classes[i + 1]);
    }
}

// =============================================================================
// compute_file_cap
// =============================================================================

#[test]
fn compute_file_cap_default_values() {
    let cap = compute_file_cap(128_000, DEFAULT_MAX_FILE_PCT, None);
    let pct_cap = (128_000.0 * DEFAULT_MAX_FILE_PCT) as usize;
    assert_eq!(cap, pct_cap.min(DEFAULT_MAX_FILE_TOKENS));
}

#[test]
fn compute_file_cap_pct_dominates_when_small_budget() {
    // 10k * 0.15 = 1500, less than DEFAULT_MAX_FILE_TOKENS (16k)
    let cap = compute_file_cap(10_000, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, 1_500);
}

#[test]
fn compute_file_cap_hard_cap_dominates_when_large_budget() {
    // 1M * 0.15 = 150k, but hard cap is 16k by default
    let cap = compute_file_cap(1_000_000, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, DEFAULT_MAX_FILE_TOKENS);
}

#[test]
fn compute_file_cap_custom_hard_cap() {
    let cap = compute_file_cap(100_000, DEFAULT_MAX_FILE_PCT, Some(50_000));
    let pct_cap = (100_000.0 * DEFAULT_MAX_FILE_PCT) as usize;
    assert_eq!(cap, pct_cap.min(50_000));
}

#[test]
fn compute_file_cap_unlimited_budget() {
    let cap = compute_file_cap(usize::MAX, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, usize::MAX);
}

// =============================================================================
// assign_policy
// =============================================================================

#[test]
fn assign_policy_full_when_under_cap() {
    let (policy, reason) = assign_policy(1_000, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

#[test]
fn assign_policy_head_tail_when_over_cap_no_skip_class() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
    assert!(reason.is_some());
    assert!(reason.unwrap().contains("head+tail"));
}

#[test]
fn assign_policy_skip_when_over_cap_generated() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::Generated]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.unwrap().contains("generated"));
}

#[test]
fn assign_policy_skip_when_over_cap_vendored() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::Vendored]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.unwrap().contains("vendored"));
}

#[test]
fn assign_policy_skip_when_over_cap_data_blob() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::DataBlob]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.unwrap().contains("data_blob"));
}

#[test]
fn assign_policy_head_tail_when_over_cap_fixture() {
    // Fixtures are NOT in skip_classes, so they get HeadTail
    let (policy, _) = assign_policy(20_000, 16_000, &[FileClassification::Fixture]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

#[test]
fn assign_policy_full_at_exact_cap() {
    let (policy, reason) = assign_policy(16_000, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}
