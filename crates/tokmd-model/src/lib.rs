//! # tokmd-model
//!
//! **Tier 1 (Logic)**
//!
//! This crate contains the core business logic for aggregating and transforming code statistics.
//! It handles the conversion from raw Tokei scan results into `tokmd` receipts.
//!
//! ## What belongs here
//! * Aggregation logic (rolling up stats to modules/languages)
//! * Deterministic sorting and filtering
//! * Path normalization rules
//! * Receipt generation logic
//!
//! ## What does NOT belong here
//! * CLI argument parsing
//! * Output formatting (printing to stdout/file)
//! * Tokei interaction (use tokmd-scan)

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use tokei::{LanguageType, Languages};
use tokmd_module_key::module_key_from_normalized;
use tokmd_types::{
    ChildIncludeMode, ChildrenMode, ExportData, FileKind, FileRow, LangReport, LangRow,
    ModuleReport, ModuleRow, Totals,
};

/// Simple heuristic: 1 token ~= 4 chars (bytes).
const CHARS_PER_TOKEN: usize = 4;

fn get_file_metrics(path: &Path) -> (usize, usize) {
    // Best-effort size calculation.
    // If the file was deleted or is inaccessible during the scan post-processing,
    // we return 0 bytes/tokens rather than crashing.
    let bytes = fs::metadata(path).map(|m| m.len() as usize).unwrap_or(0);
    let tokens = bytes / CHARS_PER_TOKEN;
    (bytes, tokens)
}

