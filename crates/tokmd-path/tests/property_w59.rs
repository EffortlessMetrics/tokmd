//! Wave-59 property-based tests for tokmd-path — idempotency, invariants,
//! and algebraic relationships between normalize functions.

use proptest::prelude::*;
use tokmd_path::{normalize_rel_path, normalize_slashes};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    // ── normalize_slashes properties ─────────────────────────────────────

    #[test]
    fn slashes_no_backslashes_in_output(path in "\\PC{0,200}") {
        let result = normalize_slashes(&path);
        prop_assert!(
            !result.contains('\\'),
            "output must not contain backslashes: {result:?}"
        );
    }

    #[test]
    fn slashes_idempotent(path in "\\PC{0,200}") {
        let once = normalize_slashes(&path);
        let twice = normalize_slashes(&once);
        prop_assert_eq!(&once, &twice);
    }

    #[test]
    fn slashes_preserves_length(path in "\\PC{0,200}") {
        let result = normalize_slashes(&path);
        prop_assert_eq!(
            result.len(), path.len(),
            "length must be preserved: input={}, output={}", path.len(), result.len()
        );
    }

    #[test]
    fn slashes_preserves_non_backslash_chars(path in "[a-zA-Z0-9/. _-]{0,100}") {
        // Paths without backslashes should pass through unchanged
        let result = normalize_slashes(&path);
        prop_assert_eq!(&result, &path);
    }

    #[test]
    fn slashes_forward_slash_count_gte_input(path in "\\PC{0,100}") {
        let input_fwd = path.chars().filter(|c| *c == '/').count();
        let input_back = path.chars().filter(|c| *c == '\\').count();
        let output_fwd = normalize_slashes(&path).chars().filter(|c| *c == '/').count();
        prop_assert_eq!(
            output_fwd, input_fwd + input_back,
            "output forward slashes should equal input forward + back slashes"
        );
    }

    // ── normalize_rel_path properties ────────────────────────────────────

    #[test]
    fn rel_no_backslashes_in_output(path in "\\PC{0,200}") {
        let result = normalize_rel_path(&path);
        prop_assert!(
            !result.contains('\\'),
            "output must not contain backslashes: {result:?}"
        );
    }

    #[test]
    fn rel_idempotent(path in "\\PC{0,200}") {
        let once = normalize_rel_path(&path);
        let twice = normalize_rel_path(&once);
        prop_assert_eq!(&once, &twice);
    }

    #[test]
    fn rel_no_leading_dot_slash(path in "\\PC{0,200}") {
        let result = normalize_rel_path(&path);
        prop_assert!(
            !result.starts_with("./"),
            "output must not start with './': {result:?}"
        );
    }

    #[test]
    fn rel_result_is_suffix_of_slashes(path in "\\PC{0,200}") {
        let slashed = normalize_slashes(&path);
        let rel = normalize_rel_path(&path);
        prop_assert!(
            slashed.ends_with(&rel),
            "normalize_rel_path result '{}' should be a suffix of normalize_slashes result '{}'",
            rel, slashed
        );
    }

    #[test]
    fn rel_length_lte_slashes(path in "\\PC{0,200}") {
        let slashed = normalize_slashes(&path);
        let rel = normalize_rel_path(&path);
        prop_assert!(
            rel.len() <= slashed.len(),
            "rel_path length {} should be <= slashes length {}",
            rel.len(), slashed.len()
        );
    }

    // ── cross-function properties ────────────────────────────────────────

    #[test]
    fn rel_path_then_slashes_is_identity_on_result(path in "\\PC{0,200}") {
        let rel = normalize_rel_path(&path);
        let slashed_of_rel = normalize_slashes(&rel);
        // normalize_slashes on already-normalized rel should be no-op
        prop_assert_eq!(&rel, &slashed_of_rel);
    }

    #[test]
    fn slashes_then_rel_equals_rel(path in "\\PC{0,200}") {
        let slashed = normalize_slashes(&path);
        let rel_of_slashed = normalize_rel_path(&slashed);
        let rel_direct = normalize_rel_path(&path);
        prop_assert_eq!(
            &rel_of_slashed, &rel_direct,
            "normalize_rel_path should commute with normalize_slashes"
        );
    }
}
