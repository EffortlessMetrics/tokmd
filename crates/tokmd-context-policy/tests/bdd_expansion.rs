//! Expanded BDD-style tests for tokmd-context-policy.
//!
//! Covers: smart exclude edge cases, file classification with Windows paths,
//! budget constraint boundaries, spine file edge cases, and policy interactions.

use tokmd_context_policy::{
    DEFAULT_DENSE_THRESHOLD, DEFAULT_MAX_FILE_PCT, DEFAULT_MAX_FILE_TOKENS, assign_policy,
    classify_file, compute_file_cap, is_spine_file, smart_exclude_reason,
};
use tokmd_types::{FileClassification, InclusionPolicy};

// ============================================================================
// Scenario: Smart excludes — additional lockfile and edge cases
// ============================================================================

mod smart_exclude_edge_cases {
    use super::*;

    #[test]
    fn given_lockfile_in_monorepo_subdirectory_then_detected() {
        assert_eq!(
            smart_exclude_reason("packages/web/yarn.lock"),
            Some("lockfile")
        );
        assert_eq!(
            smart_exclude_reason("services/api/poetry.lock"),
            Some("lockfile")
        );
    }

    #[test]
    fn given_file_named_like_lockfile_but_different_then_not_detected() {
        assert_eq!(smart_exclude_reason("my-Cargo.lock.backup"), None);
        assert_eq!(smart_exclude_reason("Cargo.lock.old"), None);
        assert_eq!(smart_exclude_reason("not-Cargo.lock"), None);
    }

    #[test]
    fn given_minified_js_at_root_then_detected() {
        assert_eq!(smart_exclude_reason("bundle.min.js"), Some("minified"));
    }

    #[test]
    fn given_minified_css_nested_deeply_then_detected() {
        assert_eq!(
            smart_exclude_reason("static/assets/v2/styles.min.css"),
            Some("minified")
        );
    }

    #[test]
    fn given_sourcemap_at_root_then_detected() {
        assert_eq!(smart_exclude_reason("app.js.map"), Some("sourcemap"));
    }

    #[test]
    fn given_file_ending_with_map_but_not_sourcemap_then_not_detected() {
        // "sitemap.xml" doesn't end with ".js.map" or ".css.map"
        assert_eq!(smart_exclude_reason("sitemap.xml"), None);
        assert_eq!(smart_exclude_reason("roadmap.md"), None);
    }

    #[test]
    fn given_normal_js_file_then_not_excluded() {
        assert_eq!(smart_exclude_reason("src/app.js"), None);
        assert_eq!(smart_exclude_reason("lib/utils.css"), None);
    }
}

// ============================================================================
// Scenario: Classification with Windows-style and edge-case paths
// ============================================================================

mod classification_path_normalization {
    use super::*;

    #[test]
    fn given_windows_path_to_vendored_file_then_vendored_detected() {
        let classes = classify_file("vendor\\lib\\dep.js", 500, 100, DEFAULT_DENSE_THRESHOLD);
        assert!(
            classes.contains(&FileClassification::Vendored),
            "Windows-style vendor path should be detected: {classes:?}"
        );
    }

    #[test]
    fn given_windows_path_to_fixture_then_fixture_detected() {
        let classes = classify_file(
            "tests\\fixtures\\sample.json",
            500,
            100,
            DEFAULT_DENSE_THRESHOLD,
        );
        assert!(
            classes.contains(&FileClassification::Fixture),
            "Windows-style fixture path should be detected: {classes:?}"
        );
    }

    #[test]
    fn given_windows_path_to_lockfile_then_lockfile_detected() {
        let classes = classify_file("project\\Cargo.lock", 500, 100, DEFAULT_DENSE_THRESHOLD);
        assert!(
            classes.contains(&FileClassification::Lockfile),
            "Windows-style lockfile should be detected: {classes:?}"
        );
    }

    #[test]
    fn given_node_modules_with_mixed_separators_then_vendored_detected() {
        let classes = classify_file(
            "project\\node_modules/react/index.js",
            500,
            100,
            DEFAULT_DENSE_THRESHOLD,
        );
        assert!(
            classes.contains(&FileClassification::Vendored),
            "Mixed-separator node_modules should be detected: {classes:?}"
        );
    }

    #[test]
    fn given_protobuf_rs_file_then_generated_detected() {
        let classes = classify_file("proto/types.pb.rs", 500, 100, DEFAULT_DENSE_THRESHOLD);
        assert!(classes.contains(&FileClassification::Generated));
    }
}

// ============================================================================
// Scenario: Dense blob threshold boundary cases
// ============================================================================

mod dense_blob_boundaries {
    use super::*;

    #[test]
    fn given_tokens_per_line_just_below_threshold_then_not_dense() {
        // 49 tokens / 1 line = 49.0, threshold = 50.0
        let classes = classify_file("data.csv", 49, 1, 50.0);
        assert!(!classes.contains(&FileClassification::DataBlob));
    }

    #[test]
    fn given_tokens_per_line_exactly_at_threshold_then_not_dense() {
        // 50 tokens / 1 line = 50.0, NOT > 50.0
        let classes = classify_file("data.csv", 50, 1, 50.0);
        assert!(!classes.contains(&FileClassification::DataBlob));
    }

    #[test]
    fn given_tokens_per_line_just_above_threshold_then_dense() {
        // 51 tokens / 1 line = 51.0 > 50.0
        let classes = classify_file("data.csv", 51, 1, 50.0);
        assert!(classes.contains(&FileClassification::DataBlob));
    }

