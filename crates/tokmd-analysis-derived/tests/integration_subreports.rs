//! Integration tests for derived sub-reports: boilerplate, verbosity,
//! lang purity, top offenders, and test density edge cases.

use tokmd_analysis_derived::derive_report;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn make_row(
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

fn export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

// ── Boilerplate ─────────────────────────────────────────────────

mod boilerplate {
    use super::*;

    #[test]
    fn given_only_logic_langs_then_boilerplate_ratio_is_zero() {
        let rows = vec![
            make_row("src/lib.rs", "src", "Rust", 200, 10, 5, 8000, 1600),
            make_row("src/app.py", "src", "Python", 100, 5, 3, 4000, 800),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.boilerplate.ratio, 0.0);
        assert_eq!(report.boilerplate.infra_lines, 0);
        assert!(report.boilerplate.infra_langs.is_empty());
    }

    #[test]
    fn given_infra_files_then_boilerplate_ratio_is_positive() {
        let rows = vec![
            make_row("src/lib.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("Cargo.toml", ".", "TOML", 50, 5, 5, 2000, 400),
            make_row("config.json", ".", "JSON", 30, 0, 0, 1200, 240),
        ];
        let report = derive_report(&export(rows), None);
        assert!(report.boilerplate.ratio > 0.0);
        assert!(report.boilerplate.infra_lines > 0);
        assert!(report.boilerplate.logic_lines > 0);
        assert!(!report.boilerplate.infra_langs.is_empty());
    }

    #[test]
    fn given_only_infra_files_then_ratio_is_one() {
        let rows = vec![
            make_row("Cargo.toml", ".", "TOML", 50, 0, 0, 2000, 400),
            make_row("package.json", ".", "JSON", 30, 0, 0, 1200, 240),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.boilerplate.ratio, 1.0);
        assert_eq!(report.boilerplate.logic_lines, 0);
    }

    #[test]
    fn given_empty_input_then_boilerplate_is_zeroed() {
        let report = derive_report(&export(vec![]), None);
        assert_eq!(report.boilerplate.ratio, 0.0);
        assert_eq!(report.boilerplate.infra_lines, 0);
        assert_eq!(report.boilerplate.logic_lines, 0);
    }
}

// ── Verbosity ───────────────────────────────────────────────────

mod verbosity {
    use super::*;

    #[test]
    fn given_files_then_verbosity_rate_is_bytes_per_line() {
        // 4000 bytes / 110 lines ≈ 36.36
        let rows = vec![make_row("src/lib.rs", "src", "Rust", 100, 5, 5, 4000, 800)];
        let report = derive_report(&export(rows), None);
        let expected_rate = 4000.0 / 110.0;
        assert!(
            (report.verbosity.total.rate - expected_rate).abs() < 0.1,
            "verbosity rate should be ~{expected_rate}, got {}",
            report.verbosity.total.rate
        );
    }

    #[test]
    fn given_multi_lang_files_then_verbosity_has_lang_breakdown() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 100, 0, 0, 8000, 1600),
            make_row("b.py", "src", "Python", 100, 0, 0, 3000, 600),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.verbosity.by_lang.len(), 2);
        // Rust has higher bytes-per-line → should be first (sorted desc by rate)
        assert_eq!(report.verbosity.by_lang[0].key, "Rust");
    }

    #[test]
    fn given_zero_lines_then_verbosity_rate_is_zero() {
        let report = derive_report(&export(vec![]), None);
        assert_eq!(report.verbosity.total.rate, 0.0);
    }
}

// ── Lang Purity ─────────────────────────────────────────────────

mod lang_purity {
    use super::*;

