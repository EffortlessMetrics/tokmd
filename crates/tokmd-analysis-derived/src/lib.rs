use std::collections::{BTreeMap, BTreeSet};

use tokmd_analysis_types::{
    BoilerplateReport, CocomoReport, ContextWindowReport, DerivedReport, DerivedTotals,
    DistributionReport, FileStatRow, HistogramBucket, IntegrityReport, LangPurityReport,
    LangPurityRow, MaxFileReport, MaxFileRow, NestingReport, NestingRow, PolyglotReport,
    RateReport, RateRow, RatioReport, RatioRow, ReadingTimeReport, TestDensityReport, TopOffenders,
};
use tokmd_analysis_util::{empty_file_row, is_infra_lang, is_test_path, path_depth};
use tokmd_export_tree::render_analysis_tree;
use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};
use tokmd_types::{ExportData, FileKind, FileRow};

const LINES_PER_MINUTE: usize = 20;
const TOP_N: usize = 10;
const MIN_DOC_LINES: usize = 50;
const MIN_DENSE_LINES: usize = 10;

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

    let doc_density = build_ratio_report(
        "total",
        totals.comments,
        totals.code + totals.comments,
        group_ratio(&parents, |r| r.lang.clone(), |r| (r.comments, r.code)),
        group_ratio(&parents, |r| r.module.clone(), |r| (r.comments, r.code)),
    );

    let whitespace = build_ratio_report(
        "total",
        totals.blanks,
        totals.code + totals.comments,
        group_ratio(
            &parents,
            |r| r.lang.clone(),
            |r| (r.blanks, r.code + r.comments),
        ),
        group_ratio(
            &parents,
            |r| r.module.clone(),
            |r| (r.blanks, r.code + r.comments),
        ),
    );

    let verbosity = build_rate_report(
        "total",
        totals.bytes,
        totals.lines,
        group_rate(&parents, |r| r.lang.clone(), |r| (r.bytes, r.lines)),
        group_rate(&parents, |r| r.module.clone(), |r| (r.bytes, r.lines)),
    );

    let file_stats = build_file_stats(&parents);

    let max_file = build_max_file_report(&file_stats);

    let lang_purity = build_lang_purity_report(&parents);

    let nesting = build_nesting_report(&file_stats);

    let test_density = build_test_density_report(&parents);

    let boilerplate = build_boilerplate_report(&parents);

    let polyglot = build_polyglot_report(&parents);

    let distribution = build_distribution_report(&parents);

    let histogram = build_histogram(&parents);

    let top = build_top_offenders(&file_stats);

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
        let (a, b, c, d) = (2.4, 1.05, 2.5, 0.38);
        let effort = a * kloc.powf(b);
        let duration = c * effort.powf(d);
        let staff = if duration == 0.0 {
            0.0
        } else {
            effort / duration
        };
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

fn build_ratio_report(
    total_key: &str,
    total_numer: usize,
    total_denom: usize,
    by_lang: BTreeMap<String, (usize, usize)>,
    by_module: BTreeMap<String, (usize, usize)>,
) -> RatioReport {
    RatioReport {
        total: RatioRow {
            key: total_key.to_string(),
            numerator: total_numer,
            denominator: total_denom,
            ratio: safe_ratio(total_numer, total_denom),
        },
        by_lang: build_ratio_rows(by_lang),
        by_module: build_ratio_rows(by_module),
    }
}

fn build_rate_report(
    total_key: &str,
    total_numer: usize,
    total_denom: usize,
    by_lang: BTreeMap<String, (usize, usize)>,
    by_module: BTreeMap<String, (usize, usize)>,
) -> RateReport {
    RateReport {
        total: RateRow {
            key: total_key.to_string(),
            numerator: total_numer,
            denominator: total_denom,
            rate: safe_ratio(total_numer, total_denom),
        },
        by_lang: build_rate_rows(by_lang),
        by_module: build_rate_rows(by_module),
    }
}

fn build_ratio_rows(map: BTreeMap<String, (usize, usize)>) -> Vec<RatioRow> {
    let mut rows: Vec<RatioRow> = map
        .into_iter()
        .map(|(key, (numer, denom))| RatioRow {
            key,
            numerator: numer,
            denominator: denom,
            ratio: safe_ratio(numer, denom),
        })
        .collect();

    rows.sort_by(|a, b| {
        b.ratio
            .partial_cmp(&a.ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.key.cmp(&b.key))
    });
    rows
}

