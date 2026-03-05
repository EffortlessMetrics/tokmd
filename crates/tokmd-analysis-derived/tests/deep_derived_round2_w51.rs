//! Deep round-2 tests for `tokmd-analysis-derived` (w51).
//!
//! Focuses on COCOMO estimation across project sizes, extreme density ratios,
//! distribution metrics with uniform vs skewed distributions, Gini coefficient
//! verification, and polyglot behavior with 1 vs 100 languages.

use tokmd_analysis_derived::derive_report;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn row(
    path: &str,
    module: &str,
    lang: &str,
    code: usize,
    comments: usize,
    blanks: usize,
    bytes: usize,
    tokens: usize,
) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments,
        blanks,
        lines: code + comments + blanks,
        bytes,
        tokens,
    }
}

fn simple_row(path: &str, lang: &str, code: usize) -> FileRow {
    row(path, "src", lang, code, 0, 0, code * 40, code * 8)
}

fn export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

// ═══════════════════════════════════════════════════════════════════
// § COCOMO with various project sizes
// ═══════════════════════════════════════════════════════════════════

mod cocomo_sizes {
    use super::*;

    #[test]
    fn cocomo_small_project_100_lines() {
        let r = derive_report(&export(vec![simple_row("a.rs", "Rust", 100)]), None);
        let c = r.cocomo.as_ref().expect("cocomo present for code > 0");
        assert!((c.kloc - 0.1).abs() < 0.001);
        // effort = 2.4 * 0.1^1.05 ≈ 0.214
        assert!(c.effort_pm > 0.0 && c.effort_pm < 1.0);
        assert!(c.duration_months > 0.0);
        assert!(c.staff > 0.0);
    }

    #[test]
    fn cocomo_medium_project_5000_lines() {
        let r = derive_report(&export(vec![simple_row("a.rs", "Rust", 5_000)]), None);
        let c = r.cocomo.as_ref().unwrap();
        assert!((c.kloc - 5.0).abs() < 0.001);
        let expected = 2.4 * 5.0_f64.powf(1.05);
        assert!(
            (c.effort_pm - expected).abs() < 0.5,
            "5 KLOC effort ≈ {expected:.2}, got {}",
            c.effort_pm
        );
    }

    #[test]
    fn cocomo_large_project_50000_lines() {
        let r = derive_report(&export(vec![simple_row("a.rs", "Rust", 50_000)]), None);
        let c = r.cocomo.as_ref().unwrap();
        assert!((c.kloc - 50.0).abs() < 0.01);
        let expected = 2.4 * 50.0_f64.powf(1.05);
        assert!(
            (c.effort_pm - expected).abs() < 1.0,
            "50 KLOC effort ≈ {expected:.2}, got {}",
            c.effort_pm
        );
    }

    #[test]
    fn cocomo_huge_project_500000_lines() {
        let r = derive_report(&export(vec![simple_row("a.rs", "Rust", 500_000)]), None);
        let c = r.cocomo.as_ref().unwrap();
        assert!((c.kloc - 500.0).abs() < 0.1);
        let expected = 2.4 * 500.0_f64.powf(1.05);
        assert!(
            (c.effort_pm - expected).abs() < 5.0,
            "500 KLOC effort ≈ {expected:.2}, got {}",
            c.effort_pm
        );
        // Huge projects require many person-months
        assert!(c.effort_pm > 100.0);
        assert!(c.duration_months > 10.0);
    }

    #[test]
    fn cocomo_effort_increases_superlinearly() {
        let r1 = derive_report(&export(vec![simple_row("a.rs", "Rust", 1_000)]), None);
        let r10 = derive_report(&export(vec![simple_row("a.rs", "Rust", 10_000)]), None);
        let r100 = derive_report(&export(vec![simple_row("a.rs", "Rust", 100_000)]), None);

        let e1 = r1.cocomo.as_ref().unwrap().effort_pm;
        let e10 = r10.cocomo.as_ref().unwrap().effort_pm;
        let e100 = r100.cocomo.as_ref().unwrap().effort_pm;

        // 10x code should yield > 10x effort (superlinear, exponent 1.05)
        assert!(e10 / e1 > 10.0, "10x code → >10x effort");
        assert!(e100 / e10 > 10.0, "10x code → >10x effort");
    }

