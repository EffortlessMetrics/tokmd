//! Wave-62 depth tests for `tokmd-analysis-derived`.
//!
//! Covers density calculations, distribution metrics, COCOMO model,
//! zero-input edge cases, single-file repos, large repos, property tests,
//! determinism, and snapshot tests for formatted metric output.

use proptest::prelude::*;
use tokmd_analysis_derived::derive_report;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ───────────────────── helpers ─────────────────────

fn row(path: &str, lang: &str, code: usize, comments: usize, blanks: usize) -> FileRow {
    let lines = code + comments + blanks;
    FileRow {
        path: path.to_string(),
        module: path
            .rsplit_once('/')
            .map(|(m, _)| m.to_string())
            .unwrap_or_default(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments,
        blanks,
        lines,
        bytes: lines * 40,
        tokens: lines * 5,
    }
}

fn row_with_bytes(
    path: &str,
    lang: &str,
    code: usize,
    comments: usize,
    blanks: usize,
    bytes: usize,
) -> FileRow {
    let lines = code + comments + blanks;
    FileRow {
        path: path.to_string(),
        module: path
            .rsplit_once('/')
            .map(|(m, _)| m.to_string())
            .unwrap_or_default(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments,
        blanks,
        lines,
        bytes,
        tokens: lines * 5,
    }
}

fn export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

// ═══════════════════════════════════════════════════════════════
// 1. Density calculations
// ═══════════════════════════════════════════════════════════════

#[test]
fn code_density_pure_code_file() {
    let r = derive_report(&export(vec![row("a.rs", "Rust", 100, 0, 0)]), None);
    // doc_density = comments/(code+comments) = 0/100 = 0
    assert_eq!(r.doc_density.total.ratio, 0.0);
    // whitespace = blanks/(code+comments) = 0/100 = 0
    assert_eq!(r.whitespace.total.ratio, 0.0);
}

#[test]
fn comment_density_all_comments() {
    let r = derive_report(&export(vec![row("a.rs", "Rust", 0, 100, 0)]), None);
    // comments/(code+comments) = 100/100 = 1.0
    assert_eq!(r.doc_density.total.ratio, 1.0);
}

#[test]
fn blank_density_calculation() {
    let r = derive_report(&export(vec![row("a.rs", "Rust", 60, 30, 10)]), None);
    // whitespace = blanks/(code+comments) = 10/90
    assert_eq!(r.whitespace.total.numerator, 10);
    assert_eq!(r.whitespace.total.denominator, 90);
    let expected = 10.0 / 90.0;
    assert!((r.whitespace.total.ratio - expected).abs() < 0.001);
}

#[test]
fn density_across_multiple_files() {
    let r = derive_report(
        &export(vec![
            row("a.rs", "Rust", 80, 20, 10),
            row("b.rs", "Rust", 100, 0, 0),
        ]),
        None,
    );
    // total comments = 20, total code+comments = 200
    assert_eq!(r.doc_density.total.numerator, 20);
    assert_eq!(r.doc_density.total.denominator, 200);
    assert!((r.doc_density.total.ratio - 0.1).abs() < 0.001);
}

#[test]
fn density_by_lang_has_entries_for_each_language() {
    let r = derive_report(
        &export(vec![
            row("a.rs", "Rust", 80, 20, 5),
            row("b.py", "Python", 50, 50, 10),
            row("c.js", "JavaScript", 30, 10, 5),
        ]),
        None,
    );
    assert_eq!(r.doc_density.by_lang.len(), 3);
}

#[test]
fn density_by_module_has_entries_for_each_module() {
    let r = derive_report(
        &export(vec![
            row("src/a.rs", "Rust", 80, 20, 5),
            row("lib/b.rs", "Rust", 50, 50, 10),
        ]),
        None,
    );
    assert!(r.doc_density.by_module.len() >= 2);
}

#[test]
fn doc_density_half_and_half() {
    let r = derive_report(&export(vec![row("a.rs", "Rust", 50, 50, 0)]), None);
    assert!((r.doc_density.total.ratio - 0.5).abs() < 0.001);
}

#[test]
fn whitespace_zero_when_no_blanks() {
    let r = derive_report(&export(vec![row("a.rs", "Rust", 100, 50, 0)]), None);
    assert_eq!(r.whitespace.total.numerator, 0);
    assert_eq!(r.whitespace.total.ratio, 0.0);
}

// ═══════════════════════════════════════════════════════════════
// 2. Distribution metrics
// ═══════════════════════════════════════════════════════════════

#[test]
fn distribution_two_files_median() {
    let r = derive_report(
        &export(vec![
            row("a.rs", "Rust", 10, 0, 0),
            row("b.rs", "Rust", 20, 0, 0),
        ]),
        None,
    );
    // Median of [10, 20] = (10+20)/2 = 15.0
    assert_eq!(r.distribution.median, 15.0);
}

#[test]
fn distribution_four_files_stats() {
    let r = derive_report(
        &export(vec![
            row("a.rs", "Rust", 10, 0, 0),
            row("b.rs", "Rust", 20, 0, 0),
            row("c.rs", "Rust", 30, 0, 0),
            row("d.rs", "Rust", 40, 0, 0),
        ]),
        None,
    );
    assert_eq!(r.distribution.count, 4);
    assert_eq!(r.distribution.min, 10);
    assert_eq!(r.distribution.max, 40);
    assert!((r.distribution.mean - 25.0).abs() < 0.01);
}

#[test]
fn distribution_p90_p99_present() {
    let rows: Vec<FileRow> = (1..=100)
        .map(|i| row(&format!("f{i}.rs"), "Rust", i * 10, 0, 0))
        .collect();
    let r = derive_report(&export(rows), None);
    assert!(r.distribution.p90 > 0.0);
    assert!(r.distribution.p99 > 0.0);
    assert!(r.distribution.p99 >= r.distribution.p90);
}

#[test]
fn distribution_gini_max_inequality() {
    // One file with all lines, rest with minimal
    let mut rows = vec![row("big.rs", "Rust", 10000, 0, 0)];
    for i in 0..99 {
        rows.push(row(&format!("s{i}.rs"), "Rust", 1, 0, 0));
    }
    let r = derive_report(&export(rows), None);
    assert!(
        r.distribution.gini > 0.5,
        "high inequality should give high gini"
    );
}

#[test]
fn distribution_identical_sizes_gini_zero() {
    let rows: Vec<FileRow> = (0..10)
        .map(|i| row(&format!("f{i}.rs"), "Rust", 50, 0, 0))
        .collect();
    let r = derive_report(&export(rows), None);
    assert_eq!(r.distribution.gini, 0.0);
}

// ═══════════════════════════════════════════════════════════════
// 3. COCOMO model calculations
// ═══════════════════════════════════════════════════════════════

#[test]
fn cocomo_none_for_zero_code() {
    let r = derive_report(&export(vec![row("a.rs", "Rust", 0, 100, 50)]), None);
    assert!(r.cocomo.is_none());
}

#[test]
fn cocomo_small_project() {
    let r = derive_report(&export(vec![row("a.rs", "Rust", 500, 0, 0)]), None);
    let c = r.cocomo.unwrap();
    assert_eq!(c.kloc, 0.5);
    // Manual: effort = 2.4 * 0.5^1.05
    let expected_effort = 2.4 * 0.5_f64.powf(1.05);
    assert!((c.effort_pm - expected_effort).abs() < 0.1);
}

#[test]
fn cocomo_100k_loc() {
    let r = derive_report(&export(vec![row("a.rs", "Rust", 100_000, 0, 0)]), None);
    let c = r.cocomo.unwrap();
    assert_eq!(c.kloc, 100.0);
    assert!(
        c.effort_pm > 200.0,
        "100 KLOC should require significant effort"
    );
    assert!(c.duration_months > 10.0);
}

#[test]
fn cocomo_multi_file_aggregation() {
    let r = derive_report(
        &export(vec![
            row("a.rs", "Rust", 3000, 0, 0),
            row("b.rs", "Rust", 2000, 0, 0),
        ]),
        None,
    );
    let c = r.cocomo.unwrap();
    assert_eq!(c.kloc, 5.0);
}

#[test]
fn cocomo_staff_positive() {
    let r = derive_report(&export(vec![row("a.rs", "Rust", 10000, 0, 0)]), None);
    let c = r.cocomo.unwrap();
    assert!(c.staff > 0.0);
    // staff = effort / duration
    let expected_staff = c.effort_pm / c.duration_months;
    assert!((c.staff - expected_staff).abs() < 0.01);
}

#[test]
fn cocomo_effort_superlinear() {
    // Effort grows superlinearly with code size (b > 1)
    let r1 = derive_report(&export(vec![row("a.rs", "Rust", 1000, 0, 0)]), None);
    let r2 = derive_report(&export(vec![row("a.rs", "Rust", 2000, 0, 0)]), None);
    let c1 = r1.cocomo.unwrap();
    let c2 = r2.cocomo.unwrap();
    // If linear, effort doubles. With b=1.05, it more than doubles.
    assert!(c2.effort_pm > 2.0 * c1.effort_pm);
}

#[test]
fn cocomo_1_loc() {
    let r = derive_report(&export(vec![row("a.rs", "Rust", 1, 0, 0)]), None);
    let c = r.cocomo.unwrap();
    assert_eq!(c.kloc, 0.001);
    // With 0.001 KLOC, effort = 2.4 * 0.001^1.05 ≈ 0.0018 which rounds to 0.0
    assert!(c.effort_pm >= 0.0);
}

// ═══════════════════════════════════════════════════════════════
// 4. Zero-input edge cases
// ═══════════════════════════════════════════════════════════════

#[test]
fn empty_export_all_zeros() {
    let r = derive_report(&export(vec![]), None);
    assert_eq!(r.totals.files, 0);
    assert_eq!(r.totals.code, 0);
    assert_eq!(r.totals.lines, 0);
    assert_eq!(r.totals.bytes, 0);
    assert_eq!(r.totals.tokens, 0);
}

#[test]
fn empty_export_reading_time_zero() {
    let r = derive_report(&export(vec![]), None);
    assert_eq!(r.reading_time.minutes, 0.0);
}

#[test]
fn empty_export_polyglot_zero_langs() {
    let r = derive_report(&export(vec![]), None);
    assert_eq!(r.polyglot.lang_count, 0);
    assert_eq!(r.polyglot.entropy, 0.0);
}

#[test]
fn empty_export_histogram_all_zeros() {
    let r = derive_report(&export(vec![]), None);
    for bucket in &r.histogram {
        assert_eq!(bucket.files, 0);
    }
}

#[test]
fn empty_export_integrity_zero_entries() {
    let r = derive_report(&export(vec![]), None);
    assert_eq!(r.integrity.entries, 0);
    assert_eq!(r.integrity.algo, "blake3");
}

#[test]
fn all_zero_line_files() {
    let r = derive_report(
        &export(vec![
            row("a.rs", "Rust", 0, 0, 0),
            row("b.rs", "Rust", 0, 0, 0),
        ]),
        None,
    );
    assert_eq!(r.totals.files, 2);
    assert_eq!(r.totals.lines, 0);
    assert!(r.cocomo.is_none());
    assert_eq!(r.distribution.min, 0);
    assert_eq!(r.distribution.max, 0);
}

#[test]
fn child_rows_excluded_from_totals() {
    let mut child = row("child.rs", "Rust", 500, 0, 0);
    child.kind = FileKind::Child;
    let r = derive_report(
        &export(vec![row("parent.rs", "Rust", 100, 0, 0), child]),
        None,
    );
    assert_eq!(r.totals.files, 1);
    assert_eq!(r.totals.code, 100);
}

// ═══════════════════════════════════════════════════════════════
// 5. Single-file repos
// ═══════════════════════════════════════════════════════════════

#[test]
fn single_file_all_metrics_populated() {
    let r = derive_report(
        &export(vec![row("src/main.rs", "Rust", 200, 50, 30)]),
        Some(100_000),
    );
    assert_eq!(r.totals.files, 1);
    assert!(r.cocomo.is_some());
    assert!(r.context_window.is_some());
    assert_eq!(r.distribution.count, 1);
    assert_eq!(r.polyglot.lang_count, 1);
    assert!(!r.integrity.hash.is_empty());
}

#[test]
fn single_file_max_file_is_itself() {
    let r = derive_report(&export(vec![row("src/main.rs", "Rust", 100, 10, 5)]), None);
    assert_eq!(r.max_file.overall.path, "src/main.rs");
}

#[test]
fn single_file_nesting_depth() {
    let r = derive_report(
        &export(vec![row("src/deep/nested/file.rs", "Rust", 100, 0, 0)]),
        None,
    );
    assert!(r.nesting.max > 0);
    assert!(r.nesting.avg > 0.0);
}

#[test]
fn single_file_test_density() {
    let r = derive_report(
        &export(vec![row("tests/test_main.rs", "Rust", 100, 0, 0)]),
        None,
    );
    assert_eq!(r.test_density.test_files, 1);
    assert_eq!(r.test_density.prod_files, 0);
    assert_eq!(r.test_density.test_lines, 100);
}

// ═══════════════════════════════════════════════════════════════
// 6. Large repos with many files
// ═══════════════════════════════════════════════════════════════

#[test]
fn large_repo_2000_files() {
    let rows: Vec<FileRow> = (0..2000)
        .map(|i| {
            row(
                &format!("src/mod{}/file{}.rs", i / 50, i),
                "Rust",
                10 + i % 200,
                i % 20,
                i % 10,
            )
        })
        .collect();
    let r = derive_report(&export(rows), None);
    assert_eq!(r.totals.files, 2000);
    assert_eq!(r.distribution.count, 2000);
    assert!(r.cocomo.is_some());
}

#[test]
fn large_repo_histogram_sums_to_total() {
    let rows: Vec<FileRow> = (0..500)
        .map(|i| row(&format!("f{i}.rs"), "Rust", 1 + i * 5, 0, 0))
        .collect();
    let r = derive_report(&export(rows), None);
    let total_in_histogram: usize = r.histogram.iter().map(|b| b.files).sum();
    assert_eq!(total_in_histogram, 500);
}

#[test]
fn large_repo_top_offenders_capped() {
    let rows: Vec<FileRow> = (0..100)
        .map(|i| row(&format!("f{i}.rs"), "Rust", 100 + i * 10, i, 5))
        .collect();
    let r = derive_report(&export(rows), None);
    assert!(r.top.largest_lines.len() <= 10);
    assert!(r.top.largest_tokens.len() <= 10);
    assert!(r.top.largest_bytes.len() <= 10);
}

#[test]
fn large_repo_multi_language() {
    let langs = ["Rust", "Python", "TypeScript", "Go", "Java"];
    let rows: Vec<FileRow> = (0..250)
        .map(|i| row(&format!("f{i}.rs"), langs[i % langs.len()], 50 + i, 10, 5))
        .collect();
    let r = derive_report(&export(rows), None);
    assert_eq!(r.polyglot.lang_count, 5);
    assert!(r.polyglot.entropy > 0.0);
}

// ═══════════════════════════════════════════════════════════════
// 7. Property tests
// ═══════════════════════════════════════════════════════════════

fn arb_row() -> impl Strategy<Value = FileRow> {
    (
        "[a-z]{1,3}(/[a-z]{1,3}){0,2}\\.rs",
        "(Rust|Python|Go|TypeScript|TOML)",
        0..3000usize,
        0..500usize,
        0..300usize,
    )
        .prop_map(|(path, lang, code, comments, blanks)| {
            let lines = code + comments + blanks;
            FileRow {
                path,
                module: "root".to_string(),
                lang,
                kind: FileKind::Parent,
                code,
                comments,
                blanks,
                lines,
                bytes: lines * 40,
                tokens: lines * 5,
            }
        })
}

fn arb_rows() -> impl Strategy<Value = Vec<FileRow>> {
    prop::collection::vec(arb_row(), 1..=15)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(150))]

    #[test]
    fn prop_doc_density_ratio_in_01(rows in arb_rows()) {
        let r = derive_report(&export(rows), None);
        prop_assert!(r.doc_density.total.ratio >= 0.0);
        prop_assert!(r.doc_density.total.ratio <= 1.0);
    }

    #[test]
    fn prop_whitespace_ratio_non_negative(rows in arb_rows()) {
        let r = derive_report(&export(rows), None);
        prop_assert!(r.whitespace.total.ratio >= 0.0);
    }

    #[test]
    fn prop_cocomo_effort_positive_for_nonzero(rows in arb_rows()) {
        let total_code: usize = rows.iter().map(|r| r.code).sum();
        let r = derive_report(&export(rows), None);
        if total_code > 0 {
            let c = r.cocomo.unwrap();
            prop_assert!(c.effort_pm > 0.0);
            prop_assert!(c.duration_months > 0.0);
        } else {
            prop_assert!(r.cocomo.is_none());
        }
    }

    #[test]
    fn prop_gini_in_01(rows in arb_rows()) {
        let r = derive_report(&export(rows), None);
        prop_assert!(r.distribution.gini >= 0.0);
        prop_assert!(r.distribution.gini <= 1.0);
    }

    #[test]
    fn prop_histogram_pct_sums_approx_one(rows in arb_rows()) {
        let r = derive_report(&export(rows), None);
        let sum: f64 = r.histogram.iter().map(|b| b.pct).sum();
        prop_assert!((sum - 1.0).abs() < 0.02, "histogram pct sum = {}", sum);
    }

    #[test]
    fn prop_totals_match_row_sums(rows in arb_rows()) {
        let expected_code: usize = rows.iter().map(|r| r.code).sum();
        let expected_comments: usize = rows.iter().map(|r| r.comments).sum();
        let expected_blanks: usize = rows.iter().map(|r| r.blanks).sum();
        let r = derive_report(&export(rows.clone()), None);
        prop_assert_eq!(r.totals.code, expected_code);
        prop_assert_eq!(r.totals.comments, expected_comments);
        prop_assert_eq!(r.totals.blanks, expected_blanks);
        prop_assert_eq!(r.totals.files, rows.len());
    }

    #[test]
    fn prop_reading_time_proportional_to_code(rows in arb_rows()) {
        let total_code: usize = rows.iter().map(|r| r.code).sum();
        let r = derive_report(&export(rows), None);
        let expected = total_code as f64 / 20.0;
        prop_assert!((r.reading_time.minutes - expected).abs() < 0.1);
    }

    #[test]
    fn prop_polyglot_entropy_non_negative(rows in arb_rows()) {
        let r = derive_report(&export(rows), None);
        prop_assert!(r.polyglot.entropy >= 0.0);
    }

    #[test]
    fn prop_distribution_mean_in_range(rows in arb_rows()) {
        let r = derive_report(&export(rows), None);
        let d = &r.distribution;
        prop_assert!(d.mean >= d.min as f64);
        prop_assert!(d.mean <= d.max as f64);
    }
}

