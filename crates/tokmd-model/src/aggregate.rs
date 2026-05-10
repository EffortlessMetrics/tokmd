//! Report aggregation builders for model receipts.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use tokei::Languages;
use tokmd_types::{
    ChildIncludeMode, ChildrenMode, ExportData, FileKind, FileRow, LangReport, LangRow,
    ModuleReport, ModuleRow, Totals,
};

use crate::sorting::{sort_file_rows, sort_lang_rows, sort_module_rows};
use crate::{avg, collect_file_rows, unique_parent_file_count_from_rows};

pub fn create_lang_report(
    languages: &Languages,
    top: usize,
    with_files: bool,
    children: ChildrenMode,
) -> LangReport {
    let rows = collect_file_rows(languages, &[], 1, ChildIncludeMode::Separate, None);
    create_lang_report_from_rows(&rows, top, with_files, children)
}

pub fn create_lang_report_from_rows(
    file_rows: &[FileRow],
    top: usize,
    with_files: bool,
    children: ChildrenMode,
) -> LangReport {
    #[derive(Default)]
    struct LangAgg {
        code: usize,
        lines: usize,
        bytes: usize,
        tokens: usize,
    }

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

    let mut rows: Vec<LangRow> = Vec::with_capacity(by_lang.len());
    for ((lang, is_embedded), (agg, files_set)) in by_lang {
        if agg.code == 0 {
            continue;
        }
        let files = files_set.len();
        rows.push(LangRow {
            lang: if is_embedded {
                format!("{} (embedded)", lang)
            } else {
                lang.to_string()
            },
            code: agg.code,
            lines: agg.lines,
            files,
            bytes: agg.bytes,
            tokens: agg.tokens,
            avg_lines: avg(agg.lines, files),
        });
    }

    sort_lang_rows(&mut rows);

    let total_code: usize = rows.iter().map(|r| r.code).sum();
    let total_lines: usize = rows.iter().map(|r| r.lines).sum();
    let total_bytes: usize = rows.iter().map(|r| r.bytes).sum();
    let total_tokens: usize = rows.iter().map(|r| r.tokens).sum();
    let total_files = unique_parent_file_count_from_rows(file_rows);

    let total = Totals {
        code: total_code,
        lines: total_lines,
        files: total_files,
        bytes: total_bytes,
        tokens: total_tokens,
        avg_lines: avg(total_lines, total_files),
    };

    if top > 0 && rows.len() > top {
        let other = fold_other_lang(&rows[top..]);
        rows.truncate(top);
        rows.push(other);
    }

    LangReport {
        rows,
        total,
        with_files,
        children,
        top,
    }
}

fn fold_other_lang(rows: &[LangRow]) -> LangRow {
    let mut code = 0usize;
    let mut lines = 0usize;
    let mut files = 0usize;
    let mut bytes = 0usize;
    let mut tokens = 0usize;

    for r in rows {
        code += r.code;
        lines += r.lines;
        files += r.files;
        bytes += r.bytes;
        tokens += r.tokens;
    }

    LangRow {
        lang: "Other".to_string(),
        code,
        lines,
        files,
        bytes,
        tokens,
        avg_lines: avg(lines, files),
    }
}

pub fn create_module_report(
    languages: &Languages,
    module_roots: &[String],
    module_depth: usize,
    children: ChildIncludeMode,
    top: usize,
) -> ModuleReport {
    let file_rows = collect_file_rows(languages, module_roots, module_depth, children, None);
    create_module_report_from_rows(&file_rows, module_roots, module_depth, children, top)
}

