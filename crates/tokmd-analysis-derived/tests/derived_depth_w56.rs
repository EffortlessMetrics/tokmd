//! Depth tests for derived analytics (COCOMO, density, distribution) (W56).

use tokmd_analysis_derived::derive_report;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ───────────────────── helpers ─────────────────────

fn make_row(path: &str, lang: &str, code: usize, comments: usize, blanks: usize) -> FileRow {
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

fn make_export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

// ───────────────────── empty export ─────────────────────

#[test]
fn empty_export_yields_zero_totals() {
    let report = derive_report(&make_export(vec![]), None);
    assert_eq!(report.totals.files, 0);
    assert_eq!(report.totals.code, 0);
    assert_eq!(report.totals.comments, 0);
    assert_eq!(report.totals.blanks, 0);
    assert_eq!(report.totals.lines, 0);
}

#[test]
fn empty_export_cocomo_is_none() {
    let report = derive_report(&make_export(vec![]), None);
    assert!(report.cocomo.is_none());
}

#[test]
fn empty_export_distribution_zeros() {
    let report = derive_report(&make_export(vec![]), None);
    assert_eq!(report.distribution.count, 0);
    assert_eq!(report.distribution.min, 0);
    assert_eq!(report.distribution.max, 0);
    assert_eq!(report.distribution.mean, 0.0);
    assert_eq!(report.distribution.gini, 0.0);
}

// ───────────────────── single file ─────────────────────

#[test]
fn single_file_totals() {
    let export = make_export(vec![make_row("src/lib.rs", "Rust", 100, 20, 10)]);
    let report = derive_report(&export, None);
    assert_eq!(report.totals.files, 1);
    assert_eq!(report.totals.code, 100);
    assert_eq!(report.totals.comments, 20);
    assert_eq!(report.totals.blanks, 10);
}

#[test]
fn single_file_distribution() {
    let export = make_export(vec![make_row("src/lib.rs", "Rust", 100, 20, 10)]);
    let report = derive_report(&export, None);
    assert_eq!(report.distribution.count, 1);
    assert_eq!(report.distribution.min, 130); // lines = code+comments+blanks
    assert_eq!(report.distribution.max, 130);
    assert_eq!(report.distribution.median, 130.0);
}

// ───────────────────── COCOMO ─────────────────────

#[test]
fn cocomo_present_for_nonzero_code() {
    let export = make_export(vec![make_row("src/lib.rs", "Rust", 1000, 100, 50)]);
    let report = derive_report(&export, None);
    let cocomo = report.cocomo.unwrap();
    assert_eq!(cocomo.mode, "organic");
    assert_eq!(cocomo.kloc, 1.0);
    assert!(cocomo.effort_pm > 0.0);
    assert!(cocomo.duration_months > 0.0);
    assert!(cocomo.staff > 0.0);
}

#[test]
fn cocomo_kloc_scales_linearly() {
    let export_1k = make_export(vec![make_row("a.rs", "Rust", 1000, 0, 0)]);
    let export_10k = make_export(vec![make_row("a.rs", "Rust", 10000, 0, 0)]);
    let c1 = derive_report(&export_1k, None).cocomo.unwrap();
    let c10 = derive_report(&export_10k, None).cocomo.unwrap();
    assert_eq!(c1.kloc, 1.0);
    assert_eq!(c10.kloc, 10.0);
}

#[test]
fn cocomo_effort_increases_with_code() {
    let small = make_export(vec![make_row("a.rs", "Rust", 1000, 0, 0)]);
    let big = make_export(vec![make_row("a.rs", "Rust", 50000, 0, 0)]);
    let cs = derive_report(&small, None).cocomo.unwrap();
    let cb = derive_report(&big, None).cocomo.unwrap();
    assert!(cb.effort_pm > cs.effort_pm);
    assert!(cb.duration_months > cs.duration_months);
}

#[test]
fn cocomo_organic_constants() {
    let export = make_export(vec![make_row("a.rs", "Rust", 1000, 0, 0)]);
    let cocomo = derive_report(&export, None).cocomo.unwrap();
    assert_eq!(cocomo.a, 2.4);
    assert_eq!(cocomo.b, 1.05);
    assert_eq!(cocomo.c, 2.5);
    assert_eq!(cocomo.d, 0.38);
}

#[test]
fn cocomo_formula_verification() {
    let export = make_export(vec![make_row("a.rs", "Rust", 5000, 0, 0)]);
    let cocomo = derive_report(&export, None).cocomo.unwrap();
    let kloc = 5.0_f64;
    let expected_effort = 2.4 * kloc.powf(1.05);
    let expected_duration = 2.5 * expected_effort.powf(0.38);
    // Allow small rounding difference
    assert!((cocomo.effort_pm - expected_effort).abs() < 0.1);
    assert!((cocomo.duration_months - expected_duration).abs() < 0.1);
}

// ───────────────────── doc density ─────────────────────

#[test]
fn doc_density_zero_when_no_comments() {
    let export = make_export(vec![make_row("a.rs", "Rust", 100, 0, 10)]);
    let report = derive_report(&export, None);
    assert_eq!(report.doc_density.total.numerator, 0);
    assert_eq!(report.doc_density.total.ratio, 0.0);
}

#[test]
fn doc_density_computed_correctly() {
    let export = make_export(vec![make_row("a.rs", "Rust", 80, 20, 10)]);
    let report = derive_report(&export, None);
    // ratio = comments / (code + comments) = 20 / 100 = 0.2
    assert_eq!(report.doc_density.total.numerator, 20);
    assert_eq!(report.doc_density.total.denominator, 100);
    assert!((report.doc_density.total.ratio - 0.2).abs() < 0.001);
}

#[test]
fn doc_density_by_lang() {
    let export = make_export(vec![
        make_row("a.rs", "Rust", 80, 20, 5),
        make_row("b.py", "Python", 50, 50, 10),
    ]);
    let report = derive_report(&export, None);
    assert!(report.doc_density.by_lang.len() >= 2);
}

// ───────────────────── whitespace ratio ─────────────────────

#[test]
fn whitespace_ratio_computed() {
    let export = make_export(vec![make_row("a.rs", "Rust", 80, 10, 10)]);
    let report = derive_report(&export, None);
    // blanks / (code + comments) = 10 / 90
    assert_eq!(report.whitespace.total.numerator, 10);
    assert_eq!(report.whitespace.total.denominator, 90);
}

// ───────────────────── verbosity (bytes per line) ─────────────────────

#[test]
fn verbosity_computed() {
    let export = make_export(vec![make_row("a.rs", "Rust", 100, 0, 0)]);
    let report = derive_report(&export, None);
    // bytes = 100*40 = 4000, lines = 100
    assert_eq!(report.verbosity.total.numerator, 4000);
    assert_eq!(report.verbosity.total.denominator, 100);
    assert!((report.verbosity.total.rate - 40.0).abs() < 0.01);
}

// ───────────────────── context window ─────────────────────

#[test]
fn context_window_none_when_not_requested() {
    let export = make_export(vec![make_row("a.rs", "Rust", 100, 0, 0)]);
    let report = derive_report(&export, None);
    assert!(report.context_window.is_none());
}

#[test]
fn context_window_fits_when_small() {
    let export = make_export(vec![make_row("a.rs", "Rust", 100, 0, 0)]);
    let report = derive_report(&export, Some(100_000));
    let cw = report.context_window.unwrap();
    assert!(cw.fits);
    assert_eq!(cw.window_tokens, 100_000);
    assert_eq!(cw.total_tokens, 500); // 100 lines * 5 tokens
}

#[test]
fn context_window_does_not_fit() {
    let export = make_export(vec![make_row("a.rs", "Rust", 10000, 0, 0)]);
    let report = derive_report(&export, Some(100));
    let cw = report.context_window.unwrap();
    assert!(!cw.fits);
    assert!(cw.pct > 1.0);
}

#[test]
fn context_window_zero_window() {
    let export = make_export(vec![make_row("a.rs", "Rust", 100, 0, 0)]);
    let report = derive_report(&export, Some(0));
    let cw = report.context_window.unwrap();
    assert_eq!(cw.pct, 0.0);
}

// ───────────────────── distribution ─────────────────────

#[test]
fn distribution_multiple_files() {
    let export = make_export(vec![
        make_row("a.rs", "Rust", 10, 0, 0),
        make_row("b.rs", "Rust", 50, 0, 0),
        make_row("c.rs", "Rust", 100, 0, 0),
    ]);
    let report = derive_report(&export, None);
    assert_eq!(report.distribution.count, 3);
    assert_eq!(report.distribution.min, 10);
    assert_eq!(report.distribution.max, 100);
    assert_eq!(report.distribution.median, 50.0);
}

#[test]
fn distribution_gini_zero_for_equal_files() {
    let export = make_export(vec![
        make_row("a.rs", "Rust", 100, 0, 0),
        make_row("b.rs", "Rust", 100, 0, 0),
        make_row("c.rs", "Rust", 100, 0, 0),
    ]);
    let report = derive_report(&export, None);
    assert_eq!(report.distribution.gini, 0.0);
}

#[test]
fn distribution_gini_nonzero_for_unequal() {
    let export = make_export(vec![
        make_row("a.rs", "Rust", 1, 0, 0),
        make_row("b.rs", "Rust", 1000, 0, 0),
    ]);
    let report = derive_report(&export, None);
    assert!(report.distribution.gini > 0.0);
}

// ───────────────────── histogram ─────────────────────

#[test]
fn histogram_bucket_labels() {
    let export = make_export(vec![make_row("a.rs", "Rust", 10, 0, 0)]);
    let report = derive_report(&export, None);
    let labels: Vec<&str> = report.histogram.iter().map(|b| b.label.as_str()).collect();
    assert_eq!(labels, vec!["Tiny", "Small", "Medium", "Large", "Huge"]);
}

#[test]
fn histogram_tiny_file_classification() {
    let export = make_export(vec![make_row("a.rs", "Rust", 10, 0, 0)]);
    let report = derive_report(&export, None);
    assert_eq!(report.histogram[0].files, 1); // Tiny
}

#[test]
fn histogram_percentages_sum_to_one() {
    let export = make_export(vec![
        make_row("a.rs", "Rust", 10, 0, 0),
        make_row("b.rs", "Rust", 100, 0, 0),
        make_row("c.rs", "Rust", 300, 0, 0),
        make_row("d.rs", "Rust", 800, 0, 0),
        make_row("e.rs", "Rust", 2000, 0, 0),
    ]);
    let report = derive_report(&export, None);
    let sum: f64 = report.histogram.iter().map(|b| b.pct).sum();
    assert!((sum - 1.0).abs() < 0.01);
}

// ───────────────────── reading time ─────────────────────

#[test]
fn reading_time_computed() {
    let export = make_export(vec![make_row("a.rs", "Rust", 200, 0, 0)]);
    let report = derive_report(&export, None);
    // 200 code lines / 20 lines per minute = 10.0 minutes
    assert_eq!(report.reading_time.minutes, 10.0);
    assert_eq!(report.reading_time.lines_per_minute, 20);
    assert_eq!(report.reading_time.basis_lines, 200);
}

#[test]
fn reading_time_zero_for_empty() {
    let report = derive_report(&make_export(vec![]), None);
    assert_eq!(report.reading_time.minutes, 0.0);
}

// ───────────────────── polyglot ─────────────────────

#[test]
fn polyglot_single_language() {
    let export = make_export(vec![make_row("a.rs", "Rust", 100, 0, 0)]);
    let report = derive_report(&export, None);
    assert_eq!(report.polyglot.lang_count, 1);
    assert_eq!(report.polyglot.dominant_lang, "Rust");
    assert_eq!(report.polyglot.entropy, 0.0);
}

#[test]
fn polyglot_multiple_languages() {
    let export = make_export(vec![
        make_row("a.rs", "Rust", 100, 0, 0),
        make_row("b.py", "Python", 100, 0, 0),
    ]);
    let report = derive_report(&export, None);
    assert_eq!(report.polyglot.lang_count, 2);
    assert!(report.polyglot.entropy > 0.0);
}

// ───────────────────── determinism ─────────────────────

#[test]
fn derive_report_deterministic() {
    let export = make_export(vec![
        make_row("src/a.rs", "Rust", 100, 20, 10),
        make_row("src/b.py", "Python", 50, 30, 5),
    ]);
    let r1 = derive_report(&export, Some(10000));
    let r2 = derive_report(&export, Some(10000));
    assert_eq!(r1.totals.code, r2.totals.code);
    assert_eq!(r1.totals.comments, r2.totals.comments);
    assert_eq!(r1.distribution.gini, r2.distribution.gini);
    assert_eq!(r1.polyglot.entropy, r2.polyglot.entropy);
    let c1 = r1.cocomo.unwrap();
    let c2 = r2.cocomo.unwrap();
    assert_eq!(c1.effort_pm, c2.effort_pm);
}

// ───────────────────── integrity ─────────────────────

#[test]
fn integrity_hash_present() {
    let export = make_export(vec![make_row("a.rs", "Rust", 100, 0, 0)]);
    let report = derive_report(&export, None);
    assert_eq!(report.integrity.algo, "blake3");
    assert!(!report.integrity.hash.is_empty());
    assert_eq!(report.integrity.entries, 1);
}

#[test]
fn integrity_hash_deterministic() {
    let export = make_export(vec![make_row("a.rs", "Rust", 100, 0, 0)]);
    let r1 = derive_report(&export, None);
    let r2 = derive_report(&export, None);
    assert_eq!(r1.integrity.hash, r2.integrity.hash);
}

// ───────────────────── thousands of files ─────────────────────

#[test]
fn large_file_count_does_not_panic() {
    let rows: Vec<FileRow> = (0..1000)
        .map(|i| make_row(&format!("src/file_{i}.rs"), "Rust", 10 + i, i % 10, 2))
        .collect();
    let export = make_export(rows);
    let report = derive_report(&export, Some(100_000));
    assert_eq!(report.totals.files, 1000);
    assert!(report.cocomo.is_some());
    assert_eq!(report.distribution.count, 1000);
}
