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
