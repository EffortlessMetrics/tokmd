use std::collections::{BTreeMap, BTreeSet};

use tokmd_analysis_types::{
    BoilerplateReport, CocomoReport, ContextWindowReport, DerivedReport, DerivedTotals,
    NestingReport, NestingRow, ReadingTimeReport, TestDensityReport,
};
use tokmd_analysis_types::{is_infra_lang, is_test_path};
use tokmd_format::render_analysis_tree;
use tokmd_scan::{round_f64, safe_ratio};
use tokmd_types::{ExportData, FileKind, FileRow};

use crate::cocomo81_core::{COCOMO81_COEFFICIENTS, cocomo81_effort_pm};

mod distribution;
mod files;
mod integrity;
mod languages;
mod ratios;
use distribution::{build_distribution_report, build_histogram};
use files::{build_max_file_report, build_top_offenders};
use integrity::build_integrity_report;
use languages::{build_lang_purity_report, build_polyglot_report};
use ratios::{build_doc_density_report, build_verbosity_report, build_whitespace_report};

const LINES_PER_MINUTE: usize = 20;

pub fn derive_report(export: &ExportData, window_tokens: Option<usize>) -> DerivedReport {
    let parents: Vec<&FileRow> = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .collect();

    let mut totals = DerivedTotals {
        files: parents.len(),
        code: 0,
        comments: 0,
        blanks: 0,
        lines: 0,
        bytes: 0,
        tokens: 0,
    };

    for row in &parents {
        totals.code += row.code;
        totals.comments += row.comments;
        totals.blanks += row.blanks;
        totals.lines += row.lines;
        totals.bytes += row.bytes;
        totals.tokens += row.tokens;
    }

    let doc_density =
        build_doc_density_report(&parents, totals.comments, totals.code + totals.comments);

    let whitespace =
        build_whitespace_report(&parents, totals.blanks, totals.code + totals.comments);

    let verbosity = build_verbosity_report(&parents, totals.bytes, totals.lines);

    let max_file = build_max_file_report(&parents);

    let lang_purity = build_lang_purity_report(&parents);

    let nesting = build_nesting_report(&parents);

    let test_density = build_test_density_report(&parents);

    let boilerplate = build_boilerplate_report(&parents);

    let polyglot = build_polyglot_report(&parents);

    let distribution = build_distribution_report(&parents);

    let histogram = build_histogram(&parents);

    let top = build_top_offenders(&parents);

    let reading_time = ReadingTimeReport {
        minutes: round_f64(totals.code as f64 / LINES_PER_MINUTE as f64, 2),
        lines_per_minute: LINES_PER_MINUTE,
        basis_lines: totals.code,
    };

    let context_window = window_tokens.map(|window| {
        let pct = if window == 0 {
            0.0
        } else {
            round_f64(totals.tokens as f64 / window as f64, 4)
        };
        ContextWindowReport {
            window_tokens: window,
            total_tokens: totals.tokens,
            pct,
            fits: totals.tokens <= window,
        }
    });

    let cocomo = if totals.code == 0 {
        None
    } else {
        let kloc = totals.code as f64 / 1000.0;
        let (a, b, c, d) = COCOMO81_COEFFICIENTS;
        let (effort, duration, staff, _) = cocomo81_effort_pm(kloc);
        Some(CocomoReport {
            mode: "organic".to_string(),
            kloc: round_f64(kloc, 4),
            effort_pm: round_f64(effort, 2),
            duration_months: round_f64(duration, 2),
            staff: round_f64(staff, 2),
            a,
            b,
            c,
            d,
        })
    };

    let integrity = build_integrity_report(&parents);

    DerivedReport {
        totals,
        doc_density,
        whitespace,
        verbosity,
        max_file,
        lang_purity,
        nesting,
        test_density,
        boilerplate,
        polyglot,
        distribution,
        histogram,
        top,
        tree: None,
        reading_time,
        context_window,
        cocomo,
        todo: None,
        integrity,
    }
}

fn build_nesting_report(rows: &[&FileRow]) -> NestingReport {
    if rows.is_empty() {
        return NestingReport {
            max: 0,
            avg: 0.0,
            by_module: vec![],
        };
    }

    let mut total_depth = 0usize;
    let mut max_depth = 0usize;
    let mut by_module: BTreeMap<&str, Vec<usize>> = BTreeMap::new();

    for row in rows {
        let depth = tokmd_analysis_types::path_depth(&row.path);
        total_depth += depth;
        max_depth = max_depth.max(depth);
        if let Some(existing) = by_module.get_mut(row.module.as_str()) {
            existing.push(depth);
        } else {
            by_module.insert(row.module.as_str(), vec![depth]);
        }
    }

    let avg = round_f64(total_depth as f64 / rows.len() as f64, 2);

    let mut module_rows = Vec::new();
    for (module, depths) in by_module {
        let max = depths.iter().copied().max().unwrap_or(0);
        let sum: usize = depths.iter().sum();
        let avg = if depths.is_empty() {
            0.0
        } else {
            round_f64(sum as f64 / depths.len() as f64, 2)
        };
        module_rows.push(NestingRow {
            key: module.to_string(),
            max,
            avg,
        });
    }

    NestingReport {
        max: max_depth,
        avg,
        by_module: module_rows,
    }
}

fn build_test_density_report(rows: &[&FileRow]) -> TestDensityReport {
    let mut test_lines = 0usize;
    let mut prod_lines = 0usize;
    let mut test_files = 0usize;
    let mut prod_files = 0usize;

    for row in rows {
        if is_test_path(&row.path) {
            test_lines += row.code;
            test_files += 1;
        } else {
            prod_lines += row.code;
            prod_files += 1;
        }
    }

    let total = test_lines + prod_lines;
    let ratio = if total == 0 {
        0.0
    } else {
        safe_ratio(test_lines, total)
    };

    TestDensityReport {
        test_lines,
        prod_lines,
        test_files,
        prod_files,
        ratio,
    }
}

fn build_boilerplate_report(rows: &[&FileRow]) -> BoilerplateReport {
    let mut infra_lines = 0usize;
    let mut logic_lines = 0usize;
    let mut infra_langs: BTreeSet<&str> = BTreeSet::new();

    for row in rows {
        if is_infra_lang(&row.lang) {
            infra_lines += row.lines;
            if !infra_langs.contains(row.lang.as_str()) {
                infra_langs.insert(row.lang.as_str());
            }
        } else {
            logic_lines += row.lines;
        }
    }

    let total = infra_lines + logic_lines;
    let ratio = if total == 0 {
        0.0
    } else {
        safe_ratio(infra_lines, total)
    };

    BoilerplateReport {
        infra_lines,
        logic_lines,
        ratio,
        infra_langs: infra_langs.into_iter().map(String::from).collect(),
    }
}

pub fn build_tree(export: &ExportData) -> String {
    render_analysis_tree(export)
}

#[cfg(test)]
mod tests;