    #[test]
    fn given_many_lines_with_few_tokens_then_not_dense() {
        // 500 tokens / 1000 lines = 0.5, well below threshold
        let classes = classify_file("src/sparse.rs", 500, 1000, 50.0);
        assert!(!classes.contains(&FileClassification::DataBlob));
    }

    #[test]
    fn given_custom_threshold_then_respected() {
        // Use a very low threshold
        let classes = classify_file("src/main.rs", 10, 1, 5.0);
        assert!(classes.contains(&FileClassification::DataBlob));

        // Use a very high threshold
        let classes = classify_file("src/main.rs", 10, 1, 100.0);
        assert!(!classes.contains(&FileClassification::DataBlob));
    }
}

// ============================================================================
// Scenario: Budget constraint edge cases
// ============================================================================

mod budget_constraints {
    use super::*;

    #[test]
    fn given_budget_of_one_then_cap_is_zero_with_default_pct() {
        // 1 * 0.15 = 0.15, truncated to 0
        let cap = compute_file_cap(1, DEFAULT_MAX_FILE_PCT, None);
        assert_eq!(cap, 0);
    }

    #[test]
    fn given_budget_where_pct_equals_hard_cap_then_returns_either() {
        // 100_000 * 0.16 = 16_000, hard_cap = 16_000 → 16_000
        let cap = compute_file_cap(100_000, 0.16, Some(16_000));
        assert_eq!(cap, 16_000);
    }

    #[test]
    fn given_very_large_pct_over_one_then_hard_cap_limits() {
        // 100 * 2.0 = 200, hard_cap = 50 → 50
        let cap = compute_file_cap(100, 2.0, Some(50));
        assert_eq!(cap, 50);
    }

    #[test]
    fn given_zero_pct_then_cap_is_zero() {
        let cap = compute_file_cap(1_000_000, 0.0, Some(16_000));
        assert_eq!(cap, 0);
    }

    #[test]
    fn given_hard_cap_of_zero_then_cap_is_zero() {
        let cap = compute_file_cap(1_000_000, 0.15, Some(0));
        assert_eq!(cap, 0);
    }

    #[test]
    fn given_usize_max_budget_with_no_hard_cap_then_returns_usize_max() {
        let cap = compute_file_cap(usize::MAX, 0.15, None);
        assert_eq!(cap, usize::MAX);
    }
}

// ============================================================================
// Scenario: Spine file edge cases
// ============================================================================

mod spine_file_edge_cases {
    use super::*;

    #[test]
    fn given_readme_variants_then_all_detected() {
        assert!(is_spine_file("README.md"));
        assert!(is_spine_file("README"));
        assert!(is_spine_file("README.rst"));
        assert!(is_spine_file("README.txt"));
    }

    #[test]
    fn given_readme_with_wrong_extension_then_not_detected() {
        assert!(!is_spine_file("README.html"));
        assert!(!is_spine_file("README.pdf"));
    }

    #[test]
    fn given_lowercase_readme_then_not_detected() {
        // SPINE_PATTERNS are exact matches; "readme.md" != "README.md"
        assert!(!is_spine_file("readme.md"));
    }

    #[test]
    fn given_docs_architecture_at_various_nesting_levels_then_detected() {
        assert!(is_spine_file("docs/architecture.md"));
        assert!(is_spine_file("project/docs/architecture.md"));
        assert!(is_spine_file("mono/packages/core/docs/architecture.md"));
    }

    #[test]
    fn given_architecture_md_without_docs_prefix_then_not_detected() {
        // "architecture.md" alone is not a spine pattern (needs docs/ prefix)
        assert!(!is_spine_file("architecture.md"));
    }

    #[test]
    fn given_empty_path_then_not_spine() {
        assert!(!is_spine_file(""));
    }
}

// ============================================================================
// Scenario: Policy assignment with multiple classifications
// ============================================================================

mod policy_multi_classification {
    use super::*;

    #[test]
    fn given_generated_and_vendored_oversized_then_skip_with_both_in_reason() {
        let (policy, reason) = assign_policy(
            20_000,
            16_000,
            &[FileClassification::Generated, FileClassification::Vendored],
        );
        assert_eq!(policy, InclusionPolicy::Skip);
        let reason = reason.unwrap();
        assert!(reason.contains("generated"));
        assert!(reason.contains("vendored"));
    }

    #[test]
    fn given_fixture_and_lockfile_oversized_then_head_tail() {
        // Neither fixture nor lockfile is a skip class
        let (policy, _) = assign_policy(
            20_000,
            16_000,
            &[FileClassification::Fixture, FileClassification::Lockfile],
        );
        assert_eq!(policy, InclusionPolicy::HeadTail);
    }

    #[test]
    fn given_empty_classifications_under_cap_then_full() {
        let (policy, reason) = assign_policy(100, 16_000, &[]);
        assert_eq!(policy, InclusionPolicy::Full);
        assert!(reason.is_none());
    }

    #[test]
    fn given_skip_class_but_under_cap_then_full() {
        // Even generated/vendored files are Full if under cap
        let (policy, reason) =
            assign_policy(100, 16_000, &[FileClassification::Generated]);
        assert_eq!(policy, InclusionPolicy::Full);
        assert!(reason.is_none());
    }

    #[test]
    fn given_data_blob_oversized_then_skip() {
        let (policy, reason) =
            assign_policy(20_000, 16_000, &[FileClassification::DataBlob]);
        assert_eq!(policy, InclusionPolicy::Skip);
        assert!(reason.unwrap().contains("data_blob"));
    }
}
