//! W75 deep tests for tokmd-analysis-derived.
//!
//! Covers COCOMO model calculations, density metrics, distribution
//! percentages, language proportions, and edge cases.

use tokmd_analysis_derived::derive_report;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────────

fn empty_export() -> ExportData {
    ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

fn single_file(code: usize, comments: usize, blanks: usize) -> ExportData {
    let lines = code + comments + blanks;
    ExportData {
        rows: vec![FileRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code,
            comments,
            blanks,
            lines,
            bytes: lines * 30,
            tokens: code * 8,
        }],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

fn multi_lang_export() -> ExportData {
    ExportData {
        rows: vec![
            FileRow {
                path: "src/main.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 400,
                comments: 80,
                blanks: 40,
                lines: 520,
                bytes: 13_000,
                tokens: 3_200,
            },
            FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 200,
                comments: 40,
                blanks: 20,
                lines: 260,
                bytes: 6_500,
                tokens: 1_600,
            },
            FileRow {
                path: "scripts/build.py".to_string(),
                module: "scripts".to_string(),
                lang: "Python".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 10,
                blanks: 10,
                lines: 120,
                bytes: 3_000,
                tokens: 800,
            },
            FileRow {
                path: "web/index.js".to_string(),
                module: "web".to_string(),
                lang: "JavaScript".to_string(),
                kind: FileKind::Parent,
                code: 50,
                comments: 5,
                blanks: 5,
                lines: 60,
                bytes: 1_500,
                tokens: 400,
            },
        ],
        module_roots: vec!["src".to_string(), "scripts".to_string(), "web".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 1. COCOMO model calculations with known inputs
// ═══════════════════════════════════════════════════════════════════

#[test]
fn cocomo_1kloc_known_values() {
    let report = derive_report(&single_file(1000, 0, 0), None);
    let c = report.cocomo.expect("COCOMO present for 1 KLOC");
    assert_eq!(c.mode, "organic");
    assert_eq!(c.kloc, 1.0);
    // effort = 2.4 * 1.0^1.05 = 2.4
    assert_eq!(c.effort_pm, 2.4);
    // duration = 2.5 * 2.4^0.38
    assert!(c.duration_months > 0.0);
    assert!(c.staff > 0.0);
}

#[test]
fn cocomo_10kloc_known_values() {
    let report = derive_report(&single_file(10_000, 0, 0), None);
    let c = report.cocomo.unwrap();
    assert_eq!(c.kloc, 10.0);
    // effort = 2.4 * 10^1.05 ≈ 26.92
    assert!((c.effort_pm - 26.92).abs() < 0.1, "effort={}", c.effort_pm);
}

#[test]
fn cocomo_none_for_zero_code() {
    let report = derive_report(&empty_export(), None);
    assert!(report.cocomo.is_none());
}

#[test]
fn cocomo_scales_monotonically() {
    let sizes = [100, 500, 1000, 5000, 10000];
    let efforts: Vec<f64> = sizes
        .iter()
        .map(|&s| {
            derive_report(&single_file(s, 0, 0), None)
                .cocomo
                .unwrap()
                .effort_pm
        })
        .collect();
    for w in efforts.windows(2) {
        assert!(w[1] > w[0], "effort must increase: {} vs {}", w[0], w[1]);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 2. Density metric computation (doc_density)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn density_50_50_code_comments() {
    let report = derive_report(&single_file(100, 100, 0), None);
    // 100 / (100 + 100) = 0.5
    assert_eq!(report.doc_density.total.ratio, 0.5);
}

#[test]
fn density_pure_code_is_zero() {
    let report = derive_report(&single_file(500, 0, 20), None);
    assert_eq!(report.doc_density.total.ratio, 0.0);
}

#[test]
fn density_pure_comments_is_one() {
    let report = derive_report(&single_file(0, 200, 0), None);
    assert_eq!(report.doc_density.total.ratio, 1.0);
}

#[test]
fn density_by_lang_present_for_multi_lang() {
    let report = derive_report(&multi_lang_export(), None);
    assert!(
        report.doc_density.by_lang.len() >= 2,
        "multi-lang should have per-lang density"
    );
}

// ═══════════════════════════════════════════════════════════════════
// § 3. Distribution percentages
// ═══════════════════════════════════════════════════════════════════

#[test]
fn distribution_single_file_all_equal() {
    let report = derive_report(&single_file(100, 20, 10), None);
    let d = &report.distribution;
    assert_eq!(d.count, 1);
    assert_eq!(d.min, d.max);
    assert_eq!(d.mean, d.median);
}

#[test]
fn distribution_multi_file_sorted_correctly() {
    let report = derive_report(&multi_lang_export(), None);
    let d = &report.distribution;
    assert_eq!(d.count, 4);
    assert_eq!(d.min, 60);
    assert_eq!(d.max, 520);
    assert!(d.mean > 0.0);
    assert!(d.median > 0.0);
    assert!(d.p90 >= d.median);
    assert!(d.p99 >= d.p90);
}

#[test]
fn distribution_gini_zero_for_equal_files() {
    let export = ExportData {
        rows: vec![
            FileRow {
                path: "a.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 0,
                blanks: 0,
                lines: 100,
                bytes: 2500,
                tokens: 800,
            },
            FileRow {
                path: "b.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 0,
                blanks: 0,
                lines: 100,
                bytes: 2500,
                tokens: 800,
            },
        ],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    };
    let report = derive_report(&export, None);
    assert_eq!(report.distribution.gini, 0.0);
}

// ═══════════════════════════════════════════════════════════════════
// § 4. Language proportion calculations (polyglot report)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn polyglot_single_lang_dominant_100pct() {
    let report = derive_report(&single_file(100, 20, 10), None);
    assert_eq!(report.polyglot.lang_count, 1);
    assert_eq!(report.polyglot.dominant_lang, "Rust");
    assert_eq!(report.polyglot.dominant_pct, 1.0);
    assert_eq!(report.polyglot.entropy, 0.0);
}

#[test]
fn polyglot_multi_lang_has_entropy() {
    let report = derive_report(&multi_lang_export(), None);
    assert_eq!(report.polyglot.lang_count, 3);
    assert!(
        report.polyglot.entropy > 0.0,
        "multi-lang entropy should be positive"
    );
    assert_eq!(report.polyglot.dominant_lang, "Rust");
    // Rust has 600 code lines out of 750 total
    assert!(report.polyglot.dominant_pct > 0.5);
}

#[test]
fn polyglot_dominant_lines_correct() {
    let report = derive_report(&multi_lang_export(), None);
    // Rust code = 400 + 200 = 600
    assert_eq!(report.polyglot.dominant_lines, 600);
}

// ═══════════════════════════════════════════════════════════════════
// § 5. Edge cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn edge_empty_project_all_zero() {
    let report = derive_report(&empty_export(), None);
    assert_eq!(report.totals.files, 0);
    assert_eq!(report.totals.code, 0);
    assert_eq!(report.totals.lines, 0);
    assert_eq!(report.distribution.count, 0);
    assert_eq!(report.polyglot.lang_count, 0);
    assert_eq!(report.reading_time.minutes, 0.0);
}

#[test]
fn edge_child_rows_excluded() {
    let export = ExportData {
        rows: vec![
            FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 10,
                blanks: 5,
                lines: 115,
                bytes: 2875,
                tokens: 800,
            },
            FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Markdown".to_string(),
                kind: FileKind::Child,
                code: 50,
                comments: 0,
                blanks: 0,
                lines: 50,
                bytes: 1250,
                tokens: 400,
            },
        ],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let report = derive_report(&export, None);
    // Only Parent rows counted
    assert_eq!(report.totals.files, 1);
    assert_eq!(report.totals.code, 100);
}

#[test]
fn edge_reading_time_matches_formula() {
    let report = derive_report(&single_file(400, 0, 0), None);
    // 400 / 20 = 20.0 minutes
    assert_eq!(report.reading_time.minutes, 20.0);
    assert_eq!(report.reading_time.lines_per_minute, 20);
    assert_eq!(report.reading_time.basis_lines, 400);
}
