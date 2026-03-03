//! Deep tests for context-policy crate.
//!
//! Covers: additional smart-exclude edge cases, classification corner cases,
//! token budget arithmetic, assign_policy boundary conditions, and
//! cross-function consistency invariants.

use tokmd_context_policy::{
    DEFAULT_DENSE_THRESHOLD, DEFAULT_MAX_FILE_PCT, DEFAULT_MAX_FILE_TOKENS, assign_policy,
    classify_file, compute_file_cap, is_spine_file, smart_exclude_reason,
};
use tokmd_types::{FileClassification, InclusionPolicy};

// ===========================================================================
// 1. Smart excludes — additional edge cases
// ===========================================================================

#[test]
fn smart_exclude_pnpm_lock_is_lockfile() {
    assert_eq!(smart_exclude_reason("pnpm-lock.yaml"), Some("lockfile"));
}

#[test]
fn smart_exclude_nested_lockfile_is_detected() {
    assert_eq!(
        smart_exclude_reason("packages/app/yarn.lock"),
        Some("lockfile")
    );
}

#[test]
fn smart_exclude_lockfile_partial_name_not_matched() {
    // "Cargo.lock.bak" should NOT match (basename is "Cargo.lock.bak")
    assert_eq!(smart_exclude_reason("Cargo.lock.bak"), None);
}

#[test]
fn smart_exclude_min_css_at_root() {
    assert_eq!(smart_exclude_reason("styles.min.css"), Some("minified"));
}

#[test]
fn smart_exclude_css_map_at_root() {
    assert_eq!(smart_exclude_reason("styles.css.map"), Some("sourcemap"));
}

#[test]
fn smart_exclude_non_minified_js_not_matched() {
    assert_eq!(smart_exclude_reason("app.bundle.js"), None);
}

#[test]
fn smart_exclude_empty_path_returns_none() {
    assert_eq!(smart_exclude_reason(""), None);
}

// ===========================================================================
// 2. Classification — simultaneous and corner cases
// ===========================================================================

#[test]
fn classify_fixture_and_generated_simultaneously() {
    // A protobuf file inside a fixture directory
    let classes = classify_file(
        "tests/fixtures/types.pb.go",
        500,
        100,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Fixture));
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_vendored_and_generated_simultaneously() {
    let classes = classify_file(
        "vendor/proto/service.pb.rs",
        500,
        100,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_lockfile_at_any_depth() {
    let classes = classify_file("packages/app/Cargo.lock", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Lockfile));
}

#[test]
fn classify_sourcemap_extension() {
    let classes = classify_file("dist/app.js.map", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Sourcemap));
}

#[test]
fn classify_minified_css() {
    let classes = classify_file("static/style.min.css", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Minified));
}

