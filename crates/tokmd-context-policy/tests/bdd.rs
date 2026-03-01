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

// ── Additional smart-exclude scenarios ──────────────────────────────────

#[test]
fn given_all_lockfile_variants_when_checking_smart_exclude_then_lockfile_reason_is_returned() {
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
fn given_deeply_nested_lockfile_when_checking_smart_exclude_then_lockfile_reason_is_returned() {
    assert_eq!(smart_exclude_reason("a/b/c/d/e/go.sum"), Some("lockfile"));
}

#[test]
fn given_minified_css_when_checking_smart_exclude_then_minified_reason_is_returned() {
    assert_eq!(
        smart_exclude_reason("assets/styles.min.css"),
        Some("minified")
    );
}

#[test]
fn given_js_sourcemap_when_checking_smart_exclude_then_sourcemap_reason_is_returned() {
    assert_eq!(
        smart_exclude_reason("build/bundle.js.map"),
        Some("sourcemap")
    );
    assert_eq!(
        smart_exclude_reason("build/bundle.css.map"),
        Some("sourcemap")
    );
}

#[test]
fn given_non_lockfile_with_lock_in_name_when_checking_smart_exclude_then_none_is_returned() {
    assert_eq!(smart_exclude_reason("src/lock_manager.rs"), None);
    assert_eq!(smart_exclude_reason("Cargo.lock.bak"), None);
}

#[test]
fn given_empty_path_when_checking_smart_exclude_then_none_is_returned() {
    assert_eq!(smart_exclude_reason(""), None);
}

// ── Additional spine-file scenarios ─────────────────────────────────────

#[test]
fn given_all_spine_file_patterns_when_matching_then_all_are_detected() {
    let spine_files = [
        "README.md",
        "README",
        "README.rst",
        "README.txt",
        "ROADMAP.md",
        "CONTRIBUTING.md",
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        "tokmd.toml",
        "cockpit.toml",
    ];
    for sf in spine_files {
        assert!(is_spine_file(sf), "expected spine file: {sf}");
    }
}

#[test]
fn given_spine_file_with_deep_nesting_when_matching_then_detected() {
    assert!(is_spine_file("project/sub/README.md"));
    assert!(is_spine_file("project/sub/package.json"));
}

#[test]
fn given_docs_roadmap_spine_pattern_when_matching_then_detected() {
    assert!(is_spine_file("docs/ROADMAP.md"));
    assert!(is_spine_file("myproject/docs/ROADMAP.md"));
}

#[test]
fn given_docs_design_spine_pattern_when_matching_then_detected() {
    assert!(is_spine_file("docs/design.md"));
    assert!(is_spine_file("repo/docs/design.md"));
}

#[test]
fn given_non_spine_file_when_matching_then_not_detected() {
    assert!(!is_spine_file("src/lib.rs"));
    assert!(!is_spine_file("tests/test_main.py"));
    assert!(!is_spine_file("README.md.bak"));
}

#[test]
fn given_windows_style_spine_path_when_matching_then_detected_after_normalization() {
    assert!(is_spine_file("docs\\architecture.md"));
    assert!(is_spine_file("project\\docs\\design.md"));
}

// ── Additional classification scenarios ─────────────────────────────────

#[test]
fn given_lockfile_path_when_classifying_then_lockfile_classification_is_returned() {
    let classes = classify_file("Cargo.lock", 1000, 100, 50.0);
    assert_eq!(classes, vec![FileClassification::Lockfile]);
}

#[test]
fn given_minified_js_when_classifying_then_minified_classification_is_returned() {
    let classes = classify_file("dist/app.min.js", 500, 100, 50.0);
    assert!(classes.contains(&FileClassification::Minified));
}

#[test]
fn given_minified_css_when_classifying_then_minified_classification_is_returned() {
    let classes = classify_file("dist/styles.min.css", 500, 100, 50.0);
    assert!(classes.contains(&FileClassification::Minified));
}

#[test]
fn given_sourcemap_when_classifying_then_sourcemap_classification_is_returned() {
    let classes = classify_file("dist/app.js.map", 500, 100, 50.0);
    assert!(classes.contains(&FileClassification::Sourcemap));
}

#[test]
fn given_css_sourcemap_when_classifying_then_sourcemap_classification_is_returned() {
    let classes = classify_file("dist/app.css.map", 500, 100, 50.0);
    assert!(classes.contains(&FileClassification::Sourcemap));
}

#[test]
fn given_protobuf_go_file_when_classifying_then_generated_classification_is_returned() {
    let classes = classify_file("api/types.pb.go", 500, 100, 50.0);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn given_protobuf_python_file_when_classifying_then_generated_classification_is_returned() {
    let classes = classify_file("api/types_pb2.py", 500, 100, 50.0);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn given_dart_generated_file_when_classifying_then_generated_classification_is_returned() {
    let classes = classify_file("lib/model.g.dart", 500, 100, 50.0);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn given_dart_freezed_file_when_classifying_then_generated_classification_is_returned() {
    let classes = classify_file("lib/model.freezed.dart", 500, 100, 50.0);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn given_generated_marker_in_name_when_classifying_then_generated_classification_is_returned() {
    let classes = classify_file("src/schema.generated.ts", 500, 100, 50.0);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn given_vendored_path_when_classifying_then_vendored_classification_is_returned() {
    for dir in ["vendor/", "third_party/", "third-party/", "node_modules/"] {
        let path = format!("{dir}lib/foo.js");
        let classes = classify_file(&path, 500, 100, 50.0);
        assert!(
            classes.contains(&FileClassification::Vendored),
            "expected vendored for path: {path}"
        );
    }
}

#[test]
fn given_fixture_path_when_classifying_then_fixture_classification_is_returned() {
    for dir in [
        "fixtures/",
        "testdata/",
        "test_data/",
        "__snapshots__/",
        "golden/",
    ] {
        let path = format!("tests/{dir}sample.json");
        let classes = classify_file(&path, 500, 100, 50.0);
        assert!(
            classes.contains(&FileClassification::Fixture),
            "expected fixture for path: {path}"
        );
    }
}

#[test]
fn given_zero_lines_file_when_classifying_then_dense_blob_detected_when_tokens_exceed_threshold() {
    let classes = classify_file("data/blob.bin", 100, 0, 50.0);
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn given_normal_file_when_classifying_then_no_classifications_returned() {
    let classes = classify_file("src/main.rs", 500, 100, 50.0);
    assert!(classes.is_empty());
}

#[test]
fn given_vendored_minified_dense_file_when_classifying_then_multiple_classifications_returned() {
    let classes = classify_file("vendor/lib/app.min.js", 50_000, 1, 50.0);
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::Minified));
    assert!(classes.contains(&FileClassification::DataBlob));
}

// ── Additional policy assignment scenarios ──────────────────────────────

#[test]
fn given_file_exactly_at_cap_when_assigning_policy_then_full_is_selected() {
    let (policy, reason) = assign_policy(16_000, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

#[test]
fn given_file_one_token_over_cap_when_assigning_policy_then_head_tail_is_selected() {
    let (policy, reason) = assign_policy(16_001, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
    assert!(reason.is_some());
}

#[test]
fn given_oversized_vendored_file_when_assigning_policy_then_skip_is_selected() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::Vendored]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.unwrap().contains("vendored"));
}

#[test]
fn given_oversized_data_blob_file_when_assigning_policy_then_skip_is_selected() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::DataBlob]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.unwrap().contains("data_blob"));
}

#[test]
fn given_oversized_fixture_file_when_assigning_policy_then_head_tail_not_skip() {
    let (policy, _) = assign_policy(20_000, 16_000, &[FileClassification::Fixture]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

#[test]
fn given_oversized_lockfile_when_assigning_policy_then_head_tail_not_skip() {
    let (policy, _) = assign_policy(20_000, 16_000, &[FileClassification::Lockfile]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

#[test]
fn given_oversized_minified_file_when_assigning_policy_then_head_tail_not_skip() {
    let (policy, _) = assign_policy(20_000, 16_000, &[FileClassification::Minified]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

#[test]
fn given_oversized_sourcemap_file_when_assigning_policy_then_head_tail_not_skip() {
    let (policy, _) = assign_policy(20_000, 16_000, &[FileClassification::Sourcemap]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

#[test]
fn given_oversized_file_with_mixed_skip_and_non_skip_classes_when_assigning_then_skip_wins() {
    let classes = vec![FileClassification::Fixture, FileClassification::Generated];
    let (policy, reason) = assign_policy(20_000, 16_000, &classes);
    assert_eq!(policy, InclusionPolicy::Skip);
    let reason_text = reason.unwrap();
    assert!(reason_text.contains("fixture"));
    assert!(reason_text.contains("generated"));
}

#[test]
fn given_zero_tokens_file_when_assigning_policy_then_full_is_selected() {
    let (policy, reason) = assign_policy(0, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

// ── Additional budget/cap scenarios ─────────────────────────────────────

#[test]
fn given_zero_budget_when_computing_cap_then_zero_is_returned() {
    let cap = compute_file_cap(0, 0.15, Some(16_000));
    assert_eq!(cap, 0);
}

#[test]
fn given_pct_cap_smaller_than_hard_cap_when_computing_then_pct_cap_wins() {
    // 10_000 * 0.10 = 1_000, hard_cap = 5_000 → min(1_000, 5_000) = 1_000
    let cap = compute_file_cap(10_000, 0.10, Some(5_000));
    assert_eq!(cap, 1_000);
}

#[test]
fn given_no_hard_cap_when_computing_then_default_hard_cap_used() {
    // 200_000 * 0.15 = 30_000, default hard cap is 16_000 → min(30_000, 16_000) = 16_000
    let cap = compute_file_cap(200_000, 0.15, None);
    assert_eq!(cap, tokmd_context_policy::DEFAULT_MAX_FILE_TOKENS);
}

#[test]
fn given_small_budget_with_no_hard_cap_when_computing_then_pct_dominates() {
    // 1_000 * 0.15 = 150, default hard cap 16_000 → min(150, 16_000) = 150
    let cap = compute_file_cap(1_000, 0.15, None);
    assert_eq!(cap, 150);
}

#[test]
fn given_full_budget_percentage_when_computing_then_hard_cap_still_limits() {
    // 100_000 * 1.0 = 100_000, hard_cap = 8_000 → 8_000
    let cap = compute_file_cap(100_000, 1.0, Some(8_000));
    assert_eq!(cap, 8_000);
}
