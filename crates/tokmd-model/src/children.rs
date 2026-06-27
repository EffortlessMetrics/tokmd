//! Child and embedded-language aggregation for model receipts.

use std::collections::{BTreeMap, BTreeSet};

use tokmd_types::{ChildrenMode, FileKind, FileRow, LangRow};

use crate::avg;

#[derive(Default)]
struct LangAgg {
    code: usize,
    lines: usize,
    bytes: usize,
    tokens: usize,
}

pub(crate) fn aggregate_lang_rows(file_rows: &[FileRow], children: ChildrenMode) -> Vec<LangRow> {
    let parent_lang_by_path: BTreeMap<&str, &str> = file_rows
        .iter()
        .filter(|row| row.kind == FileKind::Parent)
        .map(|row| (row.path.as_str(), row.lang.as_str()))
        .collect();
    let mut child_totals_by_path: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for row in file_rows.iter().filter(|row| row.kind == FileKind::Child) {
        let entry = child_totals_by_path.entry(row.path.as_str()).or_default();
        entry.0 += row.code;
        entry.1 += row.lines;
    }

    let mut by_lang: BTreeMap<(&str, bool), (LangAgg, BTreeSet<&str>)> = BTreeMap::new();

    for row in file_rows {
        match (children, row.kind) {
            (ChildrenMode::Collapse, FileKind::Parent) => {
                let entry = by_lang
                    .entry((row.lang.as_str(), false))
                    .or_insert_with(|| (LangAgg::default(), BTreeSet::new()));
                entry.0.code += row.code;
                entry.0.lines += row.lines;
                entry.0.bytes += row.bytes;
                entry.0.tokens += row.tokens;
                entry.1.insert(row.path.as_str());
            }
            (ChildrenMode::Collapse, FileKind::Child) => {
                if !parent_lang_by_path.contains_key(row.path.as_str()) {
                    let entry = by_lang
                        .entry((row.lang.as_str(), false))
                        .or_insert_with(|| (LangAgg::default(), BTreeSet::new()));
                    entry.0.code += row.code;
                    entry.0.lines += row.lines;
                    entry.0.bytes += row.bytes;
                    entry.0.tokens += row.tokens;
                    entry.1.insert(row.path.as_str());
                }
            }
            (ChildrenMode::Separate, FileKind::Parent) => {
                let (child_code, child_lines) = child_totals_by_path
                    .get(row.path.as_str())
                    .copied()
                    .unwrap_or((0, 0));

                let entry = by_lang
                    .entry((row.lang.as_str(), false))
                    .or_insert_with(|| (LangAgg::default(), BTreeSet::new()));
                entry.0.code += row.code.saturating_sub(child_code);
                entry.0.lines += row.lines.saturating_sub(child_lines);
                entry.0.bytes += row.bytes;
                entry.0.tokens += row.tokens;
                entry.1.insert(row.path.as_str());
            }
            (ChildrenMode::Separate, FileKind::Child) => {
                let entry = by_lang
                    .entry((row.lang.as_str(), true))
                    .or_insert_with(|| (LangAgg::default(), BTreeSet::new()));
                entry.0.code += row.code;
                entry.0.lines += row.lines;
                entry.1.insert(row.path.as_str());
            }
        }
    }

    by_lang
        .into_iter()
        .filter_map(|((lang, is_embedded), (agg, files_set))| {
            if agg.code == 0 {
                return None;
            }
            let files = files_set.len();
            Some(LangRow {
                lang: if is_embedded {
                    format!("{lang} (embedded)")
                } else {
                    lang.to_string()
                },
                code: agg.code,
                lines: agg.lines,
                files,
                bytes: agg.bytes,
                tokens: agg.tokens,
                avg_lines: avg(agg.lines, files),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(path: &str, lang: &str, kind: FileKind, code: usize, lines: usize) -> FileRow {
        FileRow {
            path: path.to_string(),
            module: "(root)".to_string(),
            lang: lang.to_string(),
            kind,
            code,
            comments: 0,
            blanks: lines.saturating_sub(code),
            lines,
            bytes: if kind == FileKind::Parent { 120 } else { 0 },
            tokens: if kind == FileKind::Parent { 30 } else { 0 },
        }
    }

    #[test]
    fn collapse_keeps_parent_row_and_skips_known_child_row() {
        let rows = [
            row("docs/mixed.md", "Markdown", FileKind::Parent, 10, 20),
            row("docs/mixed.md", "Rust", FileKind::Child, 4, 5),
        ];

        let aggregated = aggregate_lang_rows(&rows, ChildrenMode::Collapse);

        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].lang, "Markdown");
        assert_eq!(aggregated[0].code, 10);
        assert_eq!(aggregated[0].lines, 20);
        assert_eq!(aggregated[0].bytes, 120);
        assert_eq!(aggregated[0].tokens, 30);
    }

    #[test]
    fn separate_subtracts_child_lines_from_parent_and_labels_child() {
        let rows = [
            row("docs/mixed.md", "Markdown", FileKind::Parent, 10, 20),
            row("docs/mixed.md", "Rust", FileKind::Child, 4, 5),
        ];

        let aggregated = aggregate_lang_rows(&rows, ChildrenMode::Separate);
        let parent = aggregated
            .iter()
            .find(|row| row.lang == "Markdown")
            .expect("separate aggregation should keep parent row");
        let child = aggregated
            .iter()
            .find(|row| row.lang == "Rust (embedded)")
            .expect("separate aggregation should label child row");

        assert_eq!(parent.code, 6);
        assert_eq!(parent.lines, 15);
        assert_eq!(parent.bytes, 120);
        assert_eq!(parent.tokens, 30);
        assert_eq!(child.code, 4);
        assert_eq!(child.lines, 5);
        assert_eq!(child.bytes, 0);
        assert_eq!(child.tokens, 0);
    }

    #[test]
    fn separate_drops_parent_when_child_subsumes_all_code() {
        let rows = [
            row("docs/mixed.md", "Markdown", FileKind::Parent, 10, 20),
            row("docs/mixed.md", "Rust", FileKind::Child, 15, 25), // exceeds parent
        ];

        let aggregated = aggregate_lang_rows(&rows, ChildrenMode::Separate);

        // The parent code becomes 0 due to saturating_sub, so the parent is filtered out entirely.
        assert!(aggregated.iter().find(|r| r.lang == "Markdown").is_none());
    }

    #[test]
    fn collapse_preserves_orphan_child_without_parent() {
        let rows = [row("orphan.template", "Rust", FileKind::Child, 4, 5)];

        let aggregated = aggregate_lang_rows(&rows, ChildrenMode::Collapse);

        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].lang, "Rust");
        assert_eq!(aggregated[0].code, 4);
        assert_eq!(aggregated[0].lines, 5);
    }
}
