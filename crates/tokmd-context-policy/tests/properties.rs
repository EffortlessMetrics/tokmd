use proptest::prelude::*;
use tokmd_context_policy::{
    DEFAULT_DENSE_THRESHOLD, assign_policy, classify_file, compute_file_cap, is_spine_file,
    smart_exclude_reason,
};
use tokmd_types::InclusionPolicy;

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
}
