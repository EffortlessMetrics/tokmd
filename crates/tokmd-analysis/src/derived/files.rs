use tokmd_analysis_types::{FileStatRow, MaxFileReport, MaxFileRow, TopOffenders};
use tokmd_analysis_types::{empty_file_row, path_depth};
use tokmd_scan::safe_ratio;
use tokmd_types::FileRow;

const TOP_N: usize = 10;
const MIN_DOC_LINES: usize = 50;
const MIN_DENSE_LINES: usize = 10;

pub(super) fn build_file_stats(rows: &[&FileRow]) -> Vec<FileStatRow> {
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

pub(super) fn build_max_file_report(rows: &[FileStatRow]) -> MaxFileReport {
    let mut overall = rows
        .iter()
        .max_by(|a, b| a.lines.cmp(&b.lines).then_with(|| a.path.cmp(&b.path)))
        .cloned()
        .unwrap_or_else(empty_file_row);

    if rows.is_empty() {
        overall = empty_file_row();
    }

    let mut by_lang: std::collections::BTreeMap<&str, &FileStatRow> =
        std::collections::BTreeMap::new();
    let mut by_module: std::collections::BTreeMap<&str, &FileStatRow> =
        std::collections::BTreeMap::new();

    for row in rows {
        if let Some(existing) = by_lang.get_mut(row.lang.as_str()) {
            if row.lines > existing.lines
                || (row.lines == existing.lines && row.path < existing.path)
            {
                *existing = row;
            }
        } else {
            by_lang.insert(row.lang.as_str(), row);
        }

        if let Some(existing) = by_module.get_mut(row.module.as_str()) {
            if row.lines > existing.lines
                || (row.lines == existing.lines && row.path < existing.path)
            {
                *existing = row;
            }
        } else {
            by_module.insert(row.module.as_str(), row);
        }
    }

    MaxFileReport {
        overall,
        by_lang: by_lang
            .into_iter()
            .map(|(key, file)| MaxFileRow {
                key: key.to_string(),
                file: file.clone(),
            })
            .collect(),
        by_module: by_module
            .into_iter()
            .map(|(key, file)| MaxFileRow {
                key: key.to_string(),
                file: file.clone(),
            })
            .collect(),
    }
}

pub(super) fn build_top_offenders(rows: &[FileStatRow]) -> TopOffenders {
    fn extract_top<F>(rows: &[FileStatRow], mut compare: F) -> Vec<FileStatRow>
    where
        F: FnMut(&&FileStatRow, &&FileStatRow) -> std::cmp::Ordering,
    {
        let mut by_key: Vec<&FileStatRow> = rows.iter().collect();
        if by_key.len() > TOP_N {
            let (_, _, _) = by_key.select_nth_unstable_by(TOP_N, &mut compare);
            by_key.truncate(TOP_N);
            by_key.sort_unstable_by(compare);
            by_key.into_iter().cloned().collect()
        } else {
            by_key.sort_unstable_by(compare);
            by_key.into_iter().cloned().collect()
        }
    }

    let largest_lines = extract_top(rows, |a, b| {
        b.lines.cmp(&a.lines).then_with(|| a.path.cmp(&b.path))
    });
    let largest_tokens = extract_top(rows, |a, b| {
        b.tokens.cmp(&a.tokens).then_with(|| a.path.cmp(&b.path))
    });
    let largest_bytes = extract_top(rows, |a, b| {
        b.bytes.cmp(&a.bytes).then_with(|| a.path.cmp(&b.path))
    });

    let doc_filtered: Vec<FileStatRow> = rows
        .iter()
        .filter(|r| r.lines >= MIN_DOC_LINES)
        .cloned()
        .collect();
    let least_documented = extract_top(&doc_filtered, |a, b| {
        let a_doc = a.doc_pct.unwrap_or(0.0);
        let b_doc = b.doc_pct.unwrap_or(0.0);
        a_doc
            .partial_cmp(&b_doc)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.lines.cmp(&a.lines))
            .then_with(|| a.path.cmp(&b.path))
    });

    let dense_filtered: Vec<FileStatRow> = rows
        .iter()
        .filter(|r| r.lines >= MIN_DENSE_LINES)
        .cloned()
        .collect();
    let most_dense = extract_top(&dense_filtered, |a, b| {
        let a_rate = a.bytes_per_line.unwrap_or(0.0);
        let b_rate = b.bytes_per_line.unwrap_or(0.0);
        b_rate
            .partial_cmp(&a_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });

    TopOffenders {
        largest_lines,
        largest_tokens,
        largest_bytes,
        least_documented,
        most_dense,
    }
}