// ═══════════════════════════════════════════════════════════════
// 8. Determinism: same scan produces same metrics
// ═══════════════════════════════════════════════════════════════

#[test]
fn determinism_full_report() {
    let data = export(vec![
        row("src/a.rs", "Rust", 200, 50, 20),
        row("src/b.py", "Python", 150, 30, 10),
        row("lib/c.ts", "TypeScript", 80, 40, 15),
    ]);
    let r1 = derive_report(&data, Some(50_000));
    let r2 = derive_report(&data, Some(50_000));

    assert_eq!(r1.totals.code, r2.totals.code);
    assert_eq!(r1.totals.comments, r2.totals.comments);
    assert_eq!(r1.totals.blanks, r2.totals.blanks);
    assert_eq!(r1.totals.lines, r2.totals.lines);
    assert_eq!(r1.totals.bytes, r2.totals.bytes);
    assert_eq!(r1.totals.tokens, r2.totals.tokens);
    assert_eq!(r1.doc_density.total.ratio, r2.doc_density.total.ratio);
    assert_eq!(r1.whitespace.total.ratio, r2.whitespace.total.ratio);
    assert_eq!(r1.distribution.gini, r2.distribution.gini);
    assert_eq!(r1.distribution.median, r2.distribution.median);
    assert_eq!(r1.polyglot.entropy, r2.polyglot.entropy);
    assert_eq!(r1.integrity.hash, r2.integrity.hash);

    let c1 = r1.cocomo.unwrap();
    let c2 = r2.cocomo.unwrap();
    assert_eq!(c1.effort_pm, c2.effort_pm);
    assert_eq!(c1.duration_months, c2.duration_months);
    assert_eq!(c1.staff, c2.staff);
}

