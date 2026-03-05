//! W53: Property-based tests for `tokmd-context-policy`.
//!
//! Covers: token budget calculations, classification determinism,
//! policy assignment invariants, and boundary conditions.

use proptest::prelude::*;
use tokmd_context_policy::{
    DEFAULT_DENSE_THRESHOLD, DEFAULT_MAX_FILE_PCT, DEFAULT_MAX_FILE_TOKENS, assign_policy,
    classify_file, compute_file_cap, is_spine_file, smart_exclude_reason,
};
use tokmd_types::{FileClassification, InclusionPolicy};

fn arbitrary_path() -> impl Strategy<Value = String> {
    "\\PC{0,256}".prop_map(|s| s.replace('\u{0000}', ""))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(150))]

    // 1. classify_file is idempotent (calling twice yields same result)
    #[test]
    fn classify_idempotent(
        path in arbitrary_path(),
        tokens in 0usize..200_000,
        lines in 0usize..20_000,
        threshold in 1.0f64..200.0,
    ) {
        let a = classify_file(&path, tokens, lines, threshold);
        let b = classify_file(&path, tokens, lines, threshold);
        prop_assert_eq!(a, b);
    }

    // 2. compute_file_cap with default constants never exceeds budget
    #[test]
    fn file_cap_default_bounded(budget in 0usize..10_000_000) {
        let cap = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, Some(DEFAULT_MAX_FILE_TOKENS));
        if budget < usize::MAX {
            prop_assert!(cap <= budget, "cap {} > budget {}", cap, budget);
        }
    }

    // 3. compute_file_cap result is at most hard_cap
    #[test]
    fn file_cap_at_most_hard_cap(
        budget in 1usize..10_000_000,
        pct in 0.01f64..1.0,
        hard_cap in 1usize..100_000,
    ) {
        let cap = compute_file_cap(budget, pct, Some(hard_cap));
        prop_assert!(cap <= hard_cap, "cap {} > hard_cap {}", cap, hard_cap);
    }

    // 4. assign_policy: Full policy never has reason
    #[test]
    fn full_policy_no_reason(
        tokens in 0usize..200_000,
        cap in 0usize..200_000,
    ) {
        let classifications = vec![];
        let (policy, reason) = assign_policy(tokens, cap, &classifications);
        if policy == InclusionPolicy::Full {
            prop_assert!(reason.is_none(), "Full policy should have no reason");
        }
    }

    // 5. assign_policy: non-Full policy always has reason
    #[test]
    fn non_full_policy_has_reason(
        tokens in 1usize..200_000,
        cap in 0usize..200_000,
    ) {
        let (policy, reason) = assign_policy(tokens, cap, &[FileClassification::Lockfile]);
        if policy != InclusionPolicy::Full {
            prop_assert!(reason.is_some(), "non-Full policy should have reason");
        }
    }

    // 6. smart_exclude_reason never panics on arbitrary strings
    #[test]
    fn smart_exclude_no_panic(path in "\\PC{0,500}") {
        let _ = smart_exclude_reason(&path);
    }

    // 7. is_spine_file never panics on arbitrary strings
    #[test]
    fn is_spine_no_panic(path in "\\PC{0,500}") {
        let _ = is_spine_file(&path);
    }

    // 8. classify_file never panics on zero lines
    #[test]
    fn classify_zero_lines_no_panic(
        path in arbitrary_path(),
        tokens in 0usize..200_000,
    ) {
        let _ = classify_file(&path, tokens, 0, DEFAULT_DENSE_THRESHOLD);
    }

    // 9. Known spine files always return true
    #[test]
    fn known_spine_files(
        name in prop::sample::select(vec![
            "README.md", "Cargo.toml", "package.json", "go.mod",
            "pyproject.toml", "CONTRIBUTING.md", "ROADMAP.md",
        ]),
        prefix in prop::option::of("[a-z]+(/[a-z]+){0,2}"),
    ) {
        let path = match prefix {
            Some(p) => format!("{}/{}", p, name),
            None => name.to_string(),
        };
        prop_assert!(is_spine_file(&path), "expected spine for {}", path);
    }

    // 10. Generated patterns are classified
    #[test]
    fn generated_pattern_classified(
        stem in "[a-z]{1,10}",
        pattern in prop::sample::select(vec![
            ".generated.", ".pb.go", ".pb.rs", "_pb2.py", ".g.dart", ".freezed.dart",
        ]),
    ) {
        let path = format!("{}{}", stem, pattern);
        let classes = classify_file(&path, 100, 50, DEFAULT_DENSE_THRESHOLD);
        prop_assert!(
            classes.contains(&FileClassification::Generated),
            "expected Generated for {}",
            path
        );
    }

    // 11. Vendored directories are classified
    #[test]
    fn vendored_dir_classified(
        dir in prop::sample::select(vec![
            "vendor/", "third_party/", "third-party/", "node_modules/",
        ]),
        file in "[a-z]{1,8}\\.[a-z]{1,4}",
    ) {
        let path = format!("{}{}", dir, file);
        let classes = classify_file(&path, 100, 50, DEFAULT_DENSE_THRESHOLD);
        prop_assert!(
            classes.contains(&FileClassification::Vendored),
            "expected Vendored for {}",
            path
        );
    }

    // 12. compute_file_cap: usize::MAX budget always returns usize::MAX
    #[test]
    fn umax_budget_returns_umax(
        pct in 0.01f64..1.0,
        hard_cap in prop::option::of(1usize..100_000),
    ) {
        let cap = compute_file_cap(usize::MAX, pct, hard_cap);
        prop_assert_eq!(cap, usize::MAX);
    }
}
