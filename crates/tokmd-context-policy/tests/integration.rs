use tokmd_context_policy::{
    DEFAULT_DENSE_THRESHOLD, assign_policy, classify_file, compute_file_cap, is_spine_file,
    smart_exclude_reason,
};
use tokmd_types::{FileClassification, InclusionPolicy};

#[test]
fn policy_pipeline_marks_vendored_dense_files_as_skip_when_over_cap() {
    let path = "vendor/github.com/lib/pq/conn.min.js";
    let classes = classify_file(path, 30_000, 10, DEFAULT_DENSE_THRESHOLD);
    let cap = compute_file_cap(40_000, 0.20, Some(8_000));
    let (policy, reason) = assign_policy(30_000, cap, &classes);

    assert_eq!(smart_exclude_reason(path), Some("minified"));
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::DataBlob));
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(
        reason
            .expect("skip should include reason")
            .contains("vendored")
    );
}

#[test]
fn policy_pipeline_keeps_small_spine_files_full() {
    let path = "docs/architecture.md";
    let classes = classify_file(path, 900, 120, DEFAULT_DENSE_THRESHOLD);
    let cap = compute_file_cap(20_000, 0.25, None);
    let (policy, reason) = assign_policy(900, cap, &classes);

    assert!(is_spine_file(path));
    assert!(smart_exclude_reason(path).is_none());
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

#[test]
fn compute_file_cap_supports_unbounded_budget() {
    let cap = compute_file_cap(usize::MAX, 0.15, Some(4_000));
    assert_eq!(cap, usize::MAX);
}

// ── End-to-end pipeline: lockfile path ──────────────────────────────────

#[test]
fn policy_pipeline_treats_small_lockfile_as_full() {
    let path = "Cargo.lock";
    assert_eq!(smart_exclude_reason(path), Some("lockfile"));
    let classes = classify_file(path, 2_000, 200, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Lockfile));
    let cap = compute_file_cap(50_000, 0.15, None);
    let (policy, reason) = assign_policy(2_000, cap, &classes);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

#[test]
fn policy_pipeline_treats_oversized_lockfile_as_head_tail() {
    let path = "package-lock.json";
    let classes = classify_file(path, 50_000, 5_000, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Lockfile));
    let cap = compute_file_cap(100_000, 0.15, Some(16_000));
    let (policy, reason) = assign_policy(50_000, cap, &classes);
    // lockfile is NOT in skip classes, so it gets head_tail
    assert_eq!(policy, InclusionPolicy::HeadTail);
    assert!(reason.unwrap().contains("head+tail"));
}

// ── End-to-end pipeline: generated protobuf ─────────────────────────────

#[test]
fn policy_pipeline_skips_large_protobuf_generated_file() {
    let path = "proto/service.pb.go";
    assert!(smart_exclude_reason(path).is_none()); // not a lockfile or minified
    let classes = classify_file(path, 40_000, 2_000, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
    let cap = compute_file_cap(128_000, 0.15, Some(16_000));
    let (policy, reason) = assign_policy(40_000, cap, &classes);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.unwrap().contains("generated"));
}

// ── End-to-end pipeline: fixture data ───────────────────────────────────

#[test]
fn policy_pipeline_head_tails_oversized_fixture_file() {
    let path = "tests/fixtures/large_sample.json";
    let classes = classify_file(path, 25_000, 5_000, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));
    let cap = compute_file_cap(100_000, 0.15, Some(16_000));
    let (policy, reason) = assign_policy(25_000, cap, &classes);
    // fixture alone is not a skip class
    assert_eq!(policy, InclusionPolicy::HeadTail);
    assert!(reason.unwrap().contains("head+tail"));
}

// ── End-to-end pipeline: dense data blob in fixture ─────────────────────

#[test]
fn policy_pipeline_skips_dense_fixture_blob() {
    let path = "tests/testdata/binary_dump.json";
    let classes = classify_file(path, 100_000, 10, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));
    assert!(classes.contains(&FileClassification::DataBlob));
    let cap = compute_file_cap(128_000, 0.15, Some(16_000));
    let (policy, _) = assign_policy(100_000, cap, &classes);
    // DataBlob IS a skip class
    assert_eq!(policy, InclusionPolicy::Skip);
}

// ── End-to-end pipeline: node_modules vendored minified ─────────────────

#[test]
fn policy_pipeline_skips_oversized_node_modules_minified_js() {
    let path = "node_modules/react/umd/react.min.js";
    assert_eq!(smart_exclude_reason(path), Some("minified"));
    let classes = classify_file(path, 80_000, 2, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::Minified));
    assert!(classes.contains(&FileClassification::DataBlob));
    let cap = compute_file_cap(128_000, 0.15, Some(16_000));
    let (policy, _) = assign_policy(80_000, cap, &classes);
    assert_eq!(policy, InclusionPolicy::Skip);
}

// ── End-to-end pipeline: normal source file ─────────────────────────────

#[test]
fn policy_pipeline_keeps_normal_source_file_full() {
    let path = "src/lib.rs";
    assert!(smart_exclude_reason(path).is_none());
    assert!(!is_spine_file(path));
    let classes = classify_file(path, 3_000, 200, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.is_empty());
    let cap = compute_file_cap(128_000, 0.15, Some(16_000));
    let (policy, reason) = assign_policy(3_000, cap, &classes);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

// ── End-to-end pipeline: third-party vendored dir ───────────────────────

#[test]
fn policy_pipeline_skips_oversized_third_party_file() {
    let path = "third_party/sqlite/sqlite3.c";
    let classes = classify_file(path, 200_000, 80_000, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Vendored));
    let cap = compute_file_cap(128_000, 0.15, Some(16_000));
    let (policy, reason) = assign_policy(200_000, cap, &classes);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.unwrap().contains("vendored"));
}

// ── Budget edge cases ───────────────────────────────────────────────────

#[test]
fn compute_file_cap_with_zero_pct_returns_zero() {
    let cap = compute_file_cap(100_000, 0.0, Some(16_000));
    assert_eq!(cap, 0);
}

#[test]
fn compute_file_cap_with_tiny_budget_and_no_hard_cap() {
    // 100 * 0.15 = 15, default hard cap = 16_000 → 15
    let cap = compute_file_cap(100, 0.15, None);
    assert_eq!(cap, 15);
}

#[test]
fn compute_file_cap_pct_equals_hard_cap() {
    // 40_000 * 0.25 = 10_000, hard_cap = 10_000 → 10_000
    let cap = compute_file_cap(40_000, 0.25, Some(10_000));
    assert_eq!(cap, 10_000);
}

// ── Snapshot directory at repo root ─────────────────────────────────────

#[test]
fn policy_pipeline_classifies_snapshot_directory_as_fixture() {
    let path = "__snapshots__/Component.test.js.snap";
    let classes = classify_file(path, 500, 50, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));
}

// ── Golden test directory ───────────────────────────────────────────────

#[test]
fn policy_pipeline_classifies_golden_directory_as_fixture() {
    let path = "tests/golden/expected_output.txt";
    let classes = classify_file(path, 500, 50, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Fixture));
}