fn build_rate_rows(map: BTreeMap<String, (usize, usize)>) -> Vec<RateRow> {
    let mut rows: Vec<RateRow> = map
        .into_iter()
        .map(|(key, (numer, denom))| RateRow {
            key,
            numerator: numer,
            denominator: denom,
            rate: safe_ratio(numer, denom),
        })
        .collect();

    rows.sort_by(|a, b| {
        b.rate
            .partial_cmp(&a.rate)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.key.cmp(&b.key))
    });
    rows
}

fn group_ratio<FKey, FVals>(
    rows: &[&FileRow],
    key_fn: FKey,
    vals_fn: FVals,
) -> BTreeMap<String, (usize, usize)>
where
    FKey: Fn(&FileRow) -> String,
    FVals: Fn(&FileRow) -> (usize, usize),
{
    let mut map: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for row in rows {
        let key = key_fn(row);
        let (numer, denom_part) = vals_fn(row);
        let entry = map.entry(key).or_insert((0, 0));
        entry.0 += numer;
        entry.1 += denom_part;
    }
    map
}

fn group_rate<FKey, FVals>(
    rows: &[&FileRow],
    key_fn: FKey,
    vals_fn: FVals,
) -> BTreeMap<String, (usize, usize)>
where
    FKey: Fn(&FileRow) -> String,
    FVals: Fn(&FileRow) -> (usize, usize),
{
    let mut map: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for row in rows {
        let key = key_fn(row);
        let (numer, denom) = vals_fn(row);
        let entry = map.entry(key).or_insert((0, 0));
        entry.0 += numer;
        entry.1 += denom;
    }
    map
}

fn build_file_stats(rows: &[&FileRow]) -> Vec<FileStatRow> {
    rows.iter()
        .map(|r| FileStatRow {
            path: r.path.clone(),
            module: r.module.clone(),
            lang: r.lang.clone(),
            code: r.code,
            comments: r.comments,
            blanks: r.blanks,
            lines: r.lines,
            bytes: r.bytes,
            tokens: r.tokens,
            doc_pct: if r.code + r.comments == 0 {
                None
            } else {
                Some(safe_ratio(r.comments, r.code + r.comments))
            },
            bytes_per_line: if r.lines == 0 {
                None
            } else {
                Some(safe_ratio(r.bytes, r.lines))
            },
            depth: path_depth(&r.path),
        })
        .collect()
}

fn build_max_file_report(rows: &[FileStatRow]) -> MaxFileReport {
    let mut overall = rows
        .iter()
        .max_by(|a, b| a.lines.cmp(&b.lines).then_with(|| a.path.cmp(&b.path)))
        .cloned()
        .unwrap_or_else(empty_file_row);

    if rows.is_empty() {
        overall = empty_file_row();
    }

    let mut by_lang: BTreeMap<String, FileStatRow> = BTreeMap::new();
    let mut by_module: BTreeMap<String, FileStatRow> = BTreeMap::new();

    for row in rows {
        by_lang
            .entry(row.lang.clone())
            .and_modify(|existing| {
                if row.lines > existing.lines
                    || (row.lines == existing.lines && row.path < existing.path)
                {
                    *existing = row.clone();
                }
            })
            .or_insert_with(|| row.clone());

        by_module
            .entry(row.module.clone())
            .and_modify(|existing| {
                if row.lines > existing.lines
                    || (row.lines == existing.lines && row.path < existing.path)
                {
                    *existing = row.clone();
                }
            })
            .or_insert_with(|| row.clone());
    }

    MaxFileReport {
        overall,
        by_lang: by_lang
            .into_iter()
            .map(|(key, file)| MaxFileRow { key, file })
            .collect(),
        by_module: by_module
            .into_iter()
            .map(|(key, file)| MaxFileRow { key, file })
            .collect(),
    }
}