pub fn create_lang_report(
    languages: &Languages,
    top: usize,
    with_files: bool,
    children: ChildrenMode,
) -> LangReport {
    // Aggregate metrics per language.
    // Since we need to access the filesystem for bytes, we do it via collect_file_rows first?
    // Or just iterate and compute. Since collect_file_rows is for Module/Export, we can't reuse it easily
    // for Lang report without re-grouping.
    // However, Lang report also needs to be accurate.
    // To avoid double-counting bytes for embedded languages, we should only count bytes for PARENT languages.

    // Let's iterate languages and files similar to collect_file_rows but grouping by Lang.

    // We can't use collect_file_rows directly because it flattens everything.
    // But we CAN use the same helper logic.

    let mut rows: Vec<LangRow> = Vec::new();

    // Helper map to store aggregated stats including bytes
    #[derive(Default)]
    struct LangAgg {
        code: usize,
        lines: usize,
        files: usize,
    }

    match children {
        ChildrenMode::Collapse => {
            // Collapse embedded languages into the parent row.
            // Bytes are attributed to the parent file's language.

            for (lang_type, lang) in languages.iter() {
                let sum = lang.summarise();
                if sum.code == 0 {
                    continue;
                }

                // Compute bytes sum for all files in this language
                let mut bytes_sum = 0;
                let mut tokens_sum = 0;
                for report in &lang.reports {
                    let (b, t) = get_file_metrics(&report.name);
                    bytes_sum += b;
                    tokens_sum += t;
                }

                let lines = sum.code + sum.comments + sum.blanks;
                let files = lang.reports.len();
                let avg_lines = avg(lines, files);

                rows.push(LangRow {
                    lang: lang_type.name().to_string(),
                    code: sum.code,
                    lines,
                    files,
                    bytes: bytes_sum,
                    tokens: tokens_sum,
                    avg_lines,
                });
            }
        }
        ChildrenMode::Separate => {
            // Separate embedded languages.
            // Bytes/Tokens should only be counted for the PARENT file.
            // Embedded segments (children) have 0 bytes/tokens effectively to avoid double counting.

            let mut embedded: BTreeMap<LanguageType, LangAgg> = BTreeMap::new();

            for (lang_type, lang) in languages.iter() {
                if lang.code > 0 {
                    let lines = lang.code + lang.comments + lang.blanks;
                    let files = lang.reports.len();

                    // Parent files get the bytes
                    let mut bytes_sum = 0;
                    let mut tokens_sum = 0;
                    for report in &lang.reports {
                        let (b, t) = get_file_metrics(&report.name);
                        bytes_sum += b;
                        tokens_sum += t;
                    }

                    rows.push(LangRow {
                        lang: lang_type.name().to_string(),
                        code: lang.code,
                        lines,
                        files,
                        bytes: bytes_sum,
                        tokens: tokens_sum,
                        avg_lines: avg(lines, files),
                    });
                }

                for (child_type, reports) in &lang.children {
                    let entry = embedded.entry(*child_type).or_default();
                    entry.files += reports.len();
                    for r in reports {
                        let st = r.stats.summarise();
                        entry.code += st.code;
                        entry.lines += st.code + st.comments + st.blanks;
                        // Embedded languages don't own the file, so 0 bytes/tokens
                    }
                }
            }

            for (child_type, agg) in embedded {
                if agg.code == 0 {
                    continue;
                }
                let avg_lines = avg(agg.lines, agg.files);
                rows.push(LangRow {
                    lang: format!("{} (embedded)", child_type.name()),
                    code: agg.code,
                    lines: agg.lines,
                    files: agg.files,
                    bytes: 0,  // No bytes for embedded
                    tokens: 0, // No tokens for embedded
                    avg_lines,
                });
            }
        }
    }

    // Sort descending by code, then by language name for determinism.
    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));

    // Compute totals
    let total_code: usize = rows.iter().map(|r| r.code).sum();
    let total_lines: usize = rows.iter().map(|r| r.lines).sum();
    let total_bytes: usize = rows.iter().map(|r| r.bytes).sum();
    let total_tokens: usize = rows.iter().map(|r| r.tokens).sum();
    let total_files = unique_parent_file_count(languages);

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
    // Aggregate stats per module, but count files uniquely (parent files only).
    let file_rows = collect_file_rows(languages, module_roots, module_depth, children, None);

    #[derive(Default)]
    struct Agg {
        code: usize,
        lines: usize,
        bytes: usize,
        tokens: usize,
    }

    let mut by_module: BTreeMap<String, Agg> = BTreeMap::new();
    for r in &file_rows {
        let entry = by_module.entry(r.module.clone()).or_default();
        entry.code += r.code;
        entry.lines += r.lines;
        entry.bytes += r.bytes;
        entry.tokens += r.tokens;
    }

    // Unique parent files per module.
    let mut module_files: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (lang_type, lang) in languages.iter() {
        let _ = lang_type; // keep the pattern explicit; we only need reports
        for report in &lang.reports {
            let path = normalize_path(&report.name, None);
            let module = module_key_from_normalized(&path, module_roots, module_depth);
            module_files.entry(module).or_default().insert(path);
        }
    }

    let mut rows: Vec<ModuleRow> = Vec::new();
    for (module, agg) in by_module {
        let files = module_files.get(&module).map(|s| s.len()).unwrap_or(0);
        rows.push(ModuleRow {
            module,
            code: agg.code,
            lines: agg.lines,
            files,
            bytes: agg.bytes,
            tokens: agg.tokens,
            avg_lines: avg(agg.lines, files),
        });
    }

    // Sort descending by code, then by module name for determinism.
    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));

    if top > 0 && rows.len() > top {
        let other = fold_other_module(&rows[top..]);
        rows.truncate(top);
        rows.push(other);
    }

    let total_files = unique_parent_file_count(languages);
    let total_code: usize = file_rows.iter().map(|r| r.code).sum();
    let total_lines: usize = file_rows.iter().map(|r| r.lines).sum();
    let total_bytes: usize = file_rows.iter().map(|r| r.bytes).sum();
    let total_tokens: usize = file_rows.iter().map(|r| r.tokens).sum();

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
    let mut rows = collect_file_rows(
        languages,
        module_roots,
        module_depth,
        children,
        strip_prefix,
    );

    // Filter and sort for determinism.
    if min_code > 0 {
        rows.retain(|r| r.code >= min_code);
    }
    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.path.cmp(&b.path)));

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

