//! Tests for inclusion/exclusion policy decisions across file types,
//! smart-exclude patterns, classification accuracy, and determinism.

use tokmd_context_policy::{
    assign_policy, classify_file, compute_file_cap, is_spine_file, smart_exclude_reason,
    DEFAULT_DENSE_THRESHOLD, DEFAULT_MAX_FILE_PCT, DEFAULT_MAX_FILE_TOKENS,
};
use tokmd_types::{FileClassification, InclusionPolicy};

// ── Smart-exclude pattern coverage ──────────────────────────────────────

#[test]
fn smart_exclude_distinguishes_minified_from_sourcemap() {
    // Both end in `.js.map` vs `.min.js` — ensure no cross-contamination
    assert_eq!(smart_exclude_reason("app.min.js"), Some("minified"));
    assert_eq!(smart_exclude_reason("app.js.map"), Some("sourcemap"));
    assert_eq!(smart_exclude_reason("app.min.css"), Some("minified"));
    assert_eq!(smart_exclude_reason("app.css.map"), Some("sourcemap"));
}

#[test]
fn smart_exclude_does_not_match_partial_lockfile_names() {
    assert_eq!(smart_exclude_reason("Cargo.lock.backup"), None);
    assert_eq!(smart_exclude_reason("my-Cargo.lock"), None);
    assert_eq!(smart_exclude_reason("package-lock.json.old"), None);
}

#[test]
fn smart_exclude_handles_paths_with_only_basename() {
    assert_eq!(smart_exclude_reason("go.sum"), Some("lockfile"));
    assert_eq!(smart_exclude_reason("composer.lock"), Some("lockfile"));
    assert_eq!(smart_exclude_reason("Gemfile.lock"), Some("lockfile"));
}

// ── Classification accuracy for known file types ────────────────────────

#[test]
fn classify_detects_all_generated_file_patterns() {
    let patterns = [
        "src/node-types.json",
        "generated/grammar.json",
        "src/schema.generated.ts",
        "api/service.pb.go",
        "proto/types.pb.rs",
        "grpc/service_pb2.py",
        "lib/model.g.dart",
        "lib/state.freezed.dart",
    ];
    for path in patterns {
        let classes = classify_file(path, 100, 50, DEFAULT_DENSE_THRESHOLD);
        assert!(
            classes.contains(&FileClassification::Generated),
            "expected Generated for {path}, got {classes:?}"
        );
    }
}

#[test]
fn classify_detects_all_vendored_directory_variants() {
    let paths = [
        "vendor/lib.go",
        "third_party/sqlite.c",
        "third-party/dep.js",
        "node_modules/react/index.js",
    ];
    for path in paths {
        let classes = classify_file(path, 100, 50, DEFAULT_DENSE_THRESHOLD);
        assert!(
            classes.contains(&FileClassification::Vendored),
            "expected Vendored for {path}, got {classes:?}"
        );
    }
}

#[test]
fn classify_detects_all_fixture_directory_variants() {
    let paths = [
        "fixtures/sample.json",
        "testdata/input.txt",
        "test_data/corpus.bin",
        "__snapshots__/Component.snap",
        "golden/expected.txt",
    ];
    for path in paths {
        let classes = classify_file(path, 100, 50, DEFAULT_DENSE_THRESHOLD);
        assert!(
            classes.contains(&FileClassification::Fixture),
            "expected Fixture for {path}, got {classes:?}"
        );
    }
}

#[test]
fn classify_dense_blob_at_exact_threshold_boundary() {
    // At exactly threshold: tokens_per_line = 50.0/1.0 = 50.0, NOT > 50.0
    let at_threshold = classify_file("data.bin", 50, 1, 50.0);
    assert!(
        !at_threshold.contains(&FileClassification::DataBlob),
        "exactly at threshold should NOT be DataBlob"
    );

    // One above threshold
    let above_threshold = classify_file("data.bin", 51, 1, 50.0);
    assert!(
        above_threshold.contains(&FileClassification::DataBlob),
        "above threshold should be DataBlob"
    );
}