#[test]
fn classify_generated_dot_pattern_in_middle() {
    // ".generated." in middle of filename
    let classes = classify_file("src/schema.generated.ts", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_freezed_dart() {
    let classes = classify_file("lib/model.freezed.dart", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_g_dart() {
    let classes = classify_file("lib/model.g.dart", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_python_protobuf() {
    let classes = classify_file("proto/types_pb2.py", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_normal_rs_file_empty() {
    let classes = classify_file("src/main.rs", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.is_empty());
}

#[test]
fn classify_node_modules_deeply_nested() {
    let classes = classify_file(
        "node_modules/@scope/pkg/dist/index.js",
        500,
        100,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Vendored));
}

#[test]
fn classify_third_party_hyphen() {
    let classes = classify_file("third-party/lib/foo.c", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Vendored));
}

#[test]
fn classify_test_data_underscore() {
    let classes = classify_file("test_data/sample.json", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));
}

#[test]
fn classify_testdata_no_separator() {
    let classes = classify_file("testdata/config.yaml", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));
}

// ===========================================================================
// 3. Dense blob threshold — edge precision
// ===========================================================================

#[test]
fn dense_blob_just_below_threshold_not_flagged() {
    // 499 tokens / 10 lines = 49.9 < 50.0
    let classes = classify_file("data.json", 499, 10, 50.0);
    assert!(!classes.contains(&FileClassification::DataBlob));
}

#[test]
fn dense_blob_just_above_threshold_flagged() {
    // 501 tokens / 10 lines = 50.1 > 50.0
    let classes = classify_file("data.json", 501, 10, 50.0);
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn dense_blob_with_custom_threshold() {
    // 100 tokens / 10 lines = 10.0; threshold = 5.0 → flagged
    let classes = classify_file("data.bin", 100, 10, 5.0);
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn dense_blob_zero_lines_uses_max_one() {
    // With zero lines, effective = max(0,1) = 1
    // 100 tokens / 1 = 100.0 > 50.0
    let classes = classify_file("blob.bin", 100, 0, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn dense_blob_zero_tokens_zero_lines_not_flagged() {
    let classes = classify_file("empty.txt", 0, 0, DEFAULT_DENSE_THRESHOLD);
    assert!(!classes.contains(&FileClassification::DataBlob));
}

// ===========================================================================
// 4. Token budget — compute_file_cap edge cases
// ===========================================================================

#[test]
fn file_cap_large_budget_capped_by_hard_cap() {
    let cap = compute_file_cap(10_000_000, 0.15, Some(16_000));
    assert_eq!(cap, 16_000);
}

#[test]
fn file_cap_with_one_hundred_pct() {
    // 100% of budget = budget itself, but capped by hard_cap
    let cap = compute_file_cap(50_000, 1.0, Some(16_000));
    assert_eq!(cap, 16_000);
}

#[test]
fn file_cap_with_one_hundred_pct_no_hard_cap() {
    // 100% of budget = 50_000, default hard_cap = 16_000 → 16_000
    let cap = compute_file_cap(50_000, 1.0, None);
    assert_eq!(cap, DEFAULT_MAX_FILE_TOKENS);
}

#[test]
fn file_cap_very_small_pct() {
    // 0.001 * 100_000 = 100
    let cap = compute_file_cap(100_000, 0.001, Some(16_000));
    assert_eq!(cap, 100);
}

#[test]
fn file_cap_budget_one() {
    // 1 * 0.15 = 0.15 → truncated to 0
    let cap = compute_file_cap(1, DEFAULT_MAX_FILE_PCT, None);
    assert_eq!(cap, 0);
}

#[test]
fn file_cap_default_constants_consistent() {
    // Verify DEFAULT_MAX_FILE_PCT and DEFAULT_MAX_FILE_TOKENS are sane
    assert!(DEFAULT_MAX_FILE_PCT > 0.0 && DEFAULT_MAX_FILE_PCT < 1.0);
    assert!(DEFAULT_MAX_FILE_TOKENS > 0);
}

// ===========================================================================
// 5. assign_policy — boundary conditions and multi-class
// ===========================================================================

#[test]
fn assign_policy_zero_tokens_always_full() {
    let (policy, reason) = assign_policy(0, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

#[test]
fn assign_policy_zero_tokens_with_generated_class_full() {
    // Even with Generated class, 0 tokens ≤ any cap → Full
    let (policy, _) = assign_policy(0, 16_000, &[FileClassification::Generated]);
    assert_eq!(policy, InclusionPolicy::Full);
}

#[test]
fn assign_policy_zero_cap_nonzero_tokens_with_skip_class() {
    // tokens=1, cap=0, Generated → Skip
    let (policy, reason) = assign_policy(1, 0, &[FileClassification::Generated]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.is_some());
}

#[test]
fn assign_policy_zero_cap_nonzero_tokens_without_skip_class() {
    // tokens=1, cap=0, no skip classes → HeadTail
    let (policy, reason) = assign_policy(1, 0, &[]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
    assert!(reason.is_some());
}

#[test]
fn assign_policy_multiple_skip_classes_all_named_in_reason() {
    let classes = vec![
        FileClassification::Generated,
        FileClassification::Vendored,
        FileClassification::DataBlob,
    ];
    let (policy, reason) = assign_policy(20_000, 16_000, &classes);
    assert_eq!(policy, InclusionPolicy::Skip);
    let reason = reason.unwrap();
    assert!(reason.contains("generated"));
    assert!(reason.contains("vendored"));
    assert!(reason.contains("data_blob"));
}

#[test]
fn assign_policy_fixture_alone_never_skips() {
    // Fixture is not in the skip list
    let (policy, _) = assign_policy(100_000, 16_000, &[FileClassification::Fixture]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

#[test]
fn assign_policy_lockfile_alone_never_skips() {
    let (policy, _) = assign_policy(100_000, 16_000, &[FileClassification::Lockfile]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

#[test]
fn assign_policy_minified_alone_never_skips() {
    let (policy, _) = assign_policy(100_000, 16_000, &[FileClassification::Minified]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

#[test]
fn assign_policy_sourcemap_alone_never_skips() {
    let (policy, _) = assign_policy(100_000, 16_000, &[FileClassification::Sourcemap]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

// ===========================================================================
// 6. Spine file — additional patterns and edge cases
// ===========================================================================

#[test]
fn spine_file_docs_roadmap() {
    assert!(is_spine_file("docs/ROADMAP.md"));
}

#[test]
fn spine_file_plain_readme_no_extension() {
    assert!(is_spine_file("README"));
}

#[test]
fn spine_file_readme_rst() {
    assert!(is_spine_file("README.rst"));
}

#[test]
fn spine_file_readme_txt() {
    assert!(is_spine_file("README.txt"));
}

#[test]
fn spine_file_pyproject_toml() {
    assert!(is_spine_file("pyproject.toml"));
}

#[test]
fn spine_file_go_mod() {
    assert!(is_spine_file("go.mod"));
}

#[test]
fn spine_file_cockpit_toml() {
    assert!(is_spine_file("cockpit.toml"));
}

#[test]
fn spine_file_not_random_toml() {
    assert!(!is_spine_file("random.toml"));
}

#[test]
fn spine_file_not_random_md_in_docs() {
    assert!(!is_spine_file("docs/random.md"));
}

#[test]
fn spine_file_empty_path_is_not_spine() {
    assert!(!is_spine_file(""));
}

// ===========================================================================
// 7. Cross-function consistency
// ===========================================================================

#[test]
fn smart_exclude_and_classify_agree_on_lockfiles() {
    let lockfiles = [
        "Cargo.lock",
        "package-lock.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "poetry.lock",
        "Pipfile.lock",
        "go.sum",
        "composer.lock",
        "Gemfile.lock",
    ];
    for path in lockfiles {
        let exclude = smart_exclude_reason(path);
        let classes = classify_file(path, 100, 10, DEFAULT_DENSE_THRESHOLD);
        assert_eq!(exclude, Some("lockfile"), "smart_exclude for {path}");
        assert!(
            classes.contains(&FileClassification::Lockfile),
            "classify for {path}"
        );
    }
}

#[test]
fn smart_exclude_and_classify_agree_on_minified() {
    let minified = ["app.min.js", "styles.min.css"];
    for path in minified {
        let exclude = smart_exclude_reason(path);
        let classes = classify_file(path, 100, 10, DEFAULT_DENSE_THRESHOLD);
        assert_eq!(exclude, Some("minified"), "smart_exclude for {path}");
        assert!(
            classes.contains(&FileClassification::Minified),
            "classify for {path}"
        );
    }
}

#[test]
fn smart_exclude_and_classify_agree_on_sourcemaps() {
    let sourcemaps = ["app.js.map", "styles.css.map"];
    for path in sourcemaps {
        let exclude = smart_exclude_reason(path);
        let classes = classify_file(path, 100, 10, DEFAULT_DENSE_THRESHOLD);
        assert_eq!(exclude, Some("sourcemap"), "smart_exclude for {path}");
        assert!(
            classes.contains(&FileClassification::Sourcemap),
            "classify for {path}"
        );
    }
}

// ===========================================================================
// 8. Full pipeline with default constants
// ===========================================================================

#[test]
fn default_budget_128k_allows_typical_source_file() {
    let budget = 128_000;
    let cap = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, Some(DEFAULT_MAX_FILE_TOKENS));
    // 128_000 * 0.15 = 19_200, min(19_200, 16_000) = 16_000
    assert_eq!(cap, DEFAULT_MAX_FILE_TOKENS);

    let classes = classify_file("src/lib.rs", 5_000, 300, DEFAULT_DENSE_THRESHOLD);
    let (policy, _) = assign_policy(5_000, cap, &classes);
    assert_eq!(policy, InclusionPolicy::Full);
}

#[test]
fn default_budget_128k_head_tails_large_source_file() {
    let budget = 128_000;
    let cap = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, Some(DEFAULT_MAX_FILE_TOKENS));

    let classes = classify_file("src/big.rs", 20_000, 2_000, DEFAULT_DENSE_THRESHOLD);
    let (policy, reason) = assign_policy(20_000, cap, &classes);
    assert_eq!(policy, InclusionPolicy::HeadTail);
    assert!(reason.unwrap().contains("head+tail"));
}