fn build_lang_purity_report(rows: &[&FileRow]) -> LangPurityReport {
    let mut by_module: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();

    for row in rows {
        let entry = by_module.entry(row.module.clone()).or_default();
        *entry.entry(row.lang.clone()).or_insert(0) += row.lines;
    }

    let mut out = Vec::new();
    for (module, langs) in by_module {
        let mut total = 0usize;
        let mut dominant_lang = String::new();
        let mut dominant_lines = 0usize;
        for (lang, lines) in &langs {
            total += *lines;
            if *lines > dominant_lines || (*lines == dominant_lines && lang < &dominant_lang) {
                dominant_lines = *lines;
                dominant_lang = lang.clone();
            }
        }
        let pct = if total == 0 {
            0.0
        } else {
            safe_ratio(dominant_lines, total)
        };
        out.push(LangPurityRow {
            module,
            lang_count: langs.len(),
            dominant_lang,
            dominant_lines,
            dominant_pct: pct,
        });
    }

    out.sort_by(|a, b| a.module.cmp(&b.module));
    LangPurityReport { rows: out }
}

fn build_nesting_report(rows: &[FileStatRow]) -> NestingReport {
    if rows.is_empty() {
        return NestingReport {
            max: 0,
            avg: 0.0,
            by_module: vec![],
        };
    }

    let mut total_depth = 0usize;
    let mut max_depth = 0usize;
    let mut by_module: BTreeMap<String, Vec<usize>> = BTreeMap::new();

    for row in rows {
        total_depth += row.depth;
        max_depth = max_depth.max(row.depth);
        by_module
            .entry(row.module.clone())
            .or_default()
            .push(row.depth);
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
            key: module,
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
    let mut infra_langs: BTreeSet<String> = BTreeSet::new();

    for row in rows {
        if is_infra_lang(&row.lang) {
            infra_lines += row.lines;
            infra_langs.insert(row.lang.clone());
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
        infra_langs: infra_langs.into_iter().collect(),
    }
}

fn build_polyglot_report(rows: &[&FileRow]) -> PolyglotReport {
    let mut by_lang: BTreeMap<String, usize> = BTreeMap::new();
    let mut total = 0usize;

    for row in rows {
        *by_lang.entry(row.lang.clone()).or_insert(0) += row.code;
        total += row.code;
    }

    let mut entropy = 0.0;
    let mut dominant_lang = String::new();
    let mut dominant_lines = 0usize;

    for (lang, lines) in &by_lang {
        if *lines > dominant_lines || (*lines == dominant_lines && lang < &dominant_lang) {
            dominant_lines = *lines;
            dominant_lang = lang.clone();
        }
        if total > 0 && *lines > 0 {
            let p = *lines as f64 / total as f64;
            entropy -= p * p.log2();
        }
    }

    let dominant_pct = if total == 0 {
        0.0
    } else {
        safe_ratio(dominant_lines, total)
    };

    PolyglotReport {
        lang_count: by_lang.len(),
        entropy: round_f64(entropy, 4),
        dominant_lang,
        dominant_lines,
        dominant_pct,
    }
}

fn build_distribution_report(rows: &[&FileRow]) -> DistributionReport {
    let mut sizes: Vec<usize> = rows.iter().map(|r| r.lines).collect();
    sizes.sort();

    if sizes.is_empty() {
        return DistributionReport {
            count: 0,
            min: 0,
            max: 0,
            mean: 0.0,
            median: 0.0,
            p90: 0.0,
            p99: 0.0,
            gini: 0.0,
        };
    }

    let count = sizes.len();
    let sum: usize = sizes.iter().sum();
    let mean = sum as f64 / count as f64;
    let median = if count % 2 == 1 {
        sizes[count / 2] as f64
    } else {
        (sizes[count / 2 - 1] as f64 + sizes[count / 2] as f64) / 2.0
    };
    let p90 = percentile(&sizes, 0.90);
    let p99 = percentile(&sizes, 0.99);
    let gini = gini_coefficient(&sizes);

    DistributionReport {
        count,
        min: *sizes.first().unwrap_or(&0),
        max: *sizes.last().unwrap_or(&0),
        mean: round_f64(mean, 2),
        median: round_f64(median, 2),
        p90: round_f64(p90, 2),
        p99: round_f64(p99, 2),
        gini: round_f64(gini, 4),
    }
}

fn build_histogram(rows: &[&FileRow]) -> Vec<HistogramBucket> {
    let total = rows.len();
    let buckets = vec![
        ("Tiny", 0, Some(50)),
        ("Small", 51, Some(200)),
        ("Medium", 201, Some(500)),
        ("Large", 501, Some(1000)),
        ("Huge", 1001, None),
    ];

    let mut counts = vec![0usize; buckets.len()];
    for row in rows {
        let size = row.lines;
        for (idx, (_label, min, max)) in buckets.iter().enumerate() {
            let in_range = if let Some(max) = max {
                size >= *min && size <= *max
            } else {
                size >= *min
            };
            if in_range {
                counts[idx] += 1;
                break;
            }
        }
    }

    buckets
        .into_iter()
        .zip(counts)
        .map(|((label, min, max), files)| HistogramBucket {
            label: label.to_string(),
            min,
            max,
            files,
            pct: if total == 0 {
                0.0
            } else {
                round_f64(files as f64 / total as f64, 4)
            },
        })
        .collect()
}

pub fn build_tree(export: &ExportData) -> String {
    render_analysis_tree(export)
}

fn build_top_offenders(rows: &[FileStatRow]) -> TopOffenders {
    let mut by_lines = rows.to_vec();
    by_lines.sort_by(|a, b| b.lines.cmp(&a.lines).then_with(|| a.path.cmp(&b.path)));

    let mut by_tokens = rows.to_vec();
    by_tokens.sort_by(|a, b| b.tokens.cmp(&a.tokens).then_with(|| a.path.cmp(&b.path)));

    let mut by_bytes = rows.to_vec();
    by_bytes.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.path.cmp(&b.path)));

    let mut least_doc: Vec<FileStatRow> = rows
        .iter()
        .filter(|r| r.lines >= MIN_DOC_LINES)
        .cloned()
        .collect();
    least_doc.sort_by(|a, b| {
        let a_doc = a.doc_pct.unwrap_or(0.0);
        let b_doc = b.doc_pct.unwrap_or(0.0);
        a_doc
            .partial_cmp(&b_doc)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.lines.cmp(&a.lines))
            .then_with(|| a.path.cmp(&b.path))
    });

    let mut dense: Vec<FileStatRow> = rows
        .iter()
        .filter(|r| r.lines >= MIN_DENSE_LINES)
        .cloned()
        .collect();
    dense.sort_by(|a, b| {
        let a_rate = a.bytes_per_line.unwrap_or(0.0);
        let b_rate = b.bytes_per_line.unwrap_or(0.0);
        b_rate
            .partial_cmp(&a_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });

    TopOffenders {
        largest_lines: by_lines.into_iter().take(TOP_N).collect(),
        largest_tokens: by_tokens.into_iter().take(TOP_N).collect(),
        largest_bytes: by_bytes.into_iter().take(TOP_N).collect(),
        least_documented: least_doc.into_iter().take(TOP_N).collect(),
        most_dense: dense.into_iter().take(TOP_N).collect(),
    }
}

