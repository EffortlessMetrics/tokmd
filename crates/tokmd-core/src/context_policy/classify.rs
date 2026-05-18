//! File hygiene classification for context policy.

use tokmd_scan::normalize_slashes as normalize_path;
use tokmd_types::FileClassification;

use super::patterns::{FIXTURE_DIRS, GENERATED_PATTERNS, LOCKFILES, VENDORED_DIRS, basename};

/// Classify a file for context/handoff hygiene policy evaluation.
#[must_use]
pub fn classify_file(
    path: &str,
    tokens: usize,
    lines: usize,
    dense_threshold: f64,
) -> Vec<FileClassification> {
    let mut classes = Vec::new();
    let normalized = normalize_path(path);
    let basename = basename(&normalized);

    classify_basename(basename, &mut classes);
    classify_path(&normalized, &mut classes);
    classify_density(tokens, lines, dense_threshold, &mut classes);

    classes.sort();
    classes.dedup();
    classes
}

fn classify_basename(basename: &str, classes: &mut Vec<FileClassification>) {
    if LOCKFILES.contains(&basename) {
        classes.push(FileClassification::Lockfile);
    }

    if basename.ends_with(".min.js") || basename.ends_with(".min.css") {
        classes.push(FileClassification::Minified);
    }

    if basename.ends_with(".js.map") || basename.ends_with(".css.map") {
        classes.push(FileClassification::Sourcemap);
    }

    if GENERATED_PATTERNS
        .iter()
        .any(|pat| basename == *pat || basename.contains(pat))
    {
        classes.push(FileClassification::Generated);
    }
}

fn classify_path(normalized: &str, classes: &mut Vec<FileClassification>) {
    if VENDORED_DIRS
        .iter()
        .any(|dir| normalized.contains(dir) || normalized.starts_with(dir.trim_end_matches('/')))
    {
        classes.push(FileClassification::Vendored);
    }

    if FIXTURE_DIRS
        .iter()
        .any(|dir| normalized.contains(dir) || normalized.starts_with(dir.trim_end_matches('/')))
    {
        classes.push(FileClassification::Fixture);
    }
}

fn classify_density(
    tokens: usize,
    lines: usize,
    dense_threshold: f64,
    classes: &mut Vec<FileClassification>,
) {
    let effective_lines = lines.max(1);
    let tokens_per_line = tokens as f64 / effective_lines as f64;
    if tokens_per_line > dense_threshold {
        classes.push(FileClassification::DataBlob);
    }
}
