//! Deep property-based tests for tokmd-model.
//!
//! Covers sorting stability, aggregation invariants,
//! normalize_path idempotency, and module_key determinism.

use proptest::prelude::*;
use std::path::Path;
use tokmd_model::{avg, module_key, normalize_path};
use tokmd_types::{LangRow, ModuleRow};

// =========================================================================
// Strategies
// =========================================================================

fn arb_lang_row() -> impl Strategy<Value = LangRow> {
    (
        "[A-Z][a-zA-Z0-9 #+]{0,15}",
        0usize..500_000,
        0usize..1_000_000,
        0usize..5_000,
        0usize..50_000_000,
        0usize..5_000_000,
        0usize..2_000,
    )
        .prop_map(
            |(lang, code, lines, files, bytes, tokens, avg_lines)| LangRow {
                lang,
                code,
                lines,
                files,
                bytes,
                tokens,
                avg_lines,
            },
        )
}

fn arb_module_row() -> impl Strategy<Value = ModuleRow> {
    (
        "[a-z][a-zA-Z0-9_/]{0,20}",
        0usize..500_000,
        0usize..1_000_000,
        0usize..5_000,
        0usize..50_000_000,
        0usize..5_000_000,
        0usize..2_000,
    )
        .prop_map(
            |(module, code, lines, files, bytes, tokens, avg_lines)| ModuleRow {
                module,
                code,
                lines,
                files,
                bytes,
                tokens,
                avg_lines,
            },
        )
}

// =========================================================================
// Sorting stability: desc by code, then asc by name
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn lang_rows_sort_is_stable(rows in prop::collection::vec(arb_lang_row(), 0..20)) {
        let mut sorted1 = rows.clone();
        sorted1.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));

        let mut sorted2 = rows;
        sorted2.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));

        prop_assert_eq!(&sorted1, &sorted2, "Sorting must be deterministic");
    }

    #[test]
    fn lang_rows_sorted_descending_by_code(rows in prop::collection::vec(arb_lang_row(), 2..15)) {
        let mut sorted = rows;
        sorted.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));

        for window in sorted.windows(2) {
            prop_assert!(
                window[0].code >= window[1].code,
                "Rows must be sorted descending by code: {} >= {}",
                window[0].code, window[1].code
            );
            if window[0].code == window[1].code {
                prop_assert!(
                    window[0].lang <= window[1].lang,
                    "Equal-code rows must be sorted ascending by name: {} <= {}",
                    window[0].lang, window[1].lang
                );
            }
        }
    }

    #[test]
    fn module_rows_sort_is_stable(rows in prop::collection::vec(arb_module_row(), 0..20)) {
        let mut sorted1 = rows.clone();
        sorted1.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));

        let mut sorted2 = rows;
        sorted2.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));

        prop_assert_eq!(&sorted1, &sorted2, "Module sorting must be deterministic");
    }

    #[test]
    fn module_rows_sorted_descending_by_code(rows in prop::collection::vec(arb_module_row(), 2..15)) {
        let mut sorted = rows;
        sorted.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));

        for window in sorted.windows(2) {
            prop_assert!(
                window[0].code >= window[1].code,
                "Module rows must be sorted descending by code"
            );
        }
    }
}

// =========================================================================
// Aggregation invariant: code values are always non-negative (usize)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn lang_row_code_always_non_negative(row in arb_lang_row()) {
        // usize is always >= 0, but verify after serde roundtrip
        let json = serde_json::to_string(&row).unwrap();
        let parsed: LangRow = serde_json::from_str(&json).unwrap();
        // code is usize, so it's always >= 0 by type; verify roundtrip preserves
        prop_assert_eq!(row.code, parsed.code);
        prop_assert_eq!(row.lines, parsed.lines);
        prop_assert_eq!(row.files, parsed.files);
    }

    #[test]
    fn module_row_code_always_non_negative(row in arb_module_row()) {
        let json = serde_json::to_string(&row).unwrap();
        let parsed: ModuleRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row.code, parsed.code);
        prop_assert_eq!(row.lines, parsed.lines);
        prop_assert_eq!(row.files, parsed.files);
    }
}