pub fn create_module_report_from_rows(
    file_rows: &[FileRow],
    module_roots: &[String],
    module_depth: usize,
    children: ChildIncludeMode,
    top: usize,
) -> ModuleReport {
    #[derive(Default)]
    struct Agg {
        code: usize,
        lines: usize,
        bytes: usize,
        tokens: usize,
    }

    let mut by_module: BTreeMap<&str, (Agg, BTreeSet<&str>)> = BTreeMap::new();
    let mut total_code = 0;
    let mut total_lines = 0;
    let mut total_bytes = 0;
    let mut total_tokens = 0;

    for r in file_rows {
        total_code += r.code;
        total_lines += r.lines;
        total_bytes += r.bytes;
        total_tokens += r.tokens;

        let entry = by_module
            .entry(r.module.as_str())
            .or_insert_with(|| (Agg::default(), BTreeSet::new()));
        entry.0.code += r.code;
        entry.0.lines += r.lines;
        entry.0.bytes += r.bytes;
        entry.0.tokens += r.tokens;

        if r.kind == FileKind::Parent {
            entry.1.insert(r.path.as_str());
        }
    }

    let mut rows: Vec<ModuleRow> = Vec::with_capacity(by_module.len());
    for (module, (agg, files_set)) in by_module {
        let files = files_set.len();
        rows.push(ModuleRow {
            module: module.to_string(),
            code: agg.code,
            lines: agg.lines,
            files,
            bytes: agg.bytes,
            tokens: agg.tokens,
            avg_lines: avg(agg.lines, files),
        });
    }

    sort_module_rows(&mut rows);

    if top > 0 && rows.len() > top {
        let other = fold_other_module(&rows[top..]);
        rows.truncate(top);
        rows.push(other);
    }

    let total_files = unique_parent_file_count_from_rows(file_rows);

    let total = Totals {
        code: total_code,
        lines: total_lines,
        files: total_files,
        bytes: total_bytes,
        tokens: total_tokens,
        avg_lines: avg(total_lines, total_files),
    };

    ModuleReport {
        rows,
        total,
        module_roots: module_roots.to_vec(),
        module_depth,
        children,
        top,
    }
}

fn fold_other_module(rows: &[ModuleRow]) -> ModuleRow {
    let mut code = 0usize;
    let mut lines = 0usize;
    let mut files = 0usize;
    let mut bytes = 0usize;
    let mut tokens = 0usize;

    for r in rows {
        code += r.code;
        lines += r.lines;
        files += r.files;
        bytes += r.bytes;
        tokens += r.tokens;
    }

    ModuleRow {
        module: "Other".to_string(),
        code,
        lines,
        files,
        bytes,
        tokens,
        avg_lines: avg(lines, files),
    }
}

pub fn create_export_data(
    languages: &Languages,
    module_roots: &[String],
    module_depth: usize,
    children: ChildIncludeMode,
    strip_prefix: Option<&Path>,
    min_code: usize,
    max_rows: usize,
) -> ExportData {
    let rows = collect_file_rows(
        languages,
        module_roots,
        module_depth,
        children,
        strip_prefix,
    );
    create_export_data_from_rows(
        rows,
        module_roots,
        module_depth,
        children,
        min_code,
        max_rows,
    )
}

