//! Deep tests for tokmd-context-policy crate (W73).

use tokmd_context_policy::{
    assign_policy, classify_file, compute_file_cap, is_spine_file, smart_exclude_reason,
    DEFAULT_DENSE_THRESHOLD, DEFAULT_MAX_FILE_PCT, DEFAULT_MAX_FILE_TOKENS,
};
use tokmd_types::{FileClassification, InclusionPolicy};

// ---------------------------------------------------------------------------
// smart_exclude_reason
// ---------------------------------------------------------------------------

#[test]
fn smart_exclude_cargo_lock() {
    assert_eq!(smart_exclude_reason("Cargo.lock"), Some("lockfile"));
}

#[test]
fn smart_exclude_nested_lockfile() {
    assert_eq!(
        smart_exclude_reason("subdir/package-lock.json"),
        Some("lockfile")
    );
}

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
    for lf in &lockfiles {
        assert_eq!(
            smart_exclude_reason(lf),
            Some("lockfile"),
            "{lf} should be detected as lockfile"
        );
    }
}

#[test]
fn smart_exclude_minified_js() {
    assert_eq!(smart_exclude_reason("dist/app.min.js"), Some("minified"));
}

#[test]
fn smart_exclude_minified_css() {
    assert_eq!(
        smart_exclude_reason("assets/style.min.css"),
        Some("minified")
    );
}

#[test]
fn smart_exclude_sourcemap_js() {
    assert_eq!(smart_exclude_reason("dist/app.js.map"), Some("sourcemap"));
}

#[test]
fn smart_exclude_sourcemap_css() {
    assert_eq!(
        smart_exclude_reason("assets/style.css.map"),
        Some("sourcemap")
    );
}

#[test]
fn smart_exclude_regular_file_none() {
    assert_eq!(smart_exclude_reason("src/main.rs"), None);
    assert_eq!(smart_exclude_reason("README.md"), None);
    assert_eq!(smart_exclude_reason("lib/utils.ts"), None);
}

// ---------------------------------------------------------------------------
// is_spine_file
// ---------------------------------------------------------------------------

#[test]
fn spine_readme_variants() {
    assert!(is_spine_file("README.md"));
    assert!(is_spine_file("README"));
    assert!(is_spine_file("README.rst"));
    assert!(is_spine_file("README.txt"));
}

#[test]
fn spine_manifest_files() {
    assert!(is_spine_file("Cargo.toml"));
    assert!(is_spine_file("package.json"));
    assert!(is_spine_file("pyproject.toml"));
    assert!(is_spine_file("go.mod"));
}

#[test]
fn spine_nested_docs() {
    assert!(is_spine_file("docs/architecture.md"));
    assert!(is_spine_file("docs/design.md"));
    assert!(is_spine_file("myrepo/docs/architecture.md"));
}

#[test]
fn spine_non_spine_files() {
    assert!(!is_spine_file("src/main.rs"));
    assert!(!is_spine_file("tests/integration.rs"));
    assert!(!is_spine_file("docs/random.md"));
}

// ---------------------------------------------------------------------------
// classify_file – individual classifications
// ---------------------------------------------------------------------------

#[test]
fn classify_lockfile() {
    let classes = classify_file("Cargo.lock", 5000, 500, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Lockfile));
    assert!(!classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_minified() {
    let classes = classify_file("dist/app.min.js", 100, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Minified));
}

#[test]
fn classify_sourcemap() {
    let classes = classify_file("dist/app.js.map", 100, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Sourcemap));
}

#[test]
fn classify_generated_patterns() {
    let generated = [
        "src/node-types.json",
        "proto/service.pb.go",
        "proto/msg.pb.rs",
        "gen/api_pb2.py",
        "lib/model.g.dart",
        "lib/model.freezed.dart",
        "src/foo.generated.ts",
    ];
    for path in &generated {
        let classes = classify_file(path, 100, 100, DEFAULT_DENSE_THRESHOLD);
        assert!(
            classes.contains(&FileClassification::Generated),
            "{path} should be classified as Generated"
        );
    }
}

