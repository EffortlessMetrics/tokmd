//! Property-based tests for path normalization invariants.

use proptest::prelude::*;
use tokmd_path::{normalize_rel_path, normalize_slashes};

// ── Strategies ───────────────────────────────────────────────────

/// Arbitrary printable strings (may contain backslashes, slashes, etc.).
fn arb_path() -> impl Strategy<Value = String> {
    "\\PC{0,120}"
}

/// Strings that look like plausible file paths.
fn realistic_path() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-zA-Z0-9_./ \\\\-]{1,20}", 1..=6)
        .prop_map(|segments| segments.join("/"))
}

/// Paths with a `./` or `.\` prefix.
fn dot_prefixed_path() -> impl Strategy<Value = String> {
    prop_oneof![Just("./".to_string()), Just(".\\".to_string()),]
        .prop_flat_map(|prefix| arb_path().prop_map(move |rest| format!("{prefix}{rest}")))
}

// ── normalize_slashes properties ─────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn no_backslashes_in_output(path in arb_path()) {
        let result = normalize_slashes(&path);
        prop_assert!(
            !result.contains('\\'),
            "output still contains backslash: {result:?}"
        );
    }

    #[test]
    fn idempotent(path in arb_path()) {
        let once = normalize_slashes(&path);
        let twice = normalize_slashes(&once);
        prop_assert_eq!(&once, &twice);
    }

    #[test]
    fn length_preserved_or_equal(path in arb_path()) {
        // Replacing `\` with `/` is a 1-to-1 char swap; length stays the same.
        let result = normalize_slashes(&path);
        prop_assert_eq!(path.len(), result.len());
    }

    #[test]
    fn forward_slashes_only_added_where_backslashes_were(path in arb_path()) {
        let result = normalize_slashes(&path);
        for (orig, norm) in path.chars().zip(result.chars()) {
            if orig != '\\' {
                prop_assert_eq!(orig, norm, "non-backslash char mutated");
            } else {
                prop_assert_eq!(norm, '/', "backslash not replaced with /");
            }
        }
    }

    #[test]
    fn realistic_paths_no_backslashes(path in realistic_path()) {
        let result = normalize_slashes(&path);
        prop_assert!(!result.contains('\\'));
    }
}

// ── normalize_rel_path properties ────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn rel_no_backslashes_in_output(path in arb_path()) {
        let result = normalize_rel_path(&path);
        prop_assert!(
            !result.contains('\\'),
            "output still contains backslash: {result:?}"
        );
    }

    #[test]
    fn rel_idempotent(path in arb_path()) {
        let once = normalize_rel_path(&path);
        let twice = normalize_rel_path(&once);
        prop_assert_eq!(&once, &twice);
    }

    #[test]
    fn rel_strips_exactly_one_leading_dot_slash(path in dot_prefixed_path()) {
        let slash_normalized = normalize_slashes(&path);
        let result = normalize_rel_path(&path);
        // The function strips exactly one leading `./` from the slash-normalized form.
        let expected = slash_normalized.strip_prefix("./").unwrap_or(&slash_normalized);
        prop_assert_eq!(&result, expected);
    }

    #[test]
    fn rel_output_is_substring_of_normalized_slashes(path in arb_path()) {
        let slash_normalized = normalize_slashes(&path);
        let rel_normalized = normalize_rel_path(&path);
        // rel_path either equals slash_normalized or is a suffix of it
        // (with the `./` prefix removed).
        prop_assert!(
            slash_normalized.ends_with(&rel_normalized),
            "rel output {rel_normalized:?} is not a suffix of slash output {slash_normalized:?}"
        );
    }

    #[test]
    fn rel_result_no_longer_than_slash_normalized(path in arb_path()) {
        let slash_only = normalize_slashes(&path);
        let rel = normalize_rel_path(&path);
        prop_assert!(
            rel.len() <= slash_only.len(),
            "rel output should be at most as long as slash-only output"
        );
    }
}