#[test]
fn classify_multiple_classifications_on_single_file() {
    // A generated file in a vendored directory that is also dense
    let classes = classify_file("vendor/proto/types.pb.go", 60_000, 10, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::Generated));
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn classify_lockfile_in_nested_vendored_directory() {
    let classes = classify_file("vendor/deps/Cargo.lock", 100, 50, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Lockfile));
    assert!(classes.contains(&FileClassification::Vendored));
}

// ── Policy decision chains: end-to-end flows ────────────────────────────

#[test]
fn policy_chain_small_source_file_gets_full_inclusion() {
    let path = "src/utils.rs";
    assert!(smart_exclude_reason(path).is_none());
    assert!(!is_spine_file(path));
    let classes = classify_file(path, 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.is_empty());
    let cap = compute_file_cap(128_000, DEFAULT_MAX_FILE_PCT, None);
    let (policy, reason) = assign_policy(500, cap, &classes);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

#[test]
fn policy_chain_oversized_normal_source_gets_head_tail() {
    let path = "src/large_module.rs";
    let classes = classify_file(path, 20_000, 4_000, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.is_empty());
    let cap = compute_file_cap(100_000, DEFAULT_MAX_FILE_PCT, None);
    let (policy, reason) = assign_policy(20_000, cap, &classes);
    assert_eq!(policy, InclusionPolicy::HeadTail);
    assert!(reason.unwrap().contains("head+tail"));
}

#[test]
fn policy_chain_oversized_generated_vendored_file_gets_skip() {
    let path = "vendor/proto/big.pb.rs";
    let classes = classify_file(path, 50_000, 5_000, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::Generated));
    let cap = compute_file_cap(128_000, DEFAULT_MAX_FILE_PCT, None);
    let (policy, reason) = assign_policy(50_000, cap, &classes);
    assert_eq!(policy, InclusionPolicy::Skip);
    let reason_text = reason.unwrap();
    assert!(reason_text.contains("generated") || reason_text.contains("vendored"));
}