#[test]
fn classify_vendored_dirs() {
    let vendored = [
        "vendor/lib/foo.c",
        "third_party/zlib/zlib.h",
        "third-party/openssl/ssl.c",
        "node_modules/lodash/index.js",
    ];
    for path in &vendored {
        let classes = classify_file(path, 100, 100, DEFAULT_DENSE_THRESHOLD);
        assert!(
            classes.contains(&FileClassification::Vendored),
            "{path} should be classified as Vendored"
        );
    }
}

#[test]
fn classify_fixture_dirs() {
    let fixtures = [
        "fixtures/sample.json",
        "testdata/input.txt",
        "test_data/expected.csv",
        "__snapshots__/foo.snap",
        "golden/expected_output.txt",
    ];
    for path in &fixtures {
        let classes = classify_file(path, 100, 100, DEFAULT_DENSE_THRESHOLD);
        assert!(
            classes.contains(&FileClassification::Fixture),
            "{path} should be classified as Fixture"
        );
    }
}

#[test]
fn classify_data_blob_high_density() {
    // 10000 tokens / 10 lines = 1000 tokens_per_line >> threshold
    let classes = classify_file("data/big.json", 10_000, 10, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn classify_data_blob_below_threshold() {
    // 100 tokens / 100 lines = 1.0, well below threshold
    let classes = classify_file("data/normal.json", 100, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(!classes.contains(&FileClassification::DataBlob));
}

#[test]
fn classify_regular_source_file_empty() {
    let classes = classify_file("src/lib.rs", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.is_empty(), "normal source file should have no classifications");
}

#[test]
fn classify_deduplicates_and_sorts() {
    // A file in vendor/ that also has high density — should get both, sorted & deduped
    let classes = classify_file("vendor/big.js", 50_000, 5, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::DataBlob));
    // Verify sorted (Vendored > DataBlob in the PartialOrd)
    let positions: Vec<_> = classes.iter().collect();
    for w in positions.windows(2) {
        assert!(w[0] <= w[1], "classifications should be sorted");
    }
}

// ---------------------------------------------------------------------------
// compute_file_cap
// ---------------------------------------------------------------------------

#[test]
fn compute_file_cap_default_budget() {
    let cap = compute_file_cap(128_000, DEFAULT_MAX_FILE_PCT, None);
    // 128_000 * 0.15 = 19_200, min with DEFAULT_MAX_FILE_TOKENS (16_000)
    assert_eq!(cap, DEFAULT_MAX_FILE_TOKENS);
}

#[test]
fn compute_file_cap_small_budget() {
    let cap = compute_file_cap(10_000, DEFAULT_MAX_FILE_PCT, None);
    // 10_000 * 0.15 = 1_500, which is < 16_000
    assert_eq!(cap, 1_500);
}

#[test]
fn compute_file_cap_explicit_hard_cap() {
    let cap = compute_file_cap(128_000, DEFAULT_MAX_FILE_PCT, Some(5_000));
    // 128_000 * 0.15 = 19_200, min with 5_000 = 5_000
    assert_eq!(cap, 5_000);
}

#[test]
fn compute_file_cap_unlimited_budget() {
    let cap = compute_file_cap(usize::MAX, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, usize::MAX);
}

// ---------------------------------------------------------------------------
// assign_policy
// ---------------------------------------------------------------------------

#[test]
fn assign_policy_full_when_under_cap() {
    let (policy, reason) = assign_policy(500, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

#[test]
fn assign_policy_head_tail_when_over_cap_normal_file() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
    assert!(reason.is_some());
    assert!(reason.unwrap().contains("head+tail"));
}

#[test]
fn assign_policy_skip_generated_over_cap() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::Generated]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.unwrap().contains("generated"));
}

#[test]
fn assign_policy_skip_data_blob_over_cap() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::DataBlob]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.unwrap().contains("data_blob"));
}

#[test]
fn assign_policy_skip_vendored_over_cap() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::Vendored]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.unwrap().contains("vendored"));
}

#[test]
fn assign_policy_fixture_not_skip_class() {
    // Fixtures over cap get HeadTail, not Skip
    let (policy, _reason) = assign_policy(20_000, 16_000, &[FileClassification::Fixture]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}
