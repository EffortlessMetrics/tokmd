use tokmd_analysis_types::{FileStatRow, MaxFileReport, MaxFileRow, TopOffenders};
use tokmd_analysis_types::{empty_file_row, path_depth};
use tokmd_scan::safe_ratio;
use tokmd_types::FileRow;

const TOP_N: usize = 10;
const MIN_DOC_LINES: usize = 50;
const MIN_DENSE_LINES: usize = 10;

// Internal struct avoiding String allocations for every file
pub(super) struct FileStatRef<'a> {
    pub row: &'a FileRow,
    pub doc_pct: Option<f64>,
    pub bytes_per_line: Option<f64>,
    pub depth: usize,
}

impl<'a> FileStatRef<'a> {
    pub fn to_owned(&self) -> FileStatRow {
        FileStatRow {
            path: self.row.path.clone(),
            module: self.row.module.clone(),
            lang: self.row.lang.clone(),
            code: self.row.code,
            comments: self.row.comments,
            blanks: self.row.blanks,
            lines: self.row.lines,
            bytes: self.row.bytes,
            tokens: self.row.tokens,
            doc_pct: self.doc_pct,
            bytes_per_line: self.bytes_per_line,
            depth: self.depth,
        }
    }
}

pub(super) fn build_file_stat_refs<'a>(rows: &'a [&'a FileRow]) -> Vec<FileStatRef<'a>> {
    rows.iter()
        .map(|r| FileStatRef {
            row: r,
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

pub(super) fn build_max_file_report(refs: &[FileStatRef]) -> MaxFileReport {
    let overall_ref = refs.iter().max_by(|a, b| {
        a.row
            .lines
            .cmp(&b.row.lines)
            .then_with(|| a.row.path.cmp(&b.row.path))
    });

    let overall = match overall_ref {
        Some(r) => r.to_owned(),
        None => empty_file_row(),
    };

    let mut by_lang: std::collections::BTreeMap<&str, &FileStatRef> =
        std::collections::BTreeMap::new();
    let mut by_module: std::collections::BTreeMap<&str, &FileStatRef> =
        std::collections::BTreeMap::new();

    for r in refs {
        if let Some(existing) = by_lang.get_mut(r.row.lang.as_str()) {
            if r.row.lines > existing.row.lines
                || (r.row.lines == existing.row.lines && r.row.path < existing.row.path)
            {
                *existing = r;
            }
        } else {
            by_lang.insert(r.row.lang.as_str(), r);
        }

        if let Some(existing) = by_module.get_mut(r.row.module.as_str()) {
            if r.row.lines > existing.row.lines
                || (r.row.lines == existing.row.lines && r.row.path < existing.row.path)
            {
                *existing = r;
            }
        } else {
            by_module.insert(r.row.module.as_str(), r);
        }
    }

    MaxFileReport {
        overall,
        by_lang: by_lang
            .into_iter()
            .map(|(key, r)| MaxFileRow {
                key: key.to_string(),
                file: r.to_owned(),
            })
            .collect(),
        by_module: by_module
            .into_iter()
            .map(|(key, r)| MaxFileRow {
                key: key.to_string(),
                file: r.to_owned(),
            })
            .collect(),
    }
}

pub(super) fn build_top_offenders(refs: &[FileStatRef]) -> TopOffenders {
    let mut by_lines: Vec<&FileStatRef> = refs.iter().collect();
    by_lines.sort_by(|a, b| {
        b.row
            .lines
            .cmp(&a.row.lines)
            .then_with(|| a.row.path.cmp(&b.row.path))
    });

    let mut by_tokens: Vec<&FileStatRef> = refs.iter().collect();
    by_tokens.sort_by(|a, b| {
        b.row
            .tokens
            .cmp(&a.row.tokens)
            .then_with(|| a.row.path.cmp(&b.row.path))
    });

    let mut by_bytes: Vec<&FileStatRef> = refs.iter().collect();
    by_bytes.sort_by(|a, b| {
        b.row
            .bytes
            .cmp(&a.row.bytes)
            .then_with(|| a.row.path.cmp(&b.row.path))
    });

    let mut least_doc: Vec<&FileStatRef> = refs
        .iter()
        .filter(|r| r.row.lines >= MIN_DOC_LINES)
        .collect();
    least_doc.sort_by(|a, b| {
        let a_doc = a.doc_pct.unwrap_or(0.0);
        let b_doc = b.doc_pct.unwrap_or(0.0);
        a_doc
            .partial_cmp(&b_doc)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.row.lines.cmp(&a.row.lines))
            .then_with(|| a.row.path.cmp(&b.row.path))
    });

    let mut dense: Vec<&FileStatRef> = refs
        .iter()
        .filter(|r| r.row.lines >= MIN_DENSE_LINES)
        .collect();
    dense.sort_by(|a, b| {
        let a_rate = a.bytes_per_line.unwrap_or(0.0);
        let b_rate = b.bytes_per_line.unwrap_or(0.0);
        b_rate
            .partial_cmp(&a_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.row.path.cmp(&b.row.path))
    });

    TopOffenders {
        largest_lines: by_lines
            .into_iter()
            .take(TOP_N)
            .map(|r| r.to_owned())
            .collect(),
        largest_tokens: by_tokens
            .into_iter()
            .take(TOP_N)
            .map(|r| r.to_owned())
            .collect(),
        largest_bytes: by_bytes
            .into_iter()
            .take(TOP_N)
            .map(|r| r.to_owned())
            .collect(),
        least_documented: least_doc
            .into_iter()
            .take(TOP_N)
            .map(|r| r.to_owned())
            .collect(),
        most_dense: dense
            .into_iter()
            .take(TOP_N)
            .map(|r| r.to_owned())
            .collect(),
    }
}