#[test]
fn determinism_integrity_hash_stable() {
    let data = export(vec![
        row("a.rs", "Rust", 100, 10, 5),
        row("b.rs", "Rust", 200, 20, 10),
    ]);
    let hashes: Vec<String> = (0..5)
        .map(|_| derive_report(&data, None).integrity.hash)
        .collect();
    for h in &hashes {
        assert_eq!(h, &hashes[0]);
    }
}

// ═══════════════════════════════════════════════════════════════
// 9. Snapshot tests for JSON metric output
// ═══════════════════════════════════════════════════════════════

#[test]
fn snapshot_totals_json() {
    let r = derive_report(&export(vec![row("src/main.rs", "Rust", 100, 20, 10)]), None);
    let json = serde_json::to_value(&r.totals).unwrap();
    assert_eq!(json["files"], 1);
    assert_eq!(json["code"], 100);
    assert_eq!(json["comments"], 20);
    assert_eq!(json["blanks"], 10);
    assert_eq!(json["lines"], 130);
}

#[test]
fn snapshot_cocomo_json() {
    let r = derive_report(&export(vec![row("a.rs", "Rust", 10000, 0, 0)]), None);
    let c = r.cocomo.unwrap();
    let json = serde_json::to_value(&c).unwrap();
    assert_eq!(json["mode"], "organic");
    assert_eq!(json["kloc"], 10.0);
    assert!(json["effort_pm"].as_f64().unwrap() > 0.0);
    assert!(json["duration_months"].as_f64().unwrap() > 0.0);
}