/// Collect per-file contributions, optionally including embedded language reports.
///
/// This returns one row per (path, lang, kind), aggregated if tokei produced multiple
/// reports for the same tuple.
pub fn collect_file_rows(
    languages: &Languages,
    module_roots: &[String],
    module_depth: usize,
    children: ChildIncludeMode,
    strip_prefix: Option<&Path>,
) -> Vec<FileRow> {
    // Intermediate struct to hold data before aggregation
    struct IntermediateRow {
        path: String,
        module: String,
        lang: String,
        kind: FileKind,
        code: usize,
        comments: usize,
        blanks: usize,
        bytes: usize,
        tokens: usize,
    }

    let mut rows: Vec<IntermediateRow> = Vec::new();

    // Parent reports
    for (lang_type, lang) in languages.iter() {
        for report in &lang.reports {
            let path = normalize_path(&report.name, strip_prefix);
            let module = module_key_from_normalized(&path, module_roots, module_depth);
            let st = report.stats.summarise();
            let (bytes, tokens) = get_file_metrics(&report.name);

            rows.push(IntermediateRow {
                path, // Move string (no clone)
                module,
                lang: lang_type.name().to_string(),
                kind: FileKind::Parent,
                code: st.code,
                comments: st.comments,
                blanks: st.blanks,
                bytes,
                tokens,
            });
        }
    }

    if children == ChildIncludeMode::Separate {
        for (_lang_type, lang) in languages.iter() {
            for (child_type, reports) in &lang.children {
                for report in reports {
                    let path = normalize_path(&report.name, strip_prefix);
                    let module = module_key_from_normalized(&path, module_roots, module_depth);
                    let st = report.stats.summarise();
                    // Embedded children do not have bytes/tokens (they are inside the parent)

                    rows.push(IntermediateRow {
                        path, // Move string (no clone)
                        module,
                        lang: child_type.name().to_string(),
                        kind: FileKind::Child,
                        code: st.code,
                        comments: st.comments,
                        blanks: st.blanks,
                        bytes: 0,
                        tokens: 0,
                    });
                }
            }
        }
    }

    // Sort by (path, lang, kind) to group identical items for aggregation
    rows.sort_unstable_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then_with(|| a.lang.cmp(&b.lang))
            .then_with(|| a.kind.cmp(&b.kind))
    });

    if rows.is_empty() {
        return Vec::new();
    }

    // Coalesce adjacent identical rows
    let mut result = Vec::with_capacity(rows.len());
    let mut current = rows.into_iter();

    if let Some(mut acc) = current.next() {
        for row in current {
            if acc.path == row.path && acc.lang == row.lang && acc.kind == row.kind {
                // Aggregate
                acc.code += row.code;
                acc.comments += row.comments;
                acc.blanks += row.blanks;
                acc.bytes += row.bytes;
                acc.tokens += row.tokens;
            } else {
                // Emit accumulator
                let lines = acc.code + acc.comments + acc.blanks;
                result.push(FileRow {
                    path: acc.path,
                    module: acc.module,
                    lang: acc.lang,
                    kind: acc.kind,
                    code: acc.code,
                    comments: acc.comments,
                    blanks: acc.blanks,
                    lines,
                    bytes: acc.bytes,
                    tokens: acc.tokens,
                });
                acc = row;
            }
        }
        // Emit final accumulator
        let lines = acc.code + acc.comments + acc.blanks;
        result.push(FileRow {
            path: acc.path,
            module: acc.module,
            lang: acc.lang,
            kind: acc.kind,
            code: acc.code,
            comments: acc.comments,
            blanks: acc.blanks,
            lines,
            bytes: acc.bytes,
            tokens: acc.tokens,
        });
    }

    result
}

pub fn unique_parent_file_count(languages: &Languages) -> usize {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for (_lang_type, lang) in languages.iter() {
        for report in &lang.reports {
            let path = normalize_path(&report.name, None);
            seen.insert(path);
        }
    }
    seen.len()
}

pub fn avg(lines: usize, files: usize) -> usize {
    if files == 0 {
        return 0;
    }
    // Round to nearest integer.
    (lines + (files / 2)) / files
}