pub fn create_export_data_from_rows(
    mut rows: Vec<FileRow>,
    module_roots: &[String],
    module_depth: usize,
    children: ChildIncludeMode,
    min_code: usize,
    max_rows: usize,
) -> ExportData {
    if min_code > 0 {
        rows.retain(|r| r.code >= min_code);
    }
    sort_file_rows(&mut rows);

    if max_rows > 0 && rows.len() > max_rows {
        rows.truncate(max_rows);
    }

    ExportData {
        rows,
        module_roots: module_roots.to_vec(),
        module_depth,
        children,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_lang_row() -> impl Strategy<Value = LangRow> {
        (
            "[a-zA-Z]+",
            0usize..10000,
            0usize..20000,
            0usize..1000,
            0usize..1000000,
            0usize..100000,
        )
            .prop_map(|(lang, code, lines, files, bytes, tokens)| {
                let avg_lines = (lines + (files / 2)).checked_div(files).unwrap_or(0);
                LangRow {
                    lang,
                    code,
                    lines,
                    files,
                    bytes,
                    tokens,
                    avg_lines,
                }
            })
    }

    fn arb_module_row() -> impl Strategy<Value = ModuleRow> {
        (
            "[a-zA-Z0-9_/]+",
            0usize..10000,
            0usize..20000,
            0usize..1000,
            0usize..1000000,
            0usize..100000,
        )
            .prop_map(|(module, code, lines, files, bytes, tokens)| {
                let avg_lines = (lines + (files / 2)).checked_div(files).unwrap_or(0);
                ModuleRow {
                    module,
                    code,
                    lines,
                    files,
                    bytes,
                    tokens,
                    avg_lines,
                }
            })
    }

    proptest! {
        #[test]
        fn fold_lang_preserves_totals(rows in prop::collection::vec(arb_lang_row(), 0..10)) {
            let folded = fold_other_lang(&rows);

            let total_code: usize = rows.iter().map(|r| r.code).sum();
            let total_lines: usize = rows.iter().map(|r| r.lines).sum();
            let total_files: usize = rows.iter().map(|r| r.files).sum();
            let total_bytes: usize = rows.iter().map(|r| r.bytes).sum();
            let total_tokens: usize = rows.iter().map(|r| r.tokens).sum();

            prop_assert_eq!(folded.code, total_code, "Code mismatch");
            prop_assert_eq!(folded.lines, total_lines, "Lines mismatch");
            prop_assert_eq!(folded.files, total_files, "Files mismatch");
            prop_assert_eq!(folded.bytes, total_bytes, "Bytes mismatch");
            prop_assert_eq!(folded.tokens, total_tokens, "Tokens mismatch");
        }

        #[test]
        fn fold_lang_empty_is_zero(_dummy in 0..1u8) {
            let folded = fold_other_lang(&[]);
            prop_assert_eq!(folded.code, 0);
            prop_assert_eq!(folded.lines, 0);
            prop_assert_eq!(folded.files, 0);
            prop_assert_eq!(folded.bytes, 0);
            prop_assert_eq!(folded.tokens, 0);
            prop_assert_eq!(folded.lang, "Other");
        }

        #[test]
        fn fold_module_preserves_totals(rows in prop::collection::vec(arb_module_row(), 0..10)) {
            let folded = fold_other_module(&rows);

            let total_code: usize = rows.iter().map(|r| r.code).sum();
            let total_lines: usize = rows.iter().map(|r| r.lines).sum();
            let total_files: usize = rows.iter().map(|r| r.files).sum();
            let total_bytes: usize = rows.iter().map(|r| r.bytes).sum();
            let total_tokens: usize = rows.iter().map(|r| r.tokens).sum();

            prop_assert_eq!(folded.code, total_code, "Code mismatch");
            prop_assert_eq!(folded.lines, total_lines, "Lines mismatch");
            prop_assert_eq!(folded.files, total_files, "Files mismatch");
            prop_assert_eq!(folded.bytes, total_bytes, "Bytes mismatch");
            prop_assert_eq!(folded.tokens, total_tokens, "Tokens mismatch");
        }

        #[test]
        fn fold_module_empty_is_zero(_dummy in 0..1u8) {
            let folded = fold_other_module(&[]);
            prop_assert_eq!(folded.code, 0);
            prop_assert_eq!(folded.lines, 0);
            prop_assert_eq!(folded.files, 0);
            prop_assert_eq!(folded.bytes, 0);
            prop_assert_eq!(folded.tokens, 0);
            prop_assert_eq!(folded.module, "Other");
        }

        #[test]
        fn fold_associative_lang(
            rows1 in prop::collection::vec(arb_lang_row(), 0..5),
            rows2 in prop::collection::vec(arb_lang_row(), 0..5)
        ) {
            let all: Vec<_> = rows1.iter().chain(rows2.iter()).cloned().collect();
            let fold_all = fold_other_lang(&all);

            let fold1 = fold_other_lang(&rows1);
            let fold2 = fold_other_lang(&rows2);
            let combined = fold_other_lang(&[fold1, fold2]);

            prop_assert_eq!(fold_all.code, combined.code);
            prop_assert_eq!(fold_all.lines, combined.lines);
            prop_assert_eq!(fold_all.files, combined.files);
            prop_assert_eq!(fold_all.bytes, combined.bytes);
            prop_assert_eq!(fold_all.tokens, combined.tokens);
        }
    }
}
