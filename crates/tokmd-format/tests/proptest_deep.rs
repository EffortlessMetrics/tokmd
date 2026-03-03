//! Deep property-based tests for tokmd-format.
//!
//! Covers: diff row symmetry, diff totals idempotency,
//! output determinism across formats, and structural invariants.

use proptest::prelude::*;

use tokmd_format::{compute_diff_rows, compute_diff_totals};
use tokmd_settings::ChildrenMode;
use tokmd_types::{LangReport, LangRow, Totals};

// =========================================================================
// Strategies
// =========================================================================

fn arb_lang_row() -> impl Strategy<Value = LangRow> {
    (
        prop::sample::select(vec![
            "Rust",
            "Python",
            "Go",
            "Java",
            "C",
            "TypeScript",
            "TOML",
            "YAML",
        ]),
        1usize..10_000,
        1usize..20_000,
        1usize..100,
    )
        .prop_map(|(lang, code, lines, files)| LangRow {
            lang: lang.to_string(),
            code,
            lines: lines.max(code),
            files,
            bytes: code * 10,
            tokens: code / 4,
            avg_lines: if files > 0 { lines / files } else { 0 },
        })
}

fn arb_lang_report() -> impl Strategy<Value = LangReport> {
    prop::collection::vec(arb_lang_row(), 1..8).prop_map(|rows| {
        let mut seen = std::collections::HashSet::new();
        let rows: Vec<LangRow> = rows
            .into_iter()
            .filter(|r| seen.insert(r.lang.clone()))
            .collect();
        let total = Totals {
            code: rows.iter().map(|r| r.code).sum(),
            lines: rows.iter().map(|r| r.lines).sum(),
            files: rows.iter().map(|r| r.files).sum(),
            bytes: rows.iter().map(|r| r.bytes).sum(),
            tokens: rows.iter().map(|r| r.tokens).sum(),
            avg_lines: 0,
        };
        LangReport {
            rows,
            total,
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        }
    })
}

// =========================================================================
// Diff: self-diff produces all-zero deltas
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn self_diff_produces_zero_deltas(report in arb_lang_report()) {
        let rows = compute_diff_rows(&report, &report);
        for row in &rows {
            prop_assert_eq!(row.delta_code, 0, "self-diff code delta should be 0 for {}", row.lang);
            prop_assert_eq!(row.delta_lines, 0, "self-diff lines delta should be 0 for {}", row.lang);
            prop_assert_eq!(row.delta_files, 0, "self-diff files delta should be 0 for {}", row.lang);
        }
        let totals = compute_diff_totals(&rows);
        prop_assert_eq!(totals.delta_code, 0);
        prop_assert_eq!(totals.delta_lines, 0);
        prop_assert_eq!(totals.delta_files, 0);
    }
}

// =========================================================================
// Diff: deltas are anti-symmetric (diff(a,b) = -diff(b,a))
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn diff_deltas_anti_symmetric(
        from in arb_lang_report(),
        to in arb_lang_report(),
    ) {
        let rows_ab = compute_diff_rows(&from, &to);
        let rows_ba = compute_diff_rows(&to, &from);
        let totals_ab = compute_diff_totals(&rows_ab);
        let totals_ba = compute_diff_totals(&rows_ba);

        prop_assert_eq!(
            totals_ab.delta_code, -totals_ba.delta_code,
            "delta_code should be anti-symmetric"
        );
        prop_assert_eq!(
            totals_ab.delta_lines, -totals_ba.delta_lines,
            "delta_lines should be anti-symmetric"
        );
        prop_assert_eq!(
            totals_ab.delta_files, -totals_ba.delta_files,
            "delta_files should be anti-symmetric"
        );
    }
}

// =========================================================================
// Diff: totals old/new match source reports
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn diff_totals_match_source_reports(
        from in arb_lang_report(),
        to in arb_lang_report(),
    ) {
        let rows = compute_diff_rows(&from, &to);
        let totals = compute_diff_totals(&rows);

        prop_assert_eq!(totals.old_code as usize, from.total.code);
        prop_assert_eq!(totals.new_code as usize, to.total.code);
        prop_assert_eq!(totals.old_lines as usize, from.total.lines);
        prop_assert_eq!(totals.new_lines as usize, to.total.lines);
    }
}

// =========================================================================
// Diff: compute_diff_rows is deterministic
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn diff_rows_are_deterministic(
        from in arb_lang_report(),
        to in arb_lang_report(),
    ) {
        let rows1 = compute_diff_rows(&from, &to);
        let rows2 = compute_diff_rows(&from, &to);
        prop_assert_eq!(rows1.len(), rows2.len());
        for (r1, r2) in rows1.iter().zip(rows2.iter()) {
            prop_assert_eq!(r1, r2, "Diff rows should be deterministic");
        }
    }
}

// =========================================================================
// Diff: row count covers union of languages
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn diff_row_count_is_union_of_languages(
        from in arb_lang_report(),
        to in arb_lang_report(),
    ) {
        let rows = compute_diff_rows(&from, &to);
        let mut all_langs: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for r in &from.rows { all_langs.insert(&r.lang); }
        for r in &to.rows { all_langs.insert(&r.lang); }
        prop_assert_eq!(rows.len(), all_langs.len(),
            "Diff rows should cover union of languages");
    }
}

// =========================================================================
// Diff: each row delta = new - old
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn diff_each_row_delta_consistent(
        from in arb_lang_report(),
        to in arb_lang_report(),
    ) {
        let rows = compute_diff_rows(&from, &to);
        for row in &rows {
            prop_assert_eq!(
                row.delta_code, row.new_code as i64 - row.old_code as i64,
                "delta_code mismatch for {}", row.lang
            );
            prop_assert_eq!(
                row.delta_lines, row.new_lines as i64 - row.old_lines as i64,
                "delta_lines mismatch for {}", row.lang
            );
        }
    }
}
