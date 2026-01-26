use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use tokei::{LanguageType, Languages};
use tokmd_config::{ChildIncludeMode, ChildrenMode};
use tokmd_types::{
    ExportData, FileKind, FileRow, LangReport, LangRow, ModuleReport, ModuleRow, Totals,
};

pub fn create_lang_report(
    languages: &Languages,
    top: usize,
    with_files: bool,
    children: ChildrenMode,
) -> LangReport {
    let mut rows: Vec<LangRow> = Vec::new();

    match children {
        ChildrenMode::Collapse => {
            // Collapse embedded languages into the parent row by using tokei's
            // `Language::summarise()`.
            for (lang_type, lang) in languages.iter() {
                let sum = lang.summarise();
                if sum.code == 0 {
                    continue;
                }

                let lines = sum.code + sum.comments + sum.blanks;
                let files = lang.reports.len();
                let avg_lines = avg(lines, files);

                rows.push(LangRow {
                    lang: lang_type.name().to_string(),
                    code: sum.code,
                    lines,
                    files,
                    avg_lines,
                });
            }
        }
        ChildrenMode::Separate => {
            // Emit parent languages (raw) and also emit aggregated "(embedded)" rows
            // for child languages.
            #[derive(Default)]
            struct ChildAgg {
                code: usize,
                comments: usize,
                blanks: usize,
                files: usize,
            }

            let mut embedded: BTreeMap<LanguageType, ChildAgg> = BTreeMap::new();

            for (lang_type, lang) in languages.iter() {
                if lang.code > 0 {
                    let lines = lang.code + lang.comments + lang.blanks;
                    let files = lang.reports.len();
                    let avg_lines = avg(lines, files);

                    rows.push(LangRow {
                        lang: lang_type.name().to_string(),
                        code: lang.code,
                        lines,
                        files,
                        avg_lines,
                    });
                }

                for (child_type, reports) in &lang.children {
                    let entry = embedded.entry(*child_type).or_default();
                    entry.files += reports.len();
                    for r in reports {
                        let st = r.stats.summarise();
                        entry.code += st.code;
                        entry.comments += st.comments;
                        entry.blanks += st.blanks;
                    }
                }
            }

            for (child_type, agg) in embedded {
                if agg.code == 0 {
                    continue;
                }
                let lines = agg.code + agg.comments + agg.blanks;
                let avg_lines = avg(lines, agg.files);
                rows.push(LangRow {
                    lang: format!("{} (embedded)", child_type.name()),
                    code: agg.code,
                    lines,
                    files: agg.files,
                    avg_lines,
                });
            }
        }
    }

    // Sort descending by code, then by language name for determinism.
    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));

    // Compute totals *before* folding to top-N.
    let total_code: usize = rows.iter().map(|r| r.code).sum();
    let total_lines: usize = rows.iter().map(|r| r.lines).sum();
    let total_files = unique_parent_file_count(languages);
    let total = Totals {
        code: total_code,
        lines: total_lines,
        files: total_files,
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

    for r in rows {
        code += r.code;
        lines += r.lines;
        files += r.files;
    }

    LangRow {
        lang: "Other".to_string(),
        code,
        lines,
        files,
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
    }

    let mut by_module: BTreeMap<String, Agg> = BTreeMap::new();
    for r in &file_rows {
        let entry = by_module.entry(r.module.clone()).or_default();
        entry.code += r.code;
        entry.lines += r.lines;
    }

    // Unique parent files per module.
    let mut module_files: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (lang_type, lang) in languages.iter() {
        let _ = lang_type; // keep the pattern explicit; we only need reports
        for report in &lang.reports {
            let path = normalize_path(&report.name, None);
            let module = module_key(&path, module_roots, module_depth);
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
    let total = Totals {
        code: total_code,
        lines: total_lines,
        files: total_files,
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

    for r in rows {
        code += r.code;
        lines += r.lines;
        files += r.files;
    }

    ModuleRow {
        module: "Other".to_string(),
        code,
        lines,
        files,
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
    #[derive(Default, Clone, Copy)]
    struct Agg {
        code: usize,
        comments: usize,
        blanks: usize,
    }

    // Deterministic map: key ordering is stable.
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct Key {
        path: String,
        lang: String,
        kind: FileKind,
    }

    let mut map: BTreeMap<Key, (String /*module*/, Agg)> = BTreeMap::new();

    // Parent reports
    for (lang_type, lang) in languages.iter() {
        for report in &lang.reports {
            let path = normalize_path(&report.name, strip_prefix);
            let module = module_key(&path, module_roots, module_depth);
            let st = report.stats.summarise();

            let key = Key {
                path: path.clone(),
                lang: lang_type.name().to_string(),
                kind: FileKind::Parent,
            };
            let entry = map.entry(key).or_insert_with(|| (module, Agg::default()));
            entry.1.code += st.code;
            entry.1.comments += st.comments;
            entry.1.blanks += st.blanks;
        }
    }

    if children == ChildIncludeMode::Separate {
        for (_lang_type, lang) in languages.iter() {
            for (child_type, reports) in &lang.children {
                for report in reports {
                    let path = normalize_path(&report.name, strip_prefix);
                    let module = module_key(&path, module_roots, module_depth);
                    let st = report.stats.summarise();

                    let key = Key {
                        path: path.clone(),
                        lang: child_type.name().to_string(),
                        kind: FileKind::Child,
                    };
                    let entry = map.entry(key).or_insert_with(|| (module, Agg::default()));
                    entry.1.code += st.code;
                    entry.1.comments += st.comments;
                    entry.1.blanks += st.blanks;
                }
            }
        }
    }

    map.into_iter()
        .map(|(key, (module, agg))| {
            let lines = agg.code + agg.comments + agg.blanks;
            FileRow {
                path: key.path,
                module,
                lang: key.lang,
                kind: key.kind,
                code: agg.code,
                comments: agg.comments,
                blanks: agg.blanks,
                lines,
            }
        })
        .collect()
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
    let mut s = path.to_string_lossy().replace('\\', "/");

    // Strip leading ./ first, so strip_prefix can match against "src/" instead of "./src/"
    if let Some(stripped) = s.strip_prefix("./") {
        s = stripped.to_string();
    }

    if let Some(prefix) = strip_prefix {
        let mut pfx = prefix.to_string_lossy().replace('\\', "/");
        // Ensure prefix ends with a slash for exact segment matching.
        if !pfx.ends_with('/') {
            pfx.push('/');
        }
        if s.starts_with(&pfx) {
            s = s[pfx.len()..].to_string();
        }
    }

    s.trim_start_matches('/').to_string()
}

/// Compute a "module key" from a normalized path.
///
/// Rules:
/// - Root-level files become "(root)".
/// - If the first directory segment is in `module_roots`, join `module_depth` *directory* segments.
/// - Otherwise, module key is the top-level directory.
pub fn module_key(path: &str, module_roots: &[String], module_depth: usize) -> String {
    // Normalization here makes the function usable on both raw and already-normalized paths.
    let mut p = path.replace('\\', "/");
    if let Some(stripped) = p.strip_prefix("./") {
        p = stripped.to_string();
    }
    p = p.trim_start_matches('/').to_string();

    let parts: Vec<&str> = p.split('/').filter(|seg| !seg.is_empty()).collect();
    if parts.len() <= 1 {
        return "(root)".to_string();
    }

    // Directory segments only (exclude the filename).
    let dirs = &parts[..parts.len() - 1];
    if dirs.is_empty() {
        return "(root)".to_string();
    }

    let head = dirs[0];
    let is_root = module_roots.iter().any(|r| r == head);
    if is_root {
        let depth = module_depth.max(1).min(dirs.len());
        dirs[..depth].join("/")
    } else {
        head.to_string()
    }
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
}
