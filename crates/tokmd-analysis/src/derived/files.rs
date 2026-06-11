use tokmd_analysis_types::{FileStatRow, MaxFileReport, MaxFileRow, TopOffenders};
use tokmd_analysis_types::{empty_file_row, path_depth};
use tokmd_scan::safe_ratio;
use tokmd_types::FileRow;

const TOP_N: usize = 10;
const MIN_DOC_LINES: usize = 50;
const MIN_DENSE_LINES: usize = 10;

#[derive(Clone, Copy)]
pub(super) struct FileStatView<'a> {
    pub path: &'a str,
    pub module: &'a str,
    pub lang: &'a str,
    pub code: usize,
    pub comments: usize,
    pub blanks: usize,
    pub lines: usize,
    pub bytes: usize,
    pub tokens: usize,
    pub doc_pct: Option<f64>,
    pub bytes_per_line: Option<f64>,
    pub depth: usize,
}

impl<'a> FileStatView<'a> {
    pub(super) fn into_row(self) -> FileStatRow {
        FileStatRow {
            path: self.path.to_string(),
            module: self.module.to_string(),
            lang: self.lang.to_string(),
            code: self.code,
            comments: self.comments,
            blanks: self.blanks,
            lines: self.lines,
            bytes: self.bytes,
            tokens: self.tokens,
            doc_pct: self.doc_pct,
            bytes_per_line: self.bytes_per_line,
            depth: self.depth,
        }
    }
}

pub(super) fn build_file_stats<'a>(rows: &[&'a FileRow]) -> Vec<FileStatView<'a>> {
    rows.iter()
        .map(|r| FileStatView {
            path: r.path.as_str(),
            module: r.module.as_str(),
            lang: r.lang.as_str(),
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

pub(super) fn build_max_file_report(rows: &[FileStatView<'_>]) -> MaxFileReport {
    let mut overall = rows
        .iter()
        .max_by(|a, b| a.lines.cmp(&b.lines).then_with(|| a.path.cmp(b.path)))
        .map(|v| v.into_row())
        .unwrap_or_else(empty_file_row);

    if rows.is_empty() {
        overall = empty_file_row();
    }

    let mut by_lang: std::collections::BTreeMap<&str, &FileStatView<'_>> =
        std::collections::BTreeMap::new();
    let mut by_module: std::collections::BTreeMap<&str, &FileStatView<'_>> =
        std::collections::BTreeMap::new();

    for row in rows {
        if let Some(existing) = by_lang.get_mut(row.lang) {
            if row.lines > existing.lines
                || (row.lines == existing.lines && row.path < existing.path)
            {
                *existing = row;
            }
        } else {
            by_lang.insert(row.lang, row);
        }

        if let Some(existing) = by_module.get_mut(row.module) {
            if row.lines > existing.lines
                || (row.lines == existing.lines && row.path < existing.path)
            {
                *existing = row;
            }
        } else {
            by_module.insert(row.module, row);
        }
    }

    MaxFileReport {
        overall,
        by_lang: by_lang
            .into_iter()
            .map(|(key, file)| MaxFileRow {
                key: key.to_string(),
                file: file.into_row(),
            })
            .collect(),
        by_module: by_module
            .into_iter()
            .map(|(key, file)| MaxFileRow {
                key: key.to_string(),
                file: file.into_row(),
            })
            .collect(),
    }
}

pub(super) fn build_top_offenders(rows: &[FileStatView<'_>]) -> TopOffenders {
    let mut by_lines: Vec<&FileStatView<'_>> = rows.iter().collect();
    by_lines.sort_by(|a, b| b.lines.cmp(&a.lines).then_with(|| a.path.cmp(b.path)));

    let mut by_tokens: Vec<&FileStatView<'_>> = rows.iter().collect();
    by_tokens.sort_by(|a, b| b.tokens.cmp(&a.tokens).then_with(|| a.path.cmp(b.path)));

    let mut by_bytes: Vec<&FileStatView<'_>> = rows.iter().collect();
    by_bytes.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.path.cmp(b.path)));

    let mut least_doc: Vec<&FileStatView<'_>> =
        rows.iter().filter(|r| r.lines >= MIN_DOC_LINES).collect();
    least_doc.sort_by(|a, b| {
        let a_doc = a.doc_pct.unwrap_or(0.0);
        let b_doc = b.doc_pct.unwrap_or(0.0);
        a_doc
            .partial_cmp(&b_doc)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.lines.cmp(&a.lines))
            .then_with(|| a.path.cmp(b.path))
    });

    let mut dense: Vec<&FileStatView<'_>> =
        rows.iter().filter(|r| r.lines >= MIN_DENSE_LINES).collect();
    dense.sort_by(|a, b| {
        let a_rate = a.bytes_per_line.unwrap_or(0.0);
        let b_rate = b.bytes_per_line.unwrap_or(0.0);
        b_rate
            .partial_cmp(&a_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(b.path))
    });

    TopOffenders {
        largest_lines: by_lines
            .into_iter()
            .take(TOP_N)
            .map(|v| v.into_row())
            .collect(),
        largest_tokens: by_tokens
            .into_iter()
            .take(TOP_N)
            .map(|v| v.into_row())
            .collect(),
        largest_bytes: by_bytes
            .into_iter()
            .take(TOP_N)
            .map(|v| v.into_row())
            .collect(),
        least_documented: least_doc
            .into_iter()
            .take(TOP_N)
            .map(|v| v.into_row())
            .collect(),
        most_dense: dense
            .into_iter()
            .take(TOP_N)
            .map(|v| v.into_row())
            .collect(),
    }
}
