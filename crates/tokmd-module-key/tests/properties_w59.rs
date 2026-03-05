//! W59 — property-based tests for module-key determinism and invariants.

use proptest::prelude::*;
use tokmd_module_key::{module_key, module_key_from_normalized};

proptest! {
    // ── Determinism ──────────────────────────────────────────────────

    #[test]
    fn deterministic_same_input_same_output(
        dirs in prop::collection::vec("[a-z]{1,6}", 1..6),
        filename in "[a-z]{1,6}\\.[a-z]{1,3}",
        depth in 0usize..8,
    ) {
        let path = format!("{}/{}", dirs.join("/"), filename);
        let roots = vec![dirs[0].clone()];
        let k1 = module_key(&path, &roots, depth);
        let k2 = module_key(&path, &roots, depth);
        prop_assert_eq!(k1, k2);
    }

    #[test]
    fn deterministic_from_normalized(
        dirs in prop::collection::vec("[a-z]{1,6}", 1..5),
        filename in "[a-z]{1,6}\\.[a-z]{1,3}",
        depth in 1usize..6,
    ) {
        let path = format!("{}/{}", dirs.join("/"), filename);
        let roots: Vec<String> = vec![];
        let k1 = module_key_from_normalized(&path, &roots, depth);
        let k2 = module_key_from_normalized(&path, &roots, depth);
        prop_assert_eq!(k1, k2);
    }

    // ── Output never contains backslashes ────────────────────────────

    #[test]
    fn output_no_backslash(
        path in "[a-zA-Z0-9_./\\\\]{1,60}",
        depth in 0usize..8,
    ) {
        let roots = vec!["crates".into(), "src".into()];
        let key = module_key(&path, &roots, depth);
        prop_assert!(!key.contains('\\'), "backslash in key: {key:?}");
    }

    // ── Output never empty ───────────────────────────────────────────

    #[test]
    fn output_never_empty(
        path in "[a-zA-Z0-9_./\\\\]{0,60}",
        depth in 0usize..8,
    ) {
        let key = module_key(&path, &[], depth);
        prop_assert!(!key.is_empty());
    }

    // ── Output never ends with slash ─────────────────────────────────

    #[test]
    fn output_no_trailing_slash(
        dirs in prop::collection::vec("[a-z]{1,5}", 1..6),
        filename in "[a-z]{1,5}\\.[a-z]{1,3}",
        depth in 1usize..8,
    ) {
        let path = format!("{}/{}", dirs.join("/"), filename);
        let key = module_key(&path, &[], depth);
        prop_assert!(!key.ends_with('/'), "trailing slash: {key:?}");
    }

    // ── Output never starts with slash ───────────────────────────────

    #[test]
    fn output_no_leading_slash(
        path in "[a-zA-Z0-9_./\\\\]{1,60}\\.[a-z]{1,3}",
        depth in 0usize..6,
    ) {
        let key = module_key(&path, &[], depth);
        prop_assert!(!key.starts_with('/'), "leading slash: {key:?}");
    }

    // ── Depth monotonicity ───────────────────────────────────────────

    #[test]
    fn depth_monotonic_segment_count(
        root in "[a-z]{1,4}",
        subdirs in prop::collection::vec("[a-z]{1,4}", 3..7),
        filename in "[a-z]{1,4}\\.[a-z]{1,3}",
    ) {
        let path = format!("{}/{}/{}", root, subdirs.join("/"), filename);
        let roots = vec![root.clone()];
        let mut prev = 0;
        for d in 0..=7 {
            let segs = module_key(&path, &roots, d).split('/').count();
            prop_assert!(segs >= prev, "depth {d}: {segs} < {prev}");
            prev = segs;
        }
    }

    // ── Key segments are subset of path dirs ─────────────────────────

    #[test]
    fn key_segments_from_path(
        dirs in prop::collection::vec("[a-z]{2,6}", 2..5),
        filename in "[a-z]{2,6}\\.[a-z]{1,3}",
        depth in 1usize..5,
    ) {
        let path = format!("{}/{}", dirs.join("/"), filename);
        let roots = vec![dirs[0].clone()];
        let key = module_key(&path, &roots, depth);
        for seg in key.split('/') {
            prop_assert!(
                dirs.contains(&seg.to_string()),
                "key segment {seg:?} not in dirs {dirs:?}"
            );
        }
    }

    // ── Separator normalization ──────────────────────────────────────

    #[test]
    fn backslash_and_forward_slash_equivalent(
        parts in prop::collection::vec("[a-z]{1,5}", 2..5),
        filename in "[a-z]{1,5}\\.[a-z]{1,3}",
        depth in 1usize..5,
    ) {
        let fwd = format!("{}/{}", parts.join("/"), filename);
        let bck = format!("{}\\{}", parts.join("\\"), filename);
        let roots: Vec<String> = vec![];
        prop_assert_eq!(module_key(&fwd, &roots, depth), module_key(&bck, &roots, depth));
    }

    // ── Dot-slash prefix idempotence ─────────────────────────────────

    #[test]
    fn dot_slash_prefix_stripped(
        dirs in prop::collection::vec("[a-z]{1,5}", 1..4),
        filename in "[a-z]{1,5}\\.[a-z]{1,3}",
        depth in 1usize..4,
    ) {
        let plain = format!("{}/{}", dirs.join("/"), filename);
        let dotted = format!("./{}/{}", dirs.join("/"), filename);
        let roots: Vec<String> = vec![];
        prop_assert_eq!(module_key(&plain, &roots, depth), module_key(&dotted, &roots, depth));
    }

    // ── Root file detection ──────────────────────────────────────────

    #[test]
    fn root_files_always_root(
        filename in "[a-zA-Z][a-zA-Z0-9_]{0,10}\\.[a-z]{1,4}",
    ) {
        let key = module_key(&filename, &[], 2);
        prop_assert_eq!(key, "(root)");
    }

    // ── Depth 0 equals depth 1 ───────────────────────────────────────

    #[test]
    fn depth_zero_equals_one(
        root in "[a-z]{1,4}",
        subdirs in prop::collection::vec("[a-z]{1,4}", 1..4),
        filename in "[a-z]{1,4}\\.[a-z]{1,3}",
    ) {
        let path = format!("{}/{}/{}", root, subdirs.join("/"), filename);
        let roots = vec![root.clone()];
        prop_assert_eq!(
            module_key(&path, &roots, 0),
            module_key(&path, &roots, 1)
        );
    }
}