    #[test]
    fn cocomo_duration_constants_match() {
        let r = derive_report(&export(vec![simple_row("a.rs", "Rust", 10_000)]), None);
        let c = r.cocomo.as_ref().unwrap();
        assert_eq!(c.a, 2.4);
        assert_eq!(c.b, 1.05);
        assert_eq!(c.c, 2.5);
        assert_eq!(c.d, 0.38);
        assert_eq!(c.mode, "organic");
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Extreme density ratios
// ═══════════════════════════════════════════════════════════════════

mod density_extremes {
    use super::*;

    #[test]
    fn density_99_percent_comments() {
        // 1 code line, 99 comment lines → density ≈ 0.99
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 1, 99, 0, 4000, 80)]),
            None,
        );
        assert!(
            r.doc_density.total.ratio > 0.98,
            "99% comments → density > 0.98, got {}",
            r.doc_density.total.ratio
        );
        assert!(r.doc_density.total.ratio <= 1.0);
    }

    #[test]
    fn density_zero_percent_comments() {
        // 100 code lines, 0 comment lines → density = 0
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800)]),
            None,
        );
        assert_eq!(r.doc_density.total.ratio, 0.0);
    }

    #[test]
    fn density_all_comments_no_code() {
        // 0 code lines, 100 comment lines → density = 1.0
        let r = derive_report(
            &export(vec![row("a.rs", "src", "Rust", 0, 100, 0, 4000, 0)]),
            None,
        );
        assert_eq!(r.doc_density.total.ratio, 1.0);
    }

    #[test]
    fn density_by_lang_sorted_descending() {
        let r = derive_report(
            &export(vec![
                row("a.rs", "src", "Rust", 90, 10, 0, 4000, 800),
                row("b.py", "src", "Python", 50, 50, 0, 4000, 800),
                row("c.js", "src", "JavaScript", 20, 80, 0, 4000, 800),
            ]),
            None,
        );
        // by_lang rows should be sorted descending by ratio
        for w in r.doc_density.by_lang.windows(2) {
            assert!(
                w[0].ratio >= w[1].ratio,
                "by_lang not sorted descending: {} < {}",
                w[0].ratio,
                w[1].ratio
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Distribution with uniform vs skewed
// ═══════════════════════════════════════════════════════════════════

mod distribution_shape {
    use super::*;

    #[test]
    fn uniform_distribution_stats() {
        let rows: Vec<FileRow> = (0..10)
            .map(|i| simple_row(&format!("f{i}.rs"), "Rust", 100))
            .collect();
        let r = derive_report(&export(rows), None);
        let d = &r.distribution;
        assert_eq!(d.count, 10);
        assert_eq!(d.min, 100);
        assert_eq!(d.max, 100);
        assert!((d.mean - 100.0).abs() < 0.01);
        assert!((d.median - 100.0).abs() < 0.01);
        assert!(d.gini < 0.01, "uniform → gini ≈ 0, got {}", d.gini);
    }

    #[test]
    fn heavily_skewed_distribution() {
        let mut rows = vec![simple_row("giant.rs", "Rust", 100_000)];
        for i in 0..99 {
            rows.push(simple_row(&format!("tiny{i}.rs"), "Rust", 1));
        }
        let r = derive_report(&export(rows), None);
        let d = &r.distribution;
        assert_eq!(d.count, 100);
        assert_eq!(d.min, 1);
        assert_eq!(d.max, 100_000);
        assert!(d.gini > 0.8, "heavily skewed → gini > 0.8, got {}", d.gini);
        // p90 should be low since 90% of files are tiny
        assert!(d.p90 < 100.0);
        // p99 should be the giant file
        assert!(d.p99 >= 100_000.0);
    }

    #[test]
    fn gini_zero_for_perfectly_equal() {
        let rows: Vec<FileRow> = (0..20)
            .map(|i| simple_row(&format!("f{i}.rs"), "Rust", 50))
            .collect();
        let r = derive_report(&export(rows), None);
        assert_eq!(r.distribution.gini, 0.0);
    }

    #[test]
    fn gini_approaches_1_for_maximum_inequality() {
        // One file with all lines, rest with 1 line each
        let mut rows = vec![simple_row("big.rs", "Rust", 1_000_000)];
        for i in 0..999 {
            rows.push(simple_row(&format!("s{i}.rs"), "Rust", 1));
        }
        let r = derive_report(&export(rows), None);
        assert!(
            r.distribution.gini > 0.95,
            "extreme inequality → gini > 0.95, got {}",
            r.distribution.gini
        );
    }

    #[test]
    fn mean_between_min_and_max() {
        let rows = vec![
            simple_row("a.rs", "Rust", 10),
            simple_row("b.rs", "Rust", 100),
            simple_row("c.rs", "Rust", 1000),
        ];
        let r = derive_report(&export(rows), None);
        let d = &r.distribution;
        assert!(d.mean >= d.min as f64);
        assert!(d.mean <= d.max as f64);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Polyglot: 1 language vs 100 languages
// ═══════════════════════════════════════════════════════════════════

mod polyglot_extremes {
    use super::*;

    #[test]
    fn single_language_entropy_zero() {
        let rows: Vec<FileRow> = (0..50)
            .map(|i| simple_row(&format!("f{i}.rs"), "Rust", 100))
            .collect();
        let r = derive_report(&export(rows), None);
        assert_eq!(r.polyglot.lang_count, 1);
        assert_eq!(r.polyglot.entropy, 0.0);
        assert_eq!(r.polyglot.dominant_lang, "Rust");
        assert!((r.polyglot.dominant_pct - 1.0).abs() < 0.001);
    }

    #[test]
    fn many_languages_high_entropy() {
        let langs = [
            "Rust",
            "Python",
            "JavaScript",
            "TypeScript",
            "Go",
            "Java",
            "C",
            "C++",
            "Ruby",
            "PHP",
            "Swift",
            "Kotlin",
            "Scala",
            "Haskell",
            "OCaml",
            "Lua",
            "Perl",
            "R",
            "Julia",
            "Dart",
        ];
        let rows: Vec<FileRow> = langs
            .iter()
            .enumerate()
            .map(|(i, &lang)| simple_row(&format!("f{i}.x"), lang, 100))
            .collect();
        let r = derive_report(&export(rows), None);
        assert_eq!(r.polyglot.lang_count, 20);
        // Shannon entropy for 20 uniform languages ≈ log2(20) ≈ 4.32
        assert!(
            r.polyglot.entropy > 4.0,
            "20 uniform langs → entropy > 4.0, got {}",
            r.polyglot.entropy
        );
    }

    #[test]
    fn dominant_language_identified_correctly() {
        let r = derive_report(
            &export(vec![
                simple_row("a.rs", "Rust", 900),
                simple_row("b.py", "Python", 50),
                simple_row("c.js", "JavaScript", 50),
            ]),
            None,
        );
        assert_eq!(r.polyglot.dominant_lang, "Rust");
        assert!(r.polyglot.dominant_pct > 0.85);
        assert_eq!(r.polyglot.dominant_lines, 900);
    }

    #[test]
    fn two_equal_languages_entropy_one() {
        let r = derive_report(
            &export(vec![
                simple_row("a.rs", "Rust", 100),
                simple_row("b.py", "Python", 100),
            ]),
            None,
        );
        assert_eq!(r.polyglot.lang_count, 2);
        // Shannon entropy for 2 uniform = log2(2) = 1.0
        assert!(
            (r.polyglot.entropy - 1.0).abs() < 0.01,
            "2 equal langs → entropy ≈ 1.0, got {}",
            r.polyglot.entropy
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Histogram bucket assignment
// ═══════════════════════════════════════════════════════════════════

mod histogram_buckets {
    use super::*;

    #[test]
    fn histogram_sums_to_file_count() {
        let rows: Vec<FileRow> = (0..25)
            .map(|i| simple_row(&format!("f{i}.rs"), "Rust", (i + 1) * 50))
            .collect();
        let r = derive_report(&export(rows), None);
        let total: usize = r.histogram.iter().map(|b| b.files).sum();
        assert_eq!(total, 25);
    }

    #[test]
    fn histogram_pct_sums_to_approximately_one() {
        let rows: Vec<FileRow> = (0..10)
            .map(|i| simple_row(&format!("f{i}.rs"), "Rust", (i + 1) * 100))
            .collect();
        let r = derive_report(&export(rows), None);
        let pct_sum: f64 = r.histogram.iter().map(|b| b.pct).sum();
        assert!(
            (pct_sum - 1.0).abs() < 0.01,
            "histogram pct sum ≈ 1.0, got {pct_sum}"
        );
    }
}
