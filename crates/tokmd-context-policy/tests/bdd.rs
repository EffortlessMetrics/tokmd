use tokmd_context_policy::{
    assign_policy, classify_file, compute_file_cap, is_spine_file, smart_exclude_reason,
};
use tokmd_types::{FileClassification, InclusionPolicy};

#[test]
fn given_lockfile_path_when_checking_smart_exclude_then_lockfile_reason_is_returned() {
    assert_eq!(smart_exclude_reason("Cargo.lock"), Some("lockfile"));
    assert_eq!(
        smart_exclude_reason("services/api/package-lock.json"),
        Some("lockfile")
    );
}

#[test]
fn given_minified_and_sourcemap_files_when_checking_smart_exclude_then_expected_reason_is_returned()
{
    assert_eq!(smart_exclude_reason("dist/app.min.js"), Some("minified"));
    assert_eq!(smart_exclude_reason("dist/app.css.map"), Some("sourcemap"));
}

#[test]
fn given_repository_spine_files_when_matching_then_patterns_are_detected() {
    assert!(is_spine_file("README.md"));
    assert!(is_spine_file("docs/architecture.md"));
    assert!(is_spine_file("nested/path/Cargo.toml"));
    assert!(!is_spine_file("src/main.rs"));
}

#[test]
fn given_oversized_generated_file_when_assigning_policy_then_skip_is_selected() {
    let classes = classify_file("proto/types.pb.rs", 20_000, 200, 50.0);
    let (policy, reason) = assign_policy(20_000, 16_000, &classes);

    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(
        reason
            .expect("skip policy should include reason")
            .contains("generated")
    );
}

#[test]
fn given_regular_oversized_file_when_assigning_policy_then_head_tail_is_selected() {
    let classes = vec![FileClassification::Fixture];
    let (policy, reason) = assign_policy(20_000, 16_000, &classes);

    assert_eq!(policy, InclusionPolicy::HeadTail);
    assert!(
        reason
            .expect("headtail policy should include reason")
            .contains("head+tail")
    );
}

#[test]
fn given_budget_and_cap_settings_when_computing_cap_then_minimum_rule_is_applied() {
    let cap = compute_file_cap(128_000, 0.25, Some(10_000));
    assert_eq!(cap, 10_000);
}
