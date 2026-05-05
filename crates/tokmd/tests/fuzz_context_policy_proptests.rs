//! Deterministic property tests extracted from `fuzz_context_policy`.
//!
//! Validates invariants for:
//! - File classification
//! - File cap computations
//! - Policy assignments

use proptest::prelude::*;
use tokmd_core::context_policy::{
    DEFAULT_DENSE_THRESHOLD, DEFAULT_MAX_FILE_PCT, assign_policy, classify_file, compute_file_cap,
    is_spine_file, smart_exclude_reason,
};
use tokmd_types::InclusionPolicy;

proptest! {
    #[test]
    fn context_policy_invariants(
        path in "\\PC+",
        tokens in 0usize..1_000_000,
        lines in 0usize..1_000_000,
        budget in 0usize..1_000_000,
    ) {
        let _ = is_spine_file(path.as_ref());

        if let Some(reason) = smart_exclude_reason(path.as_ref()) {
            prop_assert!(matches!(reason, "lockfile" | "minified" | "sourcemap"));
        }

        let classes = classify_file(path.as_ref(), tokens, lines, DEFAULT_DENSE_THRESHOLD);
        let mut sorted = classes.clone();
        sorted.sort();
        sorted.dedup();
        prop_assert_eq!(&classes, &sorted);

        let cap_default = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, None);
        let cap_hard = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, Some(4_000));
        prop_assert!(cap_hard <= 4_000 || cap_hard == usize::MAX);

        let (policy_default, reason_default) = assign_policy(tokens, cap_default, &classes);
        match policy_default {
            InclusionPolicy::Full => {
                prop_assert!(tokens <= cap_default);
                prop_assert!(reason_default.is_none());
            }
            InclusionPolicy::HeadTail | InclusionPolicy::Skip => {
                if cap_default != usize::MAX {
                    prop_assert!(tokens > cap_default);
                    prop_assert!(reason_default.is_some());
                }
            }
            InclusionPolicy::Summary => {}
        }
    }
}
