use std::collections::BTreeMap;

use tokmd_analysis_types::{RateReport, RateRow, RatioReport, RatioRow};
use tokmd_scan::safe_ratio;
use tokmd_types::FileRow;

pub(super) fn build_doc_density_report(
    rows: &[&FileRow],
    comments: usize,
    code_and_comments: usize,
) -> RatioReport {
    build_ratio_report(
        "total",
        comments,
        code_and_comments,
        group_ratio(rows, |r| r.lang.as_str(), |r| (r.comments, r.code)),
        group_ratio(rows, |r| r.module.as_str(), |r| (r.comments, r.code)),
    )
}

pub(super) fn build_whitespace_report(
    rows: &[&FileRow],
    blanks: usize,
    code_and_comments: usize,
) -> RatioReport {
    build_ratio_report(
        "total",
        blanks,
        code_and_comments,
        group_ratio(
            rows,
            |r| r.lang.as_str(),
            |r| (r.blanks, r.code + r.comments),
        ),
        group_ratio(
            rows,
            |r| r.module.as_str(),
            |r| (r.blanks, r.code + r.comments),
        ),
    )
}

pub(super) fn build_verbosity_report(rows: &[&FileRow], bytes: usize, lines: usize) -> RateReport {
    build_rate_report(
        "total",
        bytes,
        lines,
        group_rate(rows, |r| r.lang.as_str(), |r| (r.bytes, r.lines)),
        group_rate(rows, |r| r.module.as_str(), |r| (r.bytes, r.lines)),
    )
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

fn group_ratio<'a, FKey, FVals>(
    rows: &'a [&'a FileRow],
    key_fn: FKey,
    vals_fn: FVals,
) -> BTreeMap<String, (usize, usize)>
where
    FKey: Fn(&'a FileRow) -> &'a str,
    FVals: Fn(&'a FileRow) -> (usize, usize),
{
    let mut map: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for row in rows {
        let key = key_fn(row);
        let (numer, denom_part) = vals_fn(row);
        let entry = map.entry(key).or_insert((0, 0));
        entry.0 += numer;
        entry.1 += denom_part;
    }
    map.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

fn group_rate<'a, FKey, FVals>(
    rows: &'a [&'a FileRow],
    key_fn: FKey,
    vals_fn: FVals,
) -> BTreeMap<String, (usize, usize)>
where
    FKey: Fn(&'a FileRow) -> &'a str,
    FVals: Fn(&'a FileRow) -> (usize, usize),
{
    let mut map: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for row in rows {
        let key = key_fn(row);
        let (numer, denom) = vals_fn(row);
        let entry = map.entry(key).or_insert((0, 0));
        entry.0 += numer;
        entry.1 += denom;
    }
    map.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}