#[test]
fn policy_chain_spine_file_under_cap_is_full() {
    let path = "Cargo.toml";
    assert!(is_spine_file(path));
    let classes = classify_file(path, 200, 30, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.is_empty());
    let cap = compute_file_cap(128_000, DEFAULT_MAX_FILE_PCT, None);
    let (policy, reason) = assign_policy(200, cap, &classes);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

// ── Boundary conditions on assign_policy ────────────────────────────────

#[test]
fn policy_exactly_at_cap_boundary_is_full() {
    let cap = 5_000;
    let (policy, _) = assign_policy(cap, cap, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
}

#[test]
fn policy_one_over_cap_with_no_classifications_is_head_tail() {
    let cap = 5_000;
    let (policy, _) = assign_policy(cap + 1, cap, &[]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

#[test]
fn policy_one_over_cap_with_generated_is_skip() {
    let cap = 5_000;
    let (policy, _) = assign_policy(cap + 1, cap, &[FileClassification::Generated]);
    assert_eq!(policy, InclusionPolicy::Skip);
}

#[test]
fn policy_skip_reason_includes_all_classification_names() {
    let classes = vec![
        FileClassification::DataBlob,
        FileClassification::Vendored,
    ];
    let (policy, reason) = assign_policy(20_000, 10_000, &classes);
    assert_eq!(policy, InclusionPolicy::Skip);
    let reason_text = reason.unwrap();
    assert!(reason_text.contains("data_blob"));
    assert!(reason_text.contains("vendored"));
}

#[test]
fn policy_reason_includes_token_counts() {
    let (_, reason) = assign_policy(20_000, 16_000, &[FileClassification::Generated]);
    let text = reason.unwrap();
    assert!(text.contains("20000"), "reason should include actual tokens");
    assert!(text.contains("16000"), "reason should include cap");
}

// ── Determinism: repeated calls produce identical results ───────────────

#[test]
fn determinism_classify_file_repeated_calls_identical() {
    let test_cases = [
        ("vendor/react.min.js", 80_000, 2),
        ("src/main.rs", 500, 100),
        ("Cargo.lock", 3_000, 200),
        ("proto/api.pb.go", 40_000, 2_000),
        ("testdata/large.json", 100_000, 10),
    ];
    for (path, tokens, lines) in test_cases {
        let first = classify_file(path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
        let second = classify_file(path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
        let third = classify_file(path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
        assert_eq!(first, second, "non-deterministic for {path}");
        assert_eq!(second, third, "non-deterministic for {path}");
    }
}

#[test]
fn determinism_full_policy_pipeline_is_stable() {
    let files = [
        ("src/lib.rs", 3_000, 200),
        ("vendor/dep.min.js", 80_000, 2),
        ("Cargo.lock", 50_000, 5_000),
        ("proto/types.pb.rs", 40_000, 2_000),
        ("README.md", 500, 80),
    ];
    let budget = 128_000;
    let cap = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, None);

    for _ in 0..3 {
        let mut policies = Vec::new();
        for (path, tokens, lines) in &files {
            let classes = classify_file(path, *tokens, *lines, DEFAULT_DENSE_THRESHOLD);
            let (policy, reason) = assign_policy(*tokens, cap, &classes);
            policies.push((policy, reason));
        }
        // Re-run and compare
        let mut policies2 = Vec::new();
        for (path, tokens, lines) in &files {
            let classes = classify_file(path, *tokens, *lines, DEFAULT_DENSE_THRESHOLD);
            let (policy, reason) = assign_policy(*tokens, cap, &classes);
            policies2.push((policy, reason));
        }
        assert_eq!(policies, policies2, "policy pipeline is not deterministic");
    }
}

// ── compute_file_cap edge cases ─────────────────────────────────────────

#[test]
fn compute_file_cap_default_constants_are_reasonable() {
    assert!(DEFAULT_MAX_FILE_PCT > 0.0 && DEFAULT_MAX_FILE_PCT < 1.0);
    assert!(DEFAULT_MAX_FILE_TOKENS > 0);
    assert!(DEFAULT_DENSE_THRESHOLD > 0.0);
}

#[test]
fn compute_file_cap_with_100pct_still_capped_by_hard_limit() {
    let cap = compute_file_cap(1_000_000, 1.0, Some(DEFAULT_MAX_FILE_TOKENS));
    assert_eq!(cap, DEFAULT_MAX_FILE_TOKENS);
}

#[test]
fn compute_file_cap_zero_pct_always_zero_regardless_of_budget() {
    assert_eq!(compute_file_cap(1_000_000, 0.0, Some(16_000)), 0);
    assert_eq!(compute_file_cap(100, 0.0, None), 0);
}

// ── Spine file path normalization ───────────────────────────────────────

#[test]
fn spine_file_backslash_normalization_for_all_path_patterns() {
    assert!(is_spine_file("docs\\architecture.md"));
    assert!(is_spine_file("docs\\design.md"));
    assert!(is_spine_file("project\\docs\\architecture.md"));
    assert!(is_spine_file("project\\Cargo.toml"));
}

#[test]
fn spine_file_rejects_partial_matches() {
    assert!(!is_spine_file("README.md.bak"));
    assert!(!is_spine_file("my-README.md"));
    assert!(!is_spine_file("Cargo.toml.orig"));
    assert!(!is_spine_file("not-a-Cargo.toml"));
}

// ── Classification does not produce duplicates ──────────────────────────

#[test]
fn classify_file_never_produces_duplicate_classifications() {
    // A file that could theoretically trigger the same pattern multiple times
    let classes = classify_file("vendor/node_modules/react.min.js", 80_000, 2, 50.0);
    let mut sorted = classes.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(classes, sorted, "classifications should be sorted and deduplicated");
}