#[test]
fn snapshot_distribution_json() {
    let r = derive_report(
        &export(vec![
            row("a.rs", "Rust", 10, 0, 0),
            row("b.rs", "Rust", 100, 0, 0),
        ]),
        None,
    );
    let json = serde_json::to_value(&r.distribution).unwrap();
    assert_eq!(json["count"], 2);
    assert_eq!(json["min"], 10);
    assert_eq!(json["max"], 100);
}

#[test]
fn snapshot_context_window_json() {
    let r = derive_report(&export(vec![row("a.rs", "Rust", 100, 0, 0)]), Some(1000));
    let cw = r.context_window.unwrap();
    let json = serde_json::to_value(&cw).unwrap();
    assert_eq!(json["window_tokens"], 1000);
    assert_eq!(json["total_tokens"], 500);
    assert_eq!(json["fits"], true);
}

// ═══════════════════════════════════════════════════════════════
// 10. Additional edge cases and metrics
// ═══════════════════════════════════════════════════════════════

#[test]
fn verbosity_rate_varies_by_bytes() {
    let r = derive_report(
        &export(vec![
            row_with_bytes("a.rs", "Rust", 100, 0, 0, 8000),
            row_with_bytes("b.rs", "Rust", 100, 0, 0, 2000),
        ]),
        None,
    );
    // total bytes = 10000, total lines = 200
    assert_eq!(r.verbosity.total.numerator, 10000);
    assert_eq!(r.verbosity.total.denominator, 200);
    assert!((r.verbosity.total.rate - 50.0).abs() < 0.01);
}

