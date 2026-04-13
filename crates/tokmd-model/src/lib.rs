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
use std::path::{Path, PathBuf};

use tokei::{CodeStats, Config, LanguageType, Languages};
use tokmd_module_key::module_key_from_normalized;
use tokmd_types::{
    ChildIncludeMode, ChildrenMode, ExportData, FileKind, FileRow, LangReport, LangRow,
    ModuleReport, ModuleRow, Totals,
};

/// Simple heuristic: 1 token ~= 4 chars (bytes).
const CHARS_PER_TOKEN: usize = 4;

#[derive(Default, Clone, Copy)]
struct Agg {
    code: usize,
    comments: usize,
    blanks: usize,
    bytes: usize,
    tokens: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key<'a> {
    path: String,
    lang: &'a str,
    kind: FileKind,
}

/// A logical in-memory file used to synthesize `FileRow`s without the host filesystem.
pub struct InMemoryRowInput<'a> {
    pub logical_path: &'a Path,
    pub bytes: &'a [u8],
}

impl<'a> InMemoryRowInput<'a> {
    #[must_use]
    pub fn new(logical_path: &'a Path, bytes: &'a [u8]) -> Self {
        Self {
            logical_path,
            bytes,
        }
    }
}

fn get_file_metrics(path: &Path) -> (usize, usize) {
    // Best-effort size calculation.
    // If the file was deleted or is inaccessible during the scan post-processing,
    // we return 0 bytes/tokens rather than crashing.
    let bytes = fs::metadata(path).map(|m| m.len() as usize).unwrap_or(0);
    metrics_from_byte_len(bytes)
}

fn metrics_from_bytes(bytes: &[u8]) -> (usize, usize) {
    metrics_from_byte_len(bytes.len())
}

fn metrics_from_byte_len(bytes: usize) -> (usize, usize) {
    let tokens = bytes / CHARS_PER_TOKEN;
    (bytes, tokens)
}

fn synthetic_detection_path(logical_path: &Path) -> PathBuf {
    let mut path = PathBuf::from("__tokmd_in_memory_detection__");
    path.push(logical_path.file_name().unwrap_or(logical_path.as_os_str()));
    path
}

fn language_from_in_memory_shebang(bytes: &[u8]) -> Option<LanguageType> {
    const READ_LIMIT: usize = 128;

    let first_line = bytes[..bytes.len().min(READ_LIMIT)]
        .split(|b| *b == b'\n')
        .next()?;
    let first_line = std::str::from_utf8(first_line).ok()?;

    let direct = LanguageType::list()
        .iter()
        .map(|(lang, _)| *lang)
        .find(|lang| lang.shebangs().contains(&first_line));
    if direct.is_some() {
        return direct;
    }

    let mut words = first_line.split_whitespace();
    if words.next() == Some("#!/usr/bin/env") {
        let interpreter = env_interpreter_token(words)?;
        return language_from_env_interpreter(interpreter);
    }

    None
}

fn env_interpreter_token<'a>(words: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let mut skip_next = false;

    for word in words {
        if skip_next {
            skip_next = false;
            continue;
        }

        if word.is_empty() {
            continue;
        }

        if looks_like_env_assignment(word) {
            continue;
        }

        match word {
            "-S" | "--split-string" | "-i" | "--ignore-environment" => continue,
            "-u" | "--unset" | "-C" | "--chdir" | "-P" | "--default-path" | "-a" | "--argv0"
            | "--default-signal" | "--ignore-signal" | "--block-signal" => {
                skip_next = true;
                continue;
            }
            _ if word.starts_with("--unset=")
                || word.starts_with("--chdir=")
                || word.starts_with("--default-path=")
                || word.starts_with("--argv0=")
                || word.starts_with("--default-signal=")
                || word.starts_with("--ignore-signal=")
                || word.starts_with("--block-signal=") =>
            {
                continue;
            }
            _ if word.starts_with('-') => continue,
            _ => return Some(word),
        }
    }

    None
}

