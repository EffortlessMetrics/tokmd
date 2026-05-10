//! File-row collection for model receipts.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use tokei::{CodeStats, Config, LanguageType, Languages};
use tokmd_types::{ChildIncludeMode, FileKind, FileRow};

use crate::module_key::module_key_from_normalized;
use crate::normalize_path;
use crate::sorting::sort_file_rows;

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

    let mut rows = rows_from_map(map);
    sort_file_rows(&mut rows);
    rows
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

    let mut rows = rows_from_map(map);
    sort_file_rows(&mut rows);
    rows
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn looks_like_env_assignment_identifies_valid_names() {
        assert!(looks_like_env_assignment("FOO=bar"));
        assert!(looks_like_env_assignment("_FOO=bar"));
        assert!(looks_like_env_assignment("A_B_C=123"));

        assert!(!looks_like_env_assignment("="));
        assert!(!looks_like_env_assignment("=bar"));
        assert!(!looks_like_env_assignment("1FOO=bar"));
        assert!(!looks_like_env_assignment("FOO-BAR=baz"));
    }

    #[test]
    fn byte_metrics_use_floor_token_estimate() {
        assert_eq!(metrics_from_byte_len(0), (0, 0));
        assert_eq!(metrics_from_byte_len(12), (12, 3));
        assert_eq!(metrics_from_byte_len(15), (15, 3));
        assert_eq!(metrics_from_bytes(b"hello world!"), (12, 3));
    }

    #[test]
    fn env_interpreter_token_skips_env_arguments() {
        assert_eq!(
            env_interpreter_token(vec!["python"].into_iter()),
            Some("python")
        );

        assert_eq!(
            env_interpreter_token(vec!["FOO=bar", "python"].into_iter()),
            Some("python")
        );

        assert_eq!(
            env_interpreter_token(vec!["-S", "-i", "python"].into_iter()),
            Some("python")
        );
        assert_eq!(
            env_interpreter_token(vec!["--split-string", "python"].into_iter()),
            Some("python")
        );
        assert_eq!(
            env_interpreter_token(vec!["--ignore-environment", "python"].into_iter()),
            Some("python")
        );

        assert_eq!(
            env_interpreter_token(vec!["-u", "FOO", "-C", "/tmp", "python"].into_iter()),
            Some("python")
        );
        assert_eq!(
            env_interpreter_token(vec!["--unset", "FOO", "python"].into_iter()),
            Some("python")
        );

        assert_eq!(
            env_interpreter_token(vec!["--unset=FOO", "python"].into_iter()),
            Some("python")
        );
        assert_eq!(
            env_interpreter_token(vec!["--chdir=/tmp", "python"].into_iter()),
            Some("python")
        );
        assert_eq!(
            env_interpreter_token(vec!["--default-path=/bin", "python"].into_iter()),
            Some("python")
        );
        assert_eq!(
            env_interpreter_token(vec!["--argv0=sh", "python"].into_iter()),
            Some("python")
        );
        assert_eq!(
            env_interpreter_token(vec!["--default-signal=SIGINT", "python"].into_iter()),
            Some("python")
        );
        assert_eq!(
            env_interpreter_token(vec!["--ignore-signal=SIGINT", "python"].into_iter()),
            Some("python")
        );
        assert_eq!(
            env_interpreter_token(vec!["--block-signal=SIGINT", "python"].into_iter()),
            Some("python")
        );

        assert_eq!(
            env_interpreter_token(vec!["--unknown-flag", "python"].into_iter()),
            Some("python")
        );

        assert_eq!(
            env_interpreter_token(vec!["", "python"].into_iter()),
            Some("python")
        );

        assert_eq!(env_interpreter_token(vec!["FOO=bar"].into_iter()), None);
    }
}