#[test]
fn test_density_mixed_test_and_prod() {
    let r = derive_report(
        &export(vec![
            row("src/lib.rs", "Rust", 200, 0, 0),
            row("tests/test_lib.rs", "Rust", 100, 0, 0),
        ]),
        None,
    );
    assert_eq!(r.test_density.test_files, 1);
    assert_eq!(r.test_density.prod_files, 1);
    assert_eq!(r.test_density.test_lines, 100);
    assert_eq!(r.test_density.prod_lines, 200);
    // ratio = 100 / 300
    let expected = 100.0 / 300.0;
    assert!((r.test_density.ratio - expected).abs() < 0.001);
}

#[test]
fn polyglot_entropy_max_with_equal_split() {
    // Equal split among N languages should give maximum entropy log2(N)
    let r = derive_report(
        &export(vec![
            row("a.rs", "Rust", 100, 0, 0),
            row("b.py", "Python", 100, 0, 0),
            row("c.go", "Go", 100, 0, 0),
            row("d.ts", "TypeScript", 100, 0, 0),
        ]),
        None,
    );
    let expected_max = 4.0_f64.log2(); // 2.0
    assert!((r.polyglot.entropy - expected_max).abs() < 0.001);
}

#[test]
fn polyglot_dominant_is_largest_code() {
    let r = derive_report(
        &export(vec![
            row("a.rs", "Rust", 500, 0, 0),
            row("b.py", "Python", 100, 0, 0),
        ]),
        None,
    );
    assert_eq!(r.polyglot.dominant_lang, "Rust");
    assert_eq!(r.polyglot.dominant_lines, 500);
}