fn looks_like_env_assignment(word: &str) -> bool {
    let Some((name, _)) = word.split_once('=') else {
        return false;
    };

    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }

    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn language_from_env_interpreter(interpreter: &str) -> Option<LanguageType> {
    let token = interpreter
        .rsplit('/')
        .next()
        .unwrap_or(interpreter)
        // Some shells and malformed env invocations can surface "-python3"-style
        // interpreter tokens; strip the leading dash defensively before matching.
        .trim_start_matches('-');

    if token.starts_with("python") {
        return LanguageType::from_file_extension("py");
    }

    match token {
        "bash" | "sh" | "zsh" | "ksh" | "fish" => LanguageType::from_name("Bash"),
        "node" | "nodejs" => LanguageType::from_name("JavaScript"),
        "ruby" => LanguageType::from_name("Ruby"),
        "perl" | "perl5" => LanguageType::from_name("Perl"),
        "php" => LanguageType::from_name("PHP"),
        "pwsh" | "powershell" => LanguageType::from_name("PowerShell"),
        _ => None,
    }
}

fn detect_in_memory_language(
    logical_path: &Path,
    bytes: &[u8],
    config: &Config,
) -> Option<LanguageType> {
    let detection_path = synthetic_detection_path(logical_path);
    LanguageType::from_path(&detection_path, config)
        .or_else(|| language_from_in_memory_shebang(bytes))
}

fn insert_row<'a>(
    map: &mut BTreeMap<Key<'a>, (String, Agg)>,
    key: Key<'a>,
    module: String,
    stats: &CodeStats,
    bytes: usize,
    tokens: usize,
) {
    let entry = map.entry(key).or_insert_with(|| (module, Agg::default()));
    entry.1.code += stats.code;
    entry.1.comments += stats.comments;
    entry.1.blanks += stats.blanks;
    entry.1.bytes += bytes;
    entry.1.tokens += tokens;
}

fn rows_from_map<'a>(map: BTreeMap<Key<'a>, (String, Agg)>) -> Vec<FileRow> {
    map.into_iter()
        .map(|(key, (module, agg))| {
            let lines = agg.code + agg.comments + agg.blanks;
            FileRow {
                path: key.path,
                module,
                lang: key.lang.to_string(),
                kind: key.kind,
                code: agg.code,
                comments: agg.comments,
                blanks: agg.blanks,
                lines,
                bytes: agg.bytes,
                tokens: agg.tokens,
            }
        })
        .collect()
}

/// Collect `FileRow`s directly from ordered in-memory inputs.
///
/// This path avoids host filesystem metadata and keeps logical paths intact,
/// which makes it suitable for browser/WASM callers.
pub fn collect_in_memory_file_rows(
    inputs: &[InMemoryRowInput<'_>],
    module_roots: &[String],
    module_depth: usize,
    children: ChildIncludeMode,
    config: &Config,
) -> Vec<FileRow> {
    let mut map = BTreeMap::new();

    for input in inputs {
        let Some(lang_type) = detect_in_memory_language(input.logical_path, input.bytes, config)
        else {
            continue;
        };

        let path = normalize_path(input.logical_path, None);
        let module = module_key_from_normalized(&path, module_roots, module_depth);
        let stats = lang_type.parse_from_slice(input.bytes, config);
        let summary = stats.summarise();
        let (bytes, tokens) = metrics_from_bytes(input.bytes);

        if children == ChildIncludeMode::Separate {
            for (child_type, child_stats) in &stats.blobs {
                let child_summary = child_stats.summarise();
                insert_row(
                    &mut map,
                    Key {
                        path: path.clone(),
                        lang: child_type.name(),
                        kind: FileKind::Child,
                    },
                    module.clone(),
                    &child_summary,
                    0,
                    0,
                );
            }
        }

        insert_row(
            &mut map,
            Key {
                path,
                lang: lang_type.name(),
                kind: FileKind::Parent,
            },
            module,
            &summary,
            bytes,
            tokens,
        );
    }

    rows_from_map(map)
}

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

    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));

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

    // Sort descending by code, then by module name for determinism.
    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));

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
    let mut map = BTreeMap::new();

    // Parent reports
    for (lang_type, lang) in languages.iter() {
        for report in &lang.reports {
            let path = normalize_path(&report.name, strip_prefix);
            let module = module_key_from_normalized(&path, module_roots, module_depth);
            let st = report.stats.summarise();
            let (bytes, tokens) = get_file_metrics(&report.name);
            insert_row(
                &mut map,
                Key {
                    path,
                    lang: lang_type.name(),
                    kind: FileKind::Parent,
                },
                module,
                &st,
                bytes,
                tokens,
            );
        }
    }

    if children == ChildIncludeMode::Separate {
        for (_lang_type, lang) in languages.iter() {
            for (child_type, reports) in &lang.children {
                for report in reports {
                    let path = normalize_path(&report.name, strip_prefix);
                    let module = module_key_from_normalized(&path, module_roots, module_depth);
                    let st = report.stats.summarise();
                    insert_row(
                        &mut map,
                        Key {
                            path,
                            lang: child_type.name(),
                            kind: FileKind::Child,
                        },
                        module,
                        &st,
                        0,
                        0,
                    );
                }
            }
        }
    }

    rows_from_map(map)
}