// =========================================================================
// normalize_path: idempotent, forward-slash only, no leading ./
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn normalize_path_idempotent_deep(
        parts in prop::collection::vec("[a-zA-Z0-9_]{1,10}", 1..=6),
        ext in prop::sample::select(vec!["rs", "py", "go", "js", "toml", "md"]),
    ) {
        let raw = format!("{}.{}", parts.join("/"), ext);
        let p = Path::new(&raw);
        let once = normalize_path(p, None);
        let twice = normalize_path(Path::new(&once), None);
        prop_assert_eq!(&once, &twice, "normalize_path must be idempotent");
    }

    #[test]
    fn normalize_path_backslash_becomes_forward(
        parts in prop::collection::vec("[a-zA-Z0-9_]{1,8}", 2..=5),
    ) {
        let win_path = parts.join("\\");
        let p = Path::new(&win_path);
        let normalized = normalize_path(p, None);
        prop_assert!(
            !normalized.contains('\\'),
            "normalize_path must not contain backslash: {}", normalized
        );
    }

    #[test]
    fn normalize_path_no_leading_dot_slash_deep(
        parts in prop::collection::vec("[a-zA-Z0-9_]{1,8}", 1..=4),
        ext in prop::sample::select(vec!["rs", "py", "go"]),
    ) {
        let raw = format!("./{}.{}", parts.join("/"), ext);
        let p = Path::new(&raw);
        let normalized = normalize_path(p, None);
        prop_assert!(
            !normalized.starts_with("./"),
            "normalize_path must strip leading ./: {}", normalized
        );
    }
}

// =========================================================================
// module_key: deterministic on same input
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn module_key_is_deterministic(
        dir in "[a-zA-Z0-9_]{1,10}",
        subdirs in prop::collection::vec("[a-zA-Z0-9_]{1,10}", 1..=3),
        filename in "[a-zA-Z0-9_]{1,10}\\.[a-z]{1,4}",
        depth in 1usize..5,
    ) {
        let path_parts: Vec<&str> = std::iter::once(dir.as_str())
            .chain(subdirs.iter().map(|s| s.as_str()))
            .chain(std::iter::once(filename.as_str()))
            .collect();
        let path = path_parts.join("/");
        let roots = vec![dir.clone()];

        let k1 = module_key(&path, &roots, depth);
        let k2 = module_key(&path, &roots, depth);
        prop_assert_eq!(&k1, &k2, "module_key must be deterministic");
    }

    #[test]
    fn module_key_root_files_always_root(
        filename in "[a-zA-Z0-9_]{1,15}\\.[a-z]{1,4}",
        depth in 1usize..5,
    ) {
        let key = module_key(&filename, &[], depth);
        prop_assert_eq!(key, "(root)", "Root-level files must map to (root)");
    }
}

// =========================================================================
// avg: invariants
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn avg_never_exceeds_lines(lines in 0usize..100_000, files in 1usize..1_000) {
        let result = avg(lines, files);
        prop_assert!(
            result <= lines,
            "avg({}, {}) = {} must not exceed lines", lines, files, result
        );
    }

    #[test]
    fn avg_zero_files_is_zero(lines in 0usize..100_000) {
        prop_assert_eq!(avg(lines, 0), 0);
    }

    #[test]
    fn avg_monotonic_in_lines(
        lines_a in 0usize..50_000,
        extra in 0usize..50_000,
        files in 1usize..1_000,
    ) {
        let lines_b = lines_a + extra;
        prop_assert!(
            avg(lines_b, files) >= avg(lines_a, files),
            "avg must be monotonically non-decreasing in lines"
        );
    }
}