fn build_integrity_report(rows: &[&FileRow]) -> IntegrityReport {
    let mut sorted_rows = rows.to_vec();
    sorted_rows.sort_unstable_by(|&a, &b| compare_integrity_rows(a, b));

    let mut hasher = blake3::Hasher::new();
    let mut first = true;
    for row in sorted_rows {
        if !first {
            hasher.update(b"\n");
        }
        first = false;
        hasher.update(row.path.as_bytes());
        hasher.update(b":");
        hasher.update(row.bytes.to_string().as_bytes());
        hasher.update(b":");
        hasher.update(row.lines.to_string().as_bytes());
    }

    IntegrityReport {
        algo: "blake3".to_string(),
        hash: hasher.finalize().to_hex().to_string(),
        entries: rows.len(),
    }
}

fn compare_integrity_rows(a: &FileRow, b: &FileRow) -> std::cmp::Ordering {
    let a_bytes = a.path.as_bytes();
    let b_bytes = b.path.as_bytes();
    let min_len = a_bytes.len().min(b_bytes.len());

    // Fast slice compare for common prefix
    let ord = a_bytes[..min_len].cmp(&b_bytes[..min_len]);
    if ord != std::cmp::Ordering::Equal {
        return ord;
    }

    // Paths are identical or one is prefix of other
    if a_bytes.len() == b_bytes.len() {
        // Identical paths. Compare numbers.
        // We must emulate string sort of "bytes:lines".
        // Format them to ensure correct string sort order.
        let a_str = format!("{}:{}", a.bytes, a.lines);
        let b_str = format!("{}:{}", b.bytes, b.lines);
        return a_str.cmp(&b_str);
    }

    // One is shorter.
    // The separator is ':'.
    if a_bytes.len() < b_bytes.len() {
        // a is prefix of b.
        // Effective string a: "path:..."
        // Effective string b: "path..."
        // Compare ':' vs b[min_len]
        b':'.cmp(&b_bytes[min_len])
    } else {
        // b is prefix of a.
        // Effective string a: "path..."
        // Effective string b: "path:..."
        // Compare a[min_len] vs ':'
        a_bytes[min_len].cmp(&b':')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_types::{FileKind, FileRow};

    fn make_row(path: &str, bytes: usize, lines: usize) -> FileRow {
        FileRow {
            path: path.to_string(),
            module: "mod".to_string(),
            lang: "rust".to_string(),
            kind: FileKind::Parent,
            code: 0,
            comments: 0,
            blanks: 0,
            lines,
            bytes,
            tokens: 0,
        }
    }

    #[test]
    fn cocomo_none_for_zero_code() {
        let export = ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 1,
            children: tokmd_types::ChildIncludeMode::Separate,
        };
        let report = derive_report(&export, None);
        assert!(report.cocomo.is_none());
    }

    #[test]
    fn cocomo_computed_for_nonzero_code() {
        let row = FileRow {
            path: "src/main.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 5000,
            comments: 100,
            blanks: 50,
            lines: 5150,
            bytes: 100_000,
            tokens: 20_000,
        };
        let export = ExportData {
            rows: vec![row],
            module_roots: vec![],
            module_depth: 1,
            children: tokmd_types::ChildIncludeMode::Separate,
        };
        let report = derive_report(&export, None);
        let cocomo = report.cocomo.as_ref().unwrap();
        assert_eq!(cocomo.mode, "organic");
        assert!((cocomo.kloc - 5.0).abs() < f64::EPSILON);
        assert!(cocomo.effort_pm > 0.0);
        assert!(cocomo.duration_months > 0.0);
        assert!(cocomo.staff > 0.0);
    }

    #[test]
    fn context_window_fits_when_tokens_within_budget() {
        let row = FileRow {
            path: "a.rs".to_string(),
            module: "mod".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 0,
            blanks: 0,
            lines: 10,
            bytes: 100,
            tokens: 50,
        };
        let export = ExportData {
            rows: vec![row],
            module_roots: vec![],
            module_depth: 1,
            children: tokmd_types::ChildIncludeMode::Separate,
        };
        let report = derive_report(&export, Some(1000));
        let cw = report.context_window.unwrap();
        assert!(cw.fits);
        assert_eq!(cw.total_tokens, 50);
        assert_eq!(cw.window_tokens, 1000);

        // zero window edge case
        let report2 = derive_report(&export, Some(0));
        let cw2 = report2.context_window.unwrap();
        assert!(!cw2.fits);
        assert!((cw2.pct - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn histogram_empty_input() {
        let export = ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 1,
            children: tokmd_types::ChildIncludeMode::Separate,
        };
        let report = derive_report(&export, None);
        for bucket in &report.histogram {
            assert_eq!(bucket.files, 0);
            assert!((bucket.pct - 0.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_compare_integrity_rows_matches_string_sort() {
        let cases = vec![
            ("a", 10, 10, "b", 10, 10),
            ("a", 10, 10, "a", 10, 10),
            ("a", 10, 10, "a", 20, 10),
            ("a", 100, 10, "a", 20, 10), // "100" < "20" as string? No, '1' < '2'. So "100" < "20".
            ("a", 10, 10, "a.b", 10, 10),
            ("a.b", 10, 10, "a", 10, 10),
            ("foo", 10, 10, "foo.bar", 10, 10),
            ("foo.bar", 10, 10, "foo", 10, 10),
            ("foo", 10, 10, "foo_bar", 10, 10),
        ];

        for (p1, b1, l1, p2, b2, l2) in cases {
            let r1 = make_row(p1, b1, l1);
            let r2 = make_row(p2, b2, l2);

            let s1 = format!("{}:{}:{}", p1, b1, l1);
            let s2 = format!("{}:{}:{}", p2, b2, l2);
            let expected = s1.cmp(&s2);
            let actual = compare_integrity_rows(&r1, &r2);

            assert_eq!(actual, expected, "Failed for {} vs {}", s1, s2);
        }
    }
}
