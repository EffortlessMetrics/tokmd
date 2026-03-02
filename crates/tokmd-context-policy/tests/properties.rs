use proptest::prelude::*;
use tokmd_context_policy::{
    assign_policy, classify_file, compute_file_cap, is_spine_file, smart_exclude_reason,
    DEFAULT_DENSE_THRESHOLD,
};
use tokmd_types::{FileClassification, InclusionPolicy};

fn arbitrary_path() -> impl Strategy<Value = String> {
    "\\PC{0,256}".prop_map(|s| s.replace('\u{0000}', ""))
}

proptest! {
    #[test]
    fn smart_exclude_reason_is_deterministic(path in arbitrary_path()) {
        let a = smart_exclude_reason(&path);
        let b = smart_exclude_reason(&path);
        prop_assert_eq!(a, b);
    }

    #[test]
    fn smart_exclude_reason_is_always_known_reason(path in arbitrary_path()) {
        let reason = smart_exclude_reason(&path);
        if let Some(label) = reason {
            prop_assert!(matches!(label, "lockfile" | "minified" | "sourcemap"));
        }
    }

    #[test]
    fn classify_file_results_are_sorted_and_unique(
        path in arbitrary_path(),
        tokens in 0usize..200_000,
        lines in 0usize..20_000,
    ) {
        let classes = classify_file(&path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
        let mut sorted = classes.clone();
        sorted.sort();
        sorted.dedup();
        prop_assert_eq!(classes, sorted);
    }

    #[test]
    fn assign_policy_without_skip_classes_behaves_as_size_gate(
        tokens in 0usize..200_000,
        cap in 0usize..200_000,
    ) {
        let (policy, reason) = assign_policy(tokens, cap, &[]);
        if tokens <= cap {
            prop_assert_eq!(policy, InclusionPolicy::Full);
            prop_assert!(reason.is_none());
        } else {
            prop_assert_eq!(policy, InclusionPolicy::HeadTail);
            prop_assert!(reason.is_some());
        }
    }

    #[test]
    fn compute_file_cap_is_bounded_by_hard_cap_when_provided(
        budget in 0usize..1_000_000,
        pct in 0f64..1.0,
        hard_cap in 1usize..50_000,
    ) {
        let cap = compute_file_cap(budget, pct, Some(hard_cap));
        prop_assert!(cap <= hard_cap);
    }

    #[test]
    fn spine_classification_is_deterministic(path in arbitrary_path()) {
        let a = is_spine_file(&path);
        let b = is_spine_file(&path);
        prop_assert_eq!(a, b);
    }

    #[test]
    fn classify_file_is_deterministic(
        path in arbitrary_path(),
        tokens in 0usize..200_000,
        lines in 0usize..20_000,
    ) {
        let a = classify_file(&path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
        let b = classify_file(&path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
        prop_assert_eq!(a, b);
    }

    #[test]
    fn compute_file_cap_never_exceeds_budget(
        budget in 0usize..1_000_000,
        pct in 0f64..1.0,
    ) {
        let cap = compute_file_cap(budget, pct, None);
        prop_assert!(cap <= budget);
    }

    #[test]
    fn compute_file_cap_unbounded_returns_max(
        pct in 0f64..1.0,
        hard_cap in 1usize..100_000,
    ) {
        let cap = compute_file_cap(usize::MAX, pct, Some(hard_cap));
        prop_assert_eq!(cap, usize::MAX);
    }

    #[test]
    fn assign_policy_full_implies_no_reason(
        tokens in 0usize..50_000,
        cap in 0usize..200_000,
    ) {
        let (policy, reason) = assign_policy(tokens, cap, &[]);
        if policy == InclusionPolicy::Full {
            prop_assert!(reason.is_none());
        }
    }

    #[test]
    fn assign_policy_skip_or_headtail_implies_reason(
        tokens in 1usize..200_000,
        cap in 0usize..200_000,
    ) {
        let (policy, reason) = assign_policy(tokens, cap, &[FileClassification::Generated]);
        if policy != InclusionPolicy::Full {
            prop_assert!(reason.is_some());
        }
    }

    #[test]
    fn assign_policy_generated_oversized_always_skip(
        tokens in 1usize..200_000,
        cap in 0usize..200_000,
    ) {
        let (policy, _) = assign_policy(tokens, cap, &[FileClassification::Generated]);
        if tokens > cap {
            prop_assert_eq!(policy, InclusionPolicy::Skip);
        }
    }

    #[test]
    fn assign_policy_vendored_oversized_always_skip(
        tokens in 1usize..200_000,
        cap in 0usize..200_000,
    ) {
        let (policy, _) = assign_policy(tokens, cap, &[FileClassification::Vendored]);
        if tokens > cap {
            prop_assert_eq!(policy, InclusionPolicy::Skip);
        }
    }

    #[test]
    fn assign_policy_datablob_oversized_always_skip(
        tokens in 1usize..200_000,
        cap in 0usize..200_000,
    ) {
        let (policy, _) = assign_policy(tokens, cap, &[FileClassification::DataBlob]);
        if tokens > cap {
            prop_assert_eq!(policy, InclusionPolicy::Skip);
        }
    }

    #[test]
    fn assign_policy_fixture_oversized_never_skip(
        tokens in 1usize..200_000,
        cap in 0usize..200_000,
    ) {
        let (policy, _) = assign_policy(tokens, cap, &[FileClassification::Fixture]);
        if tokens > cap {
            prop_assert_eq!(policy, InclusionPolicy::HeadTail);
        }
    }

    #[test]
    fn assign_policy_lockfile_oversized_never_skip(
        tokens in 1usize..200_000,
        cap in 0usize..200_000,
    ) {
        let (policy, _) = assign_policy(tokens, cap, &[FileClassification::Lockfile]);
        if tokens > cap {
            prop_assert_eq!(policy, InclusionPolicy::HeadTail);
        }
    }

    #[test]
    fn classify_file_zero_lines_with_tokens_always_dense(
        tokens in 1usize..200_000,
    ) {
        let classes = classify_file("blob.bin", tokens, 0, DEFAULT_DENSE_THRESHOLD);
        // tokens/max(0,1) = tokens, which should exceed threshold for tokens > 50
        if tokens as f64 > DEFAULT_DENSE_THRESHOLD {
            prop_assert!(classes.contains(&FileClassification::DataBlob));
        }
    }

    #[test]
    fn smart_exclude_and_classify_agree_on_lockfiles(path in arbitrary_path()) {
        let exclude = smart_exclude_reason(&path);
        let classes = classify_file(&path, 100, 10, DEFAULT_DENSE_THRESHOLD);
        if exclude == Some("lockfile") {
            prop_assert!(classes.contains(&FileClassification::Lockfile));
        }
    }

    // NEW property tests

    #[test]
    fn file_cap_positive(
        budget in 1usize..1_000_000,
        pct in 0.01f64..1.0,
        max_tokens in prop::option::of(1usize..100_000),
    ) {
        let cap = compute_file_cap(budget, pct, max_tokens);
        prop_assert!(cap > 0);
    }

    #[test]
    fn classify_lockfile(
        name in prop::sample::select(vec![
            "Cargo.lock", "package-lock.json", "yarn.lock", "poetry.lock",
            "Gemfile.lock", "pnpm-lock.yaml", "composer.lock",
        ])
    ) {
        let classes = classify_file(name, 100, 50, DEFAULT_DENSE_THRESHOLD);
        prop_assert!(classes.contains(&FileClassification::Lockfile));
    }

    #[test]
    fn assign_policy_under_cap(tokens in 1usize..100) {
        let cap = 1000;
        let (policy, _reason) = assign_policy(tokens, cap, &[]);
        prop_assert_eq!(policy, InclusionPolicy::Full);
    }

    #[test]
    fn compute_file_cap_bounded(budget in 1usize..1_000_000, pct in 0.01f64..1.0) {
        let cap = compute_file_cap(budget, pct, None);
        prop_assert!(cap <= budget);
    }

    #[test]
    fn classify_generated(
        name in prop::sample::select(vec![
            "generated.rs", "auto_generated.py", "codegen_output.ts",
            "src/generated/types.rs", "bindings_generated.h",
        ])
    ) {
        let classes = classify_file(name, 100, 50, DEFAULT_DENSE_THRESHOLD);
        prop_assert!(classes.contains(&FileClassification::Generated));
    }

}
