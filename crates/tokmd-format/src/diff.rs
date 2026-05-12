use crate::now_ms;
// -----------------
// Diff output
// -----------------

mod compute;
mod render;

pub use compute::{compute_diff_rows, compute_diff_totals};
pub use render::{DiffColorMode, DiffRenderOptions, render_diff_md, render_diff_md_with_options};
use tokmd_types::{DiffReceipt, DiffRow, DiffTotals, ToolInfo};

/// Create a DiffReceipt for JSON output.
pub fn create_diff_receipt(
    from_source: &str,
    to_source: &str,
    rows: Vec<DiffRow>,
    totals: DiffTotals,
) -> DiffReceipt {
    DiffReceipt {
        schema_version: tokmd_types::SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "diff".to_string(),
        from_source: from_source.to_string(),
        to_source: to_source.to_string(),
        diff_rows: rows,
        totals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_settings::ChildrenMode;
    use tokmd_types::{LangReport, LangRow, Totals};
    #[test]
    fn test_render_diff_md_smoke() {
        // Kills mutants: render_diff_md -> String::new() / "xyzzy".into()
        let from = LangReport {
            rows: vec![LangRow {
                lang: "Rust".to_string(),
                code: 10,
                lines: 10,
                files: 1,
                bytes: 100,
                tokens: 20,
                avg_lines: 10,
            }],
            total: Totals {
                code: 10,
                lines: 10,
                files: 1,
                bytes: 100,
                tokens: 20,
                avg_lines: 10,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let to = LangReport {
            rows: vec![LangRow {
                lang: "Rust".to_string(),
                code: 12,
                lines: 12,
                files: 1,
                bytes: 120,
                tokens: 24,
                avg_lines: 12,
            }],
            total: Totals {
                code: 12,
                lines: 12,
                files: 1,
                bytes: 120,
                tokens: 24,
                avg_lines: 12,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let rows = compute_diff_rows(&from, &to);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].lang, "Rust");
        assert_eq!(rows[0].delta_code, 2);

        let totals = compute_diff_totals(&rows);
        assert_eq!(totals.delta_code, 2);

        let md = render_diff_md("from", "to", &rows, &totals);

        assert!(!md.trim().is_empty(), "diff markdown must not be empty");
        assert!(md.contains("from"));
        assert!(md.contains("to"));
        assert!(md.contains("Rust"));
        assert!(md.contains("|LOC|"));
        assert!(md.contains("|Lines|"));
        assert!(md.contains("|Files|"));
        assert!(md.contains("|Bytes|"));
        assert!(md.contains("|Tokens|"));
        assert!(md.contains("### Language Movement"));
    }

    #[test]
    fn test_render_diff_md_compact_includes_movement_counts() {
        let from = LangReport {
            rows: vec![LangRow {
                lang: "Rust".to_string(),
                code: 10,
                lines: 10,
                files: 1,
                bytes: 100,
                tokens: 20,
                avg_lines: 10,
            }],
            total: Totals {
                code: 10,
                lines: 10,
                files: 1,
                bytes: 100,
                tokens: 20,
                avg_lines: 10,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };
        let to = LangReport {
            rows: vec![
                LangRow {
                    lang: "Rust".to_string(),
                    code: 12,
                    lines: 12,
                    files: 1,
                    bytes: 120,
                    tokens: 24,
                    avg_lines: 12,
                },
                LangRow {
                    lang: "Python".to_string(),
                    code: 8,
                    lines: 8,
                    files: 1,
                    bytes: 80,
                    tokens: 16,
                    avg_lines: 8,
                },
            ],
            total: Totals {
                code: 20,
                lines: 20,
                files: 2,
                bytes: 200,
                tokens: 40,
                avg_lines: 10,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };
        let rows = compute_diff_rows(&from, &to);
        let totals = compute_diff_totals(&rows);
        let md = render_diff_md_with_options(
            "from",
            "to",
            &rows,
            &totals,
            DiffRenderOptions {
                compact: true,
                color: DiffColorMode::Off,
            },
        );

        assert!(md.contains("|Delta Lines|"));
        assert!(md.contains("|Delta Files|"));
        assert!(md.contains("|Delta Bytes|"));
        assert!(md.contains("|Delta Tokens|"));
        assert!(md.contains("|Languages added|1|"));
        assert!(md.contains("|Languages modified|1|"));
    }

    #[test]
    fn test_compute_diff_rows_language_added() {
        // Tests language being added (was 0, now has code)
        let from = LangReport {
            rows: vec![],
            total: Totals {
                code: 0,
                lines: 0,
                files: 0,
                bytes: 0,
                tokens: 0,
                avg_lines: 0,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let to = LangReport {
            rows: vec![LangRow {
                lang: "Python".to_string(),
                code: 100,
                lines: 120,
                files: 5,
                bytes: 5000,
                tokens: 250,
                avg_lines: 24,
            }],
            total: Totals {
                code: 100,
                lines: 120,
                files: 5,
                bytes: 5000,
                tokens: 250,
                avg_lines: 24,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let rows = compute_diff_rows(&from, &to);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].lang, "Python");
        assert_eq!(rows[0].old_code, 0);
        assert_eq!(rows[0].new_code, 100);
        assert_eq!(rows[0].delta_code, 100);
    }

    #[test]
    fn test_compute_diff_rows_language_removed() {
        // Tests language being removed (had code, now 0)
        let from = LangReport {
            rows: vec![LangRow {
                lang: "Go".to_string(),
                code: 50,
                lines: 60,
                files: 2,
                bytes: 2000,
                tokens: 125,
                avg_lines: 30,
            }],
            total: Totals {
                code: 50,
                lines: 60,
                files: 2,
                bytes: 2000,
                tokens: 125,
                avg_lines: 30,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let to = LangReport {
            rows: vec![],
            total: Totals {
                code: 0,
                lines: 0,
                files: 0,
                bytes: 0,
                tokens: 0,
                avg_lines: 0,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let rows = compute_diff_rows(&from, &to);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].lang, "Go");
        assert_eq!(rows[0].old_code, 50);
        assert_eq!(rows[0].new_code, 0);
        assert_eq!(rows[0].delta_code, -50);
    }

    #[test]
    fn test_compute_diff_rows_unchanged_excluded() {
        // Tests that unchanged languages are excluded from diff
        let report = LangReport {
            rows: vec![LangRow {
                lang: "Rust".to_string(),
                code: 100,
                lines: 100,
                files: 1,
                bytes: 1000,
                tokens: 250,
                avg_lines: 100,
            }],
            total: Totals {
                code: 100,
                lines: 100,
                files: 1,
                bytes: 1000,
                tokens: 250,
                avg_lines: 100,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let rows = compute_diff_rows(&report, &report);
        assert!(rows.is_empty(), "unchanged languages should be excluded");
    }
}
