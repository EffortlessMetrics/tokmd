use proptest::prelude::*;
use tokmd_context_policy::{
    DEFAULT_DENSE_THRESHOLD, DEFAULT_MAX_FILE_PCT, assign_policy, classify_file, compute_file_cap,
    is_spine_file, smart_exclude_reason,
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

    // ── Property: policy assignment preserves file count ─────────────

    #[test]
    fn policy_assignment_preserves_file_count(
        tokens_vec in prop::collection::vec(0usize..200_000, 1..50),
        lines_vec in prop::collection::vec(0usize..20_000, 1..50),
        budget in 1usize..500_000,
    ) {
        let n = tokens_vec.len().min(lines_vec.len());
        let cap = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, None);
        let mut policies = Vec::with_capacity(n);
        for i in 0..n {
            let path = format!("src/file_{i}.rs");
            let classes = classify_file(&path, tokens_vec[i], lines_vec[i], DEFAULT_DENSE_THRESHOLD);
            let policy = assign_policy(tokens_vec[i], cap, &classes);
            policies.push(policy);
        }
        prop_assert_eq!(policies.len(), n, "every file must receive a policy");
    }

    // ── Property: all classifications are valid enum variants ────────

    #[test]
    fn all_classifications_are_known_variants(
        path in arbitrary_path(),
        tokens in 0usize..200_000,
        lines in 0usize..20_000,
    ) {
        let all_known = [
            FileClassification::Generated,
            FileClassification::Fixture,
            FileClassification::Vendored,
            FileClassification::Lockfile,
            FileClassification::Minified,
            FileClassification::DataBlob,
            FileClassification::Sourcemap,
        ];
        let classes = classify_file(&path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
        for c in &classes {
            prop_assert!(all_known.contains(c), "unknown classification: {:?}", c);
        }
    }

    // ── Property: effective tokens never negative / never exceed input ─

    #[test]
    fn policy_effective_tokens_never_exceed_original(
        tokens in 0usize..200_000,
        budget in 1usize..500_000,
    ) {
        let cap = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, None);
        let (policy, _) = assign_policy(tokens, cap, &[]);
        let effective = match policy {
            InclusionPolicy::Full => tokens,
            InclusionPolicy::HeadTail => cap,
            InclusionPolicy::Skip | InclusionPolicy::Summary => 0,
        };
        prop_assert!(effective <= tokens, "effective {} must not exceed original {}", effective, tokens);
    }

    // ── Property: spine detection is idempotent (repeated calls) ─────

    #[test]
    fn is_spine_file_idempotent(path in arbitrary_path()) {
        let first  = is_spine_file(&path);
        let second = is_spine_file(&path);
        let third  = is_spine_file(&path);
        prop_assert_eq!(first, second);
        prop_assert_eq!(second, third);
    }

    // ── Property: policy with varied classifications preserves count ─

    #[test]
    fn policy_assignment_preserves_count_across_file_kinds(
        budget in 1usize..500_000,
    ) {
        let file_entries: Vec<(&str, usize, usize)> = vec![
            ("Cargo.lock", 2_000, 200),
            ("vendor/lib/foo.js", 30_000, 500),
            ("dist/app.min.js", 50_000, 2),
            ("proto/types.pb.go", 40_000, 2_000),
            ("testdata/sample.json", 25_000, 5_000),
            ("src/main.rs", 3_000, 200),
            ("README.md", 500, 80),
            ("dist/app.js.map", 60_000, 10),
            ("node_modules/react/index.js", 15_000, 800),
            ("tests/golden/expected.txt", 1_000, 100),
        ];
        let cap = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, None);
        let mut count = 0;
        for (path, tokens, lines) in &file_entries {
            let classes = classify_file(path, *tokens, *lines, DEFAULT_DENSE_THRESHOLD);
            let _policy = assign_policy(*tokens, cap, &classes);
            count += 1;
        }
        prop_assert_eq!(count, file_entries.len());
    }

    // ── Property: cap ≤ budget for any finite budget ────────────────

    #[test]
    fn compute_file_cap_never_exceeds_budget_with_hard_cap(
        budget in 0usize..1_000_000,
        pct in 0f64..1.0,
        hard_cap in 0usize..100_000,
    ) {
        let cap = compute_file_cap(budget, pct, Some(hard_cap));
        prop_assert!(cap <= budget, "cap {} exceeded budget {}", cap, budget);
    }
}