    #[test]
    fn given_single_lang_module_then_purity_is_one() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("src/b.rs", "src", "Rust", 200, 0, 0, 8000, 1600),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.lang_purity.rows.len(), 1);
        assert_eq!(report.lang_purity.rows[0].module, "src");
        assert_eq!(report.lang_purity.rows[0].lang_count, 1);
        assert_eq!(report.lang_purity.rows[0].dominant_pct, 1.0);
    }

    #[test]
    fn given_mixed_lang_module_then_purity_reflects_dominant() {
        let rows = vec![
            make_row("src/lib.rs", "src", "Rust", 300, 0, 0, 12000, 2400),
            make_row("src/helper.py", "src", "Python", 100, 0, 0, 4000, 800),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.lang_purity.rows.len(), 1);
        let row = &report.lang_purity.rows[0];
        assert_eq!(row.lang_count, 2);
        assert_eq!(row.dominant_lang, "Rust");
        // Rust: 300 lines, Python: 100 lines → 300/400 = 0.75
        assert_eq!(row.dominant_pct, 0.75);
    }

    #[test]
    fn given_multiple_modules_then_purity_rows_per_module() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("lib/b.py", "lib", "Python", 100, 0, 0, 4000, 800),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.lang_purity.rows.len(), 2);
        // Sorted by module name
        assert_eq!(report.lang_purity.rows[0].module, "lib");
        assert_eq!(report.lang_purity.rows[1].module, "src");
    }

    #[test]
    fn given_empty_input_then_purity_is_empty() {
        let report = derive_report(&export(vec![]), None);
        assert!(report.lang_purity.rows.is_empty());
    }
}

// ── Top Offenders ───────────────────────────────────────────────

mod top_offenders {
    use super::*;

    #[test]
    fn given_files_then_largest_by_lines_sorted_desc() {
        let rows = vec![
            make_row("small.rs", "src", "Rust", 10, 0, 0, 400, 80),
            make_row("big.rs", "src", "Rust", 500, 0, 0, 20000, 4000),
            make_row("medium.rs", "src", "Rust", 100, 0, 0, 4000, 800),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.top.largest_lines[0].path, "big.rs");
        assert_eq!(report.top.largest_lines[1].path, "medium.rs");
        assert_eq!(report.top.largest_lines[2].path, "small.rs");
    }

    #[test]
    fn given_files_then_largest_by_tokens_sorted_desc() {
        let rows = vec![
            make_row("low_tok.rs", "src", "Rust", 100, 0, 0, 4000, 100),
            make_row("high_tok.rs", "src", "Rust", 100, 0, 0, 4000, 5000),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.top.largest_tokens[0].path, "high_tok.rs");
        assert_eq!(report.top.largest_tokens[1].path, "low_tok.rs");
    }

    #[test]
    fn given_more_than_ten_files_then_top_bounded_at_ten() {
        let rows: Vec<FileRow> = (0..15)
            .map(|i| {
                make_row(
                    &format!("f{i:02}.rs"),
                    "src",
                    "Rust",
                    (i + 1) * 10,
                    0,
                    0,
                    (i + 1) * 400,
                    (i + 1) * 80,
                )
            })
            .collect();
        let report = derive_report(&export(rows), None);
        assert!(report.top.largest_lines.len() <= 10);
        assert!(report.top.largest_tokens.len() <= 10);
        assert!(report.top.largest_bytes.len() <= 10);
    }

    #[test]
    fn given_undocumented_large_files_then_least_documented_populated() {
        // Files >= 50 lines with zero comments show up in least_documented
        let rows = vec![
            make_row("nodoc.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("doc.rs", "src", "Rust", 50, 50, 0, 4000, 800),
        ];
        let report = derive_report(&export(rows), None);
        assert!(!report.top.least_documented.is_empty());
        // nodoc.rs has 0% documentation
        assert_eq!(report.top.least_documented[0].path, "nodoc.rs");
    }

    #[test]
    fn given_dense_files_then_most_dense_populated() {
        // Files >= 10 lines with high bytes-per-line
        let rows = vec![
            make_row("sparse.rs", "src", "Rust", 50, 0, 0, 500, 100),
            make_row("dense.rs", "src", "Rust", 50, 0, 0, 50000, 10000),
        ];
        let report = derive_report(&export(rows), None);
        assert!(!report.top.most_dense.is_empty());
        // dense.rs has 50000/50=1000 bytes per line
        assert_eq!(report.top.most_dense[0].path, "dense.rs");
    }

    #[test]
    fn given_empty_input_then_all_top_offenders_empty() {
        let report = derive_report(&export(vec![]), None);
        assert!(report.top.largest_lines.is_empty());
        assert!(report.top.largest_tokens.is_empty());
        assert!(report.top.largest_bytes.is_empty());
        assert!(report.top.least_documented.is_empty());
        assert!(report.top.most_dense.is_empty());
    }
}
