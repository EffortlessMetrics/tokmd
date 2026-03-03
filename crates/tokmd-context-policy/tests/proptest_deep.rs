//! Deep property-based tests for tokmd-context-policy.
//!
//! Covers: policy monotonicity, classification completeness,
//! cap computation bounds, and smart-exclude consistency.

use proptest::prelude::*;
use tokmd_context_policy::{
    DEFAULT_DENSE_THRESHOLD, assign_policy, classify_file, compute_file_cap, is_spine_file,
    smart_exclude_reason,
};
use tokmd_types::{FileClassification, InclusionPolicy};

// =========================================================================
// Policy monotonicity: more tokens => more restrictive or equal policy
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn policy_monotonic_with_tokens(
        tokens_small in 0usize..50_000,
        delta in 1usize..100_000,
        cap in 1usize..100_000,
    ) {
        let tokens_large = tokens_small + delta;
        let (policy_small, _) = assign_policy(tokens_small, cap, &[]);
        let (policy_large, _) = assign_policy(tokens_large, cap, &[]);
        // Policy ordering: Full < HeadTail < Summary < Skip
        let rank = |p: InclusionPolicy| match p {
            InclusionPolicy::Full => 0,
            InclusionPolicy::HeadTail => 1,
            InclusionPolicy::Summary => 2,
            InclusionPolicy::Skip => 3,
        };
        prop_assert!(
            rank(policy_large) >= rank(policy_small),
            "More tokens ({}) should not produce less restrictive policy than fewer ({})",
            tokens_large, tokens_small
        );
    }
}

// =========================================================================
// Classification: generated/vendored/datablob implies skip on oversized
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn skip_classes_always_skip_when_oversized(
        tokens in 1usize..200_000,
        cap in 0usize..200_000,
        class_idx in 0usize..3,
    ) {
        let class = [
            FileClassification::Generated,
            FileClassification::Vendored,
            FileClassification::DataBlob,
        ][class_idx];
        let (policy, _) = assign_policy(tokens, cap, &[class]);
        if tokens > cap {
            prop_assert_eq!(policy, InclusionPolicy::Skip,
                "class {:?} with tokens={} > cap={} should skip", class, tokens, cap);
        }
    }

    #[test]
    fn non_skip_classes_never_skip_when_oversized(
        tokens in 1usize..200_000,
        cap in 0usize..200_000,
        class_idx in 0usize..3,
    ) {
        let class = [
            FileClassification::Fixture,
            FileClassification::Lockfile,
            FileClassification::Minified,
        ][class_idx];
        let (policy, _) = assign_policy(tokens, cap, &[class]);
        if tokens > cap {
            prop_assert_ne!(policy, InclusionPolicy::Skip,
                "class {:?} with tokens={} > cap={} should not skip", class, tokens, cap);
        }
    }
}

// =========================================================================
// compute_file_cap: mathematical bounds
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn file_cap_monotonic_with_budget(
        budget_small in 0usize..500_000,
        delta in 1usize..500_000,
        pct in 0.01f64..1.0,
    ) {
        let budget_large = budget_small + delta;
        let cap_small = compute_file_cap(budget_small, pct, None);
        let cap_large = compute_file_cap(budget_large, pct, None);
        prop_assert!(cap_large >= cap_small,
            "More budget should give >= cap: cap({})={}, cap({})={}",
            budget_large, cap_large, budget_small, cap_small);
    }

    #[test]
    fn file_cap_monotonic_with_pct(
        budget in 1000usize..1_000_000,
        pct_small in 0.01f64..0.5,
    ) {
        let pct_large = pct_small + 0.01;
        let cap_small = compute_file_cap(budget, pct_small, None);
        let cap_large = compute_file_cap(budget, pct_large, None);
        prop_assert!(cap_large >= cap_small,
            "More pct should give >= cap");
    }

    #[test]
    fn file_cap_hard_cap_respected(
        budget in 0usize..1_000_000,
        pct in 0.01f64..1.0,
        hard_cap in 1usize..50_000,
    ) {
        let cap = compute_file_cap(budget, pct, Some(hard_cap));
        prop_assert!(cap <= hard_cap, "cap {} exceeds hard_cap {}", cap, hard_cap);
    }
}

// =========================================================================
// Smart exclude: comprehensive pattern matching
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn sourcemap_files_detected(
        stem in "[a-z]{2,12}",
        prefix in prop::option::of("[a-z]{2,8}"),
    ) {
        let filename = format!("{}.js.map", stem);
        let path = match prefix {
            Some(p) => format!("{}/{}", p, filename),
            None => filename,
        };
        let reason = smart_exclude_reason(&path);
        prop_assert_eq!(reason, Some("sourcemap"),
            "Expected sourcemap for {}", path);
    }

    #[test]
    fn css_sourcemap_detected(
        stem in "[a-z]{2,12}",
    ) {
        let path = format!("{}.css.map", stem);
        let reason = smart_exclude_reason(&path);
        prop_assert_eq!(reason, Some("sourcemap"),
            "Expected sourcemap for {}", path);
    }

    #[test]
    fn regular_files_not_excluded(
        stem in "[a-z]{2,12}",
        ext in prop::sample::select(vec!["rs", "py", "go", "java", "c", "cpp", "h"]),
    ) {
        let path = format!("{}.{}", stem, ext);
        let reason = smart_exclude_reason(&path);
        prop_assert_eq!(reason, None,
            "Regular source file {} should not be excluded", path);
    }
}

// =========================================================================
// classify_file + smart_exclude consistency
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn classify_and_exclude_agree_on_minified(
        stem in "[a-z]{2,12}",
        ext in prop::sample::select(vec!["js", "css"]),
    ) {
        let path = format!("{}.min.{}", stem, ext);
        let exclude = smart_exclude_reason(&path);
        let classes = classify_file(&path, 100, 10, DEFAULT_DENSE_THRESHOLD);
        if exclude == Some("minified") {
            prop_assert!(classes.contains(&FileClassification::Minified),
                "minified file {} not classified as Minified", path);
        }
    }

    #[test]
    fn spine_file_is_deterministic_and_consistent(path in "[a-zA-Z0-9_./-]{1,40}") {
        let a = is_spine_file(&path);
        let b = is_spine_file(&path);
        prop_assert_eq!(a, b, "is_spine_file must be deterministic");
    }
}