#[test]
fn lang_purity_single_lang_module() {
    let r = derive_report(
        &export(vec![
            row("src/a.rs", "Rust", 100, 0, 0),
            row("src/b.rs", "Rust", 200, 0, 0),
        ]),
        None,
    );
    // Both files in "src" module, all Rust
    let src_row = r.lang_purity.rows.iter().find(|p| p.module == "src");
    assert!(src_row.is_some());
    let src = src_row.unwrap();
    assert_eq!(src.lang_count, 1);
    assert_eq!(src.dominant_pct, 1.0);
}

#[test]
fn nesting_report_multiple_depths() {
    let r = derive_report(
        &export(vec![
            row("a.rs", "Rust", 100, 0, 0),          // depth 1
            row("src/b.rs", "Rust", 100, 0, 0),      // depth 2
            row("src/deep/c.rs", "Rust", 100, 0, 0), // depth 3
        ]),
        None,
    );
    assert!(r.nesting.max >= 3);
}

#[test]
fn context_window_exact_boundary() {
    // tokens exactly equals window
    let r = derive_report(
        &export(vec![row("a.rs", "Rust", 20, 0, 0)]), // 20 lines * 5 = 100 tokens
        Some(100),
    );
    let cw = r.context_window.unwrap();
    assert!(cw.fits); // total_tokens <= window_tokens
    assert_eq!(cw.total_tokens, 100);
    assert!((cw.pct - 1.0).abs() < 0.001);
}

#[test]
fn histogram_huge_file_classification() {
    let r = derive_report(&export(vec![row("big.rs", "Rust", 2000, 0, 0)]), None);
    // 2000 lines > 1000 => "Huge"
    assert_eq!(r.histogram[4].files, 1);
    assert_eq!(r.histogram[4].label, "Huge");
}

#[test]
fn boilerplate_report_with_infra_lang() {
    let r = derive_report(
        &export(vec![
            row("src/main.rs", "Rust", 200, 0, 0),
            row("Cargo.toml", "TOML", 50, 0, 0),
        ]),
        None,
    );
    // TOML is typically an infra lang
    assert!(r.boilerplate.infra_lines > 0 || r.boilerplate.logic_lines > 0);
    assert!(r.boilerplate.ratio >= 0.0 && r.boilerplate.ratio <= 1.0);
}