pub fn unique_parent_file_count(languages: &Languages) -> usize {
    let rows = collect_file_rows(languages, &[], 1, ChildIncludeMode::ParentsOnly, None);
    unique_parent_file_count_from_rows(&rows)
}

pub fn unique_parent_file_count_from_rows(file_rows: &[FileRow]) -> usize {
    file_rows
        .iter()
        .filter(|row| row.kind == FileKind::Parent)
        .map(|row| row.path.as_str())
        .collect::<BTreeSet<_>>()
        .len()
}

/// Compute the average of `lines` over `files`, rounding to nearest integer.
///
/// Returns 0 if `files` is zero.
///
/// # Examples
///
/// ```
/// use tokmd_model::avg;
///
/// assert_eq!(avg(300, 3), 100);
/// assert_eq!(avg(0, 5), 0);
/// assert_eq!(avg(100, 0), 0);
/// // Rounds to nearest: 7 / 2 = 3.5 → 4
/// assert_eq!(avg(7, 2), 4);
/// ```
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
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use tokmd_model::normalize_path;
///
/// // Normalizes backslashes to forward slashes
/// let p = Path::new("src\\main.rs");
/// assert_eq!(normalize_path(p, None), "src/main.rs");
///
/// // Strips a prefix
/// let p = Path::new("project/src/lib.rs");
/// let prefix = Path::new("project");
/// assert_eq!(normalize_path(&p, Some(&prefix)), "src/lib.rs");
/// ```
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
    slice = slice.trim_start_matches('/');

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
///
/// # Examples
///
/// ```
/// use tokmd_model::module_key;
///
/// let roots = vec!["crates".to_string()];
/// assert_eq!(module_key("crates/foo/src/lib.rs", &roots, 2), "crates/foo");
/// assert_eq!(module_key("src/lib.rs", &roots, 2), "src");
/// assert_eq!(module_key("Cargo.toml", &roots, 2), "(root)");
/// ```
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

    mod normalize_properties {
        use super::*;
        use proptest::prelude::*;

        fn arb_path_component() -> impl Strategy<Value = String> {
            "[a-zA-Z0-9_.-]+"
        }

        fn arb_path(max_depth: usize) -> impl Strategy<Value = String> {
            prop::collection::vec(arb_path_component(), 1..=max_depth)
                .prop_map(|comps| comps.join("/"))
        }

        proptest! {
            #[test]
            fn normalize_path_is_idempotent(path in arb_path(5)) {
                let p = PathBuf::from(&path);
                let norm1 = normalize_path(&p, None);
                let p2 = PathBuf::from(&norm1);
                let norm2 = normalize_path(&p2, None);
                prop_assert_eq!(norm1, norm2);
            }

            #[test]
            fn normalize_path_handles_windows_separators(path in arb_path(5)) {
                let win_path = path.replace('/', "\\");
                let p_win = PathBuf::from(&win_path);
                let p_unix = PathBuf::from(&path);

                let norm_win = normalize_path(&p_win, None);
                let norm_unix = normalize_path(&p_unix, None);

                prop_assert_eq!(norm_win, norm_unix);
            }

            #[test]
            fn normalize_path_no_leading_slash(path in arb_path(5)) {
                let p = PathBuf::from(&path);
                let norm = normalize_path(&p, None);
                prop_assert!(!norm.starts_with('/'));
            }

            #[test]
            fn normalize_path_no_leading_dot_slash(path in arb_path(5)) {
                let p = PathBuf::from(&path);
                let norm = normalize_path(&p, None);
                prop_assert!(!norm.starts_with("./"));
            }

            #[test]
            fn module_key_deterministic(
                path in arb_path(5),
                roots in prop::collection::vec(arb_path_component(), 1..3),
                depth in 1usize..5
            ) {
                let k1 = module_key(&path, &roots, depth);
                let k2 = module_key(&path, &roots, depth);
                prop_assert_eq!(k1, k2);
            }
        }
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

    #[test]
    fn test_looks_like_env_assignment() {
        assert!(looks_like_env_assignment("FOO=bar"));
        assert!(looks_like_env_assignment("_FOO=bar"));
        assert!(looks_like_env_assignment("A_B_C=123"));

        assert!(!looks_like_env_assignment("="));
        assert!(!looks_like_env_assignment("=bar"));
        assert!(!looks_like_env_assignment("1FOO=bar"));
        assert!(!looks_like_env_assignment("FOO-BAR=baz"));
    }

    #[test]
    fn test_env_interpreter_token() {
        // Simple case
        assert_eq!(env_interpreter_token(vec!["python"].into_iter()), Some("python"));

        // Skip env assignments
        assert_eq!(env_interpreter_token(vec!["FOO=bar", "python"].into_iter()), Some("python"));

        // Skip common env flags without args
        assert_eq!(env_interpreter_token(vec!["-S", "-i", "python"].into_iter()), Some("python"));

        // Skip flags with next argument
        assert_eq!(env_interpreter_token(vec!["-u", "FOO", "-C", "/tmp", "python"].into_iter()), Some("python"));
        assert_eq!(env_interpreter_token(vec!["--unset", "FOO", "python"].into_iter()), Some("python"));

        // Skip long flags with = assignment
        assert_eq!(env_interpreter_token(vec!["--unset=FOO", "python"].into_iter()), Some("python"));
        assert_eq!(env_interpreter_token(vec!["--chdir=/tmp", "python"].into_iter()), Some("python"));
        assert_eq!(env_interpreter_token(vec!["--default-path=/bin", "python"].into_iter()), Some("python"));
        assert_eq!(env_interpreter_token(vec!["--argv0=sh", "python"].into_iter()), Some("python"));
        assert_eq!(env_interpreter_token(vec!["--default-signal=SIGINT", "python"].into_iter()), Some("python"));
        assert_eq!(env_interpreter_token(vec!["--ignore-signal=SIGINT", "python"].into_iter()), Some("python"));
        assert_eq!(env_interpreter_token(vec!["--block-signal=SIGINT", "python"].into_iter()), Some("python"));

        // Unknown flags starting with - are skipped (mimicking coreutils env behavior)
        assert_eq!(env_interpreter_token(vec!["--unknown-flag", "python"].into_iter()), Some("python"));

        // Empty words
        assert_eq!(env_interpreter_token(vec!["", "python"].into_iter()), Some("python"));

        // No interpreter found
        assert_eq!(env_interpreter_token(vec!["FOO=bar"].into_iter()), None);
    }
}
