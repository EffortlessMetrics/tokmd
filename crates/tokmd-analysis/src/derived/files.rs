use tokmd_analysis_types::{FileStatRow, MaxFileReport, MaxFileRow, TopOffenders};
use tokmd_analysis_types::{empty_file_row, path_depth};
use tokmd_scan::safe_ratio;
use tokmd_types::FileRow;

const TOP_N: usize = 10;
const MIN_DOC_LINES: usize = 50;
const MIN_DENSE_LINES: usize = 10;

pub(crate) fn to_stat_row(r: &FileRow) -> FileStatRow {
    FileStatRow {
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
    }
}

pub(super) fn build_max_file_report(rows: &[&FileRow]) -> MaxFileReport {
    let mut overall: Option<&FileRow> = None;
    let mut by_lang: std::collections::BTreeMap<&str, &FileRow> = std::collections::BTreeMap::new();
    let mut by_module: std::collections::BTreeMap<&str, &FileRow> =
        std::collections::BTreeMap::new();

    for row in rows {
        if let Some(existing) = overall {
            if row.lines > existing.lines
                || (row.lines == existing.lines && row.path < existing.path)
            {
                overall = Some(*row);
            }
        } else {
            overall = Some(*row);
        }

        if let Some(existing) = by_lang.get_mut(row.lang.as_str()) {
            if row.lines > existing.lines
                || (row.lines == existing.lines && row.path < existing.path)
            {
                *existing = *row;
            }
        } else {
            by_lang.insert(row.lang.as_str(), *row);
        }

        if let Some(existing) = by_module.get_mut(row.module.as_str()) {
            if row.lines > existing.lines
                || (row.lines == existing.lines && row.path < existing.path)
            {
                *existing = *row;
            }
        } else {
            by_module.insert(row.module.as_str(), *row);
        }
    }

    MaxFileReport {
        overall: overall.map(to_stat_row).unwrap_or_else(empty_file_row),
        by_lang: by_lang
            .into_iter()
            .map(|(key, file)| MaxFileRow {
                key: key.to_string(),
                file: to_stat_row(file),
            })
            .collect(),
        by_module: by_module
            .into_iter()
            .map(|(key, file)| MaxFileRow {
                key: key.to_string(),
                file: to_stat_row(file),
            })
            .collect(),
    }
}

pub(super) fn build_top_offenders(rows: &[&FileRow]) -> TopOffenders {
    let mut by_lines: Vec<&FileRow> = rows.to_vec();
    by_lines.sort_by(|a, b| b.lines.cmp(&a.lines).then_with(|| a.path.cmp(&b.path)));
    by_lines.truncate(TOP_N);

    let mut by_tokens: Vec<&FileRow> = rows.to_vec();
    by_tokens.sort_by(|a, b| b.tokens.cmp(&a.tokens).then_with(|| a.path.cmp(&b.path)));
    by_tokens.truncate(TOP_N);

    let mut by_bytes: Vec<&FileRow> = rows.to_vec();
    by_bytes.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.path.cmp(&b.path)));
    by_bytes.truncate(TOP_N);

    let mut least_doc: Vec<&FileRow> = rows
        .iter()
        .copied()
        .filter(|r| r.lines >= MIN_DOC_LINES)
        .collect();
    least_doc.sort_by(|a, b| {
        let a_doc = if a.code + a.comments == 0 {
            0.0
        } else {
            safe_ratio(a.comments, a.code + a.comments)
        };
        let b_doc = if b.code + b.comments == 0 {
            0.0
        } else {
            safe_ratio(b.comments, b.code + b.comments)
        };
        a_doc
            .partial_cmp(&b_doc)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.lines.cmp(&a.lines))
            .then_with(|| a.path.cmp(&b.path))
    });
    least_doc.truncate(TOP_N);

    let mut dense: Vec<&FileRow> = rows
        .iter()
        .copied()
        .filter(|r| r.lines >= MIN_DENSE_LINES)
        .collect();
    dense.sort_by(|a, b| {
        let a_rate = if a.lines == 0 {
            0.0
        } else {
            safe_ratio(a.bytes, a.lines)
        };
        let b_rate = if b.lines == 0 {
            0.0
        } else {
            safe_ratio(b.bytes, b.lines)
        };
        b_rate
            .partial_cmp(&a_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });
    dense.truncate(TOP_N);

    TopOffenders {
        largest_lines: by_lines.into_iter().map(to_stat_row).collect(),
        largest_tokens: by_tokens.into_iter().map(to_stat_row).collect(),
        largest_bytes: by_bytes.into_iter().map(to_stat_row).collect(),
        least_documented: least_doc.into_iter().map(to_stat_row).collect(),
        most_dense: dense.into_iter().map(to_stat_row).collect(),
    }
}