/// Normalize a path for portable output.
///
/// - Uses `/` separators
/// - Strips leading `./`
/// - Optionally strips a user-provided prefix (after normalization)
pub fn normalize_path(path: &Path, strip_prefix: Option<&Path>) -> String {
    let s_cow = path.to_string_lossy();
    let s: Cow<str> = if s_cow.contains('\\') {
        Cow::Owned(s_cow.replace('\\', "/"))
    } else {
        s_cow
    };

    let mut slice: &str = &s;

    // Strip leading ./ first, so strip_prefix can match against "src/" instead of "./src/"
    if let Some(stripped) = slice.strip_prefix("./") {
        slice = stripped;
    }

    if let Some(prefix) = strip_prefix {
        let p_cow = prefix.to_string_lossy();
        // Strip leading ./ from prefix so it can match normalized paths
        let p_cow_stripped: Cow<str> = if let Some(stripped) = p_cow.strip_prefix("./") {
            Cow::Borrowed(stripped)
        } else {
            p_cow
        };

        let needs_replace = p_cow_stripped.contains('\\');
        let needs_slash = !p_cow_stripped.ends_with('/');

        if !needs_replace && !needs_slash {
            // Fast path: prefix is already clean and ends with slash
            if slice.starts_with(p_cow_stripped.as_ref()) {
                slice = &slice[p_cow_stripped.len()..];
            }
        } else {
            // Slow path: normalize prefix
            let mut pfx = if needs_replace {
                p_cow_stripped.replace('\\', "/")
            } else {
                p_cow_stripped.into_owned()
            };
            if needs_slash {
                pfx.push('/');
            }
            if slice.starts_with(&pfx) {
                slice = &slice[pfx.len()..];
            }
        }
    }

    slice = slice.trim_start_matches('/');

    // After trimming slashes, we might be left with a leading ./ (e.g. from "/./")
    if let Some(stripped) = slice.strip_prefix("./") {
        slice = stripped;
    }

    if slice.len() == s.len() {
        s.into_owned()
    } else {
        slice.to_string()
    }
}

/// Compute a "module key" from an input path.
///
/// Rules:
/// - Root-level files become "(root)".
/// - If the first directory segment is in `module_roots`, join `module_depth` *directory* segments.
/// - Otherwise, module key is the top-level directory.
pub fn module_key(path: &str, module_roots: &[String], module_depth: usize) -> String {
    tokmd_module_key::module_key(path, module_roots, module_depth)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn module_key_root_level_file() {
        assert_eq!(module_key("Cargo.toml", &["crates".into()], 2), "(root)");
        assert_eq!(module_key("./Cargo.toml", &["crates".into()], 2), "(root)");
    }

    #[test]
    fn module_key_crates_depth_2() {
        let roots = vec!["crates".into(), "packages".into()];
        assert_eq!(module_key("crates/foo/src/lib.rs", &roots, 2), "crates/foo");
        assert_eq!(
            module_key("packages/bar/src/main.rs", &roots, 2),
            "packages/bar"
        );
    }

    #[test]
    fn module_key_crates_depth_1() {
        let roots = vec!["crates".into(), "packages".into()];
        assert_eq!(module_key("crates/foo/src/lib.rs", &roots, 1), "crates");
    }

    #[test]
    fn module_key_non_root() {
        let roots = vec!["crates".into()];
        assert_eq!(module_key("src/lib.rs", &roots, 2), "src");
        assert_eq!(module_key("tools/gen.rs", &roots, 2), "tools");
    }

    #[test]
    fn module_key_depth_overflow_does_not_include_filename() {
        let roots = vec!["crates".into()];
        // File directly under a root: depth=2 should NOT include the filename
        assert_eq!(module_key("crates/foo.rs", &roots, 2), "crates");
        // Depth exceeds available directories: should stop at deepest directory
        assert_eq!(
            module_key("crates/foo/src/lib.rs", &roots, 10),
            "crates/foo/src"
        );
    }

    #[test]
    fn normalize_path_strips_prefix() {
        let p = PathBuf::from("C:/Code/Repo/src/main.rs");
        let prefix = PathBuf::from("C:/Code/Repo");
        let got = normalize_path(&p, Some(&prefix));
        assert_eq!(got, "src/main.rs");
    }

    #[test]
    fn normalize_path_normalization_slashes() {
        let p = PathBuf::from(r"C:\Code\Repo\src\main.rs");
        let got = normalize_path(&p, None);
        assert_eq!(got, "C:/Code/Repo/src/main.rs");
    }

    // Property-based tests for fold_other_* functions
    mod fold_properties {
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
                    let avg_lines = if files == 0 {
                        0
                    } else {
                        (lines + (files / 2)) / files
                    };
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
                    let avg_lines = if files == 0 {
                        0
                    } else {
                        (lines + (files / 2)) / files
                    };
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
                // Folding all at once should equal folding parts and combining
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
}
