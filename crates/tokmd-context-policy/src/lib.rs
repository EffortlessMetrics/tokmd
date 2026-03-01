//! Deterministic context/handoff policy helpers.

#![forbid(unsafe_code)]

use tokmd_path::normalize_slashes as normalize_path;
use tokmd_types::{FileClassification, InclusionPolicy};

/// Default maximum fraction of budget a single file may consume.
pub const DEFAULT_MAX_FILE_PCT: f64 = 0.15;
/// Default hard cap for a single file when no explicit cap is provided.
pub const DEFAULT_MAX_FILE_TOKENS: usize = 16_000;
/// Default tokens-per-line threshold for dense blob detection.
pub const DEFAULT_DENSE_THRESHOLD: f64 = 50.0;

const LOCKFILES: &[&str] = &[
    "Cargo.lock",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "poetry.lock",
    "Pipfile.lock",
    "go.sum",
    "composer.lock",
    "Gemfile.lock",
];

const SMART_EXCLUDE_SUFFIXES: &[(&str, &str)] = &[
    (".min.js", "minified"),
    (".min.css", "minified"),
    (".js.map", "sourcemap"),
    (".css.map", "sourcemap"),
];

const SPINE_PATTERNS: &[&str] = &[
    "README.md",
    "README",
    "README.rst",
    "README.txt",
    "ROADMAP.md",
    "docs/ROADMAP.md",
    "CONTRIBUTING.md",
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "go.mod",
    "docs/architecture.md",
    "docs/design.md",
    "tokmd.toml",
    "cockpit.toml",
];

const GENERATED_PATTERNS: &[&str] = &[
    "node-types.json",
    "grammar.json",
    ".generated.",
    ".pb.go",
    ".pb.rs",
    "_pb2.py",
    ".g.dart",
    ".freezed.dart",
];

const VENDORED_DIRS: &[&str] = &["vendor/", "third_party/", "third-party/", "node_modules/"];
const FIXTURE_DIRS: &[&str] = &[
    "fixtures/",
    "testdata/",
    "test_data/",
    "__snapshots__/",
    "golden/",
];

/// Returns the smart-exclude reason for a path, if any.
///
/// Reasons:
/// - `lockfile`
/// - `minified`
/// - `sourcemap`
#[must_use]
pub fn smart_exclude_reason(path: &str) -> Option<&'static str> {
    let basename = path.rsplit('/').next().unwrap_or(path);

    if LOCKFILES.contains(&basename) {
        return Some("lockfile");
    }

    for &(suffix, reason) in SMART_EXCLUDE_SUFFIXES {
        if basename.ends_with(suffix) {
            return Some(reason);
        }
    }

    None
}

/// Returns `true` when a path matches a "spine" file that should be prioritized.
#[must_use]
pub fn is_spine_file(path: &str) -> bool {
    let normalized = normalize_path(path);
    let basename = normalized.rsplit('/').next().unwrap_or(&normalized);

    for &pattern in SPINE_PATTERNS {
        if pattern.contains('/') {
            if normalized == pattern || normalized.ends_with(&format!("/{pattern}")) {
                return true;
            }
        } else if basename == pattern {
            return true;
        }
    }

    false
}

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
    let basename = normalized.rsplit('/').next().unwrap_or(&normalized);

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

    let effective_lines = lines.max(1);
    let tokens_per_line = tokens as f64 / effective_lines as f64;
    if tokens_per_line > dense_threshold {
        classes.push(FileClassification::DataBlob);
    }

    classes.sort();
    classes.dedup();
    classes
}

/// Compute the maximum tokens a single file may consume.
#[must_use]
pub fn compute_file_cap(budget: usize, max_file_pct: f64, max_file_tokens: Option<usize>) -> usize {
    if budget == usize::MAX {
        return usize::MAX;
    }

    let pct_cap = (budget as f64 * max_file_pct) as usize;
    let hard_cap = max_file_tokens.unwrap_or(DEFAULT_MAX_FILE_TOKENS);
    pct_cap.min(hard_cap)
}

/// Assign an inclusion policy based on size and file classifications.
#[must_use]
pub fn assign_policy(
    tokens: usize,
    file_cap: usize,
    classifications: &[FileClassification],
) -> (InclusionPolicy, Option<String>) {
    if tokens <= file_cap {
        return (InclusionPolicy::Full, None);
    }

    let skip_classes = [
        FileClassification::Generated,
        FileClassification::DataBlob,
        FileClassification::Vendored,
    ];

    if classifications.iter().any(|c| skip_classes.contains(c)) {
        let class_names: Vec<&str> = classifications.iter().map(classification_name).collect();
        return (
            InclusionPolicy::Skip,
            Some(format!(
                "{} file exceeds cap ({} > {} tokens)",
                class_names.join("+"),
                tokens,
                file_cap
            )),
        );
    }

    (
        InclusionPolicy::HeadTail,
        Some(format!(
            "file exceeds cap ({} > {} tokens); head+tail included",
            tokens, file_cap
        )),
    )
}

fn classification_name(classification: &FileClassification) -> &'static str {
    match classification {
        FileClassification::Generated => "generated",
        FileClassification::Fixture => "fixture",
        FileClassification::Vendored => "vendored",
        FileClassification::Lockfile => "lockfile",
        FileClassification::Minified => "minified",
        FileClassification::DataBlob => "data_blob",
        FileClassification::Sourcemap => "sourcemap",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smart_exclude_reason_detects_lockfiles_and_sourcemaps() {
        assert_eq!(smart_exclude_reason("Cargo.lock"), Some("lockfile"));
        assert_eq!(smart_exclude_reason("dist/app.js.map"), Some("sourcemap"));
        assert_eq!(smart_exclude_reason("src/main.rs"), None);
    }

    #[test]
    fn is_spine_file_matches_basename_and_document_paths() {
        assert!(is_spine_file("README.md"));
        assert!(is_spine_file("nested/docs/architecture.md"));
        assert!(!is_spine_file("src/main.rs"));
    }

    #[test]
    fn classify_file_detects_generated_and_dense_blob() {
        let classes = classify_file("src/node-types.json", 50_000, 5, 50.0);
        assert!(classes.contains(&FileClassification::Generated));
        assert!(classes.contains(&FileClassification::DataBlob));
    }

    #[test]
    fn assign_policy_skips_oversized_generated_files() {
        let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::Generated]);
        assert_eq!(policy, InclusionPolicy::Skip);
        assert!(reason.unwrap_or_default().contains("generated"));
    }

    // ========================
    // compute_file_cap edge cases
    // ========================

    #[test]
    fn compute_file_cap_returns_max_for_unlimited_budget() {
        assert_eq!(compute_file_cap(usize::MAX, 0.15, None), usize::MAX);
    }

    #[test]
    fn compute_file_cap_respects_pct_when_smaller() {
        // budget=10_000, 15% = 1500, hard cap defaults to 16_000
        // pct cap (1500) < hard cap (16_000), so pct wins
        assert_eq!(compute_file_cap(10_000, 0.15, None), 1_500);
    }

    #[test]
    fn compute_file_cap_respects_hard_cap_when_smaller() {
        // budget=1_000_000, 15% = 150_000, hard cap = 500
        assert_eq!(compute_file_cap(1_000_000, 0.15, Some(500)), 500);
    }

    // ========================
    // classify_file additional cases
    // ========================

    #[test]
    fn classify_file_detects_lockfile() {
        let classes = classify_file("Cargo.lock", 100, 50, 50.0);
        assert!(classes.contains(&FileClassification::Lockfile));
    }

    #[test]
    fn classify_file_detects_minified_js() {
        let classes = classify_file("dist/app.min.js", 100, 50, 50.0);
        assert!(classes.contains(&FileClassification::Minified));
    }

    #[test]
    fn classify_file_detects_vendored_dir() {
        let classes = classify_file("vendor/lib/foo.go", 100, 50, 50.0);
        assert!(classes.contains(&FileClassification::Vendored));
    }

    #[test]
    fn classify_file_detects_fixture_dir() {
        let classes = classify_file("testdata/sample.json", 100, 50, 50.0);
        assert!(classes.contains(&FileClassification::Fixture));
    }

    #[test]
    fn classify_file_normal_file_has_no_classifications() {
        let classes = classify_file("src/main.rs", 100, 50, 50.0);
        assert!(classes.is_empty());
    }

    // ========================
    // assign_policy additional cases
    // ========================

    #[test]
    fn assign_policy_full_when_under_cap() {
        let (policy, reason) = assign_policy(100, 16_000, &[]);
        assert_eq!(policy, InclusionPolicy::Full);
        assert!(reason.is_none());
    }

    #[test]
    fn assign_policy_head_tail_for_oversized_normal_file() {
        let (policy, reason) = assign_policy(20_000, 16_000, &[]);
        assert_eq!(policy, InclusionPolicy::HeadTail);
        assert!(reason.unwrap_or_default().contains("head+tail"));
    }

    #[test]
    fn assign_policy_skips_oversized_vendored() {
        let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::Vendored]);
        assert_eq!(policy, InclusionPolicy::Skip);
        assert!(reason.unwrap_or_default().contains("vendored"));
    }

    // ========================
    // smart_exclude_reason additional cases
    // ========================

    #[test]
    fn smart_exclude_reason_detects_minified_css() {
        assert_eq!(smart_exclude_reason("styles.min.css"), Some("minified"));
    }

    #[test]
    fn smart_exclude_reason_detects_nested_lockfile() {
        assert_eq!(
            smart_exclude_reason("some/deep/path/yarn.lock"),
            Some("lockfile")
        );
    }

    #[test]
    fn smart_exclude_reason_returns_none_for_normal_file() {
        assert_eq!(smart_exclude_reason("src/lib.rs"), None);
    }

    // ========================
    // is_spine_file additional cases
    // ========================

    #[test]
    fn is_spine_file_matches_cargo_toml() {
        assert!(is_spine_file("Cargo.toml"));
    }

    #[test]
    fn is_spine_file_matches_nested_readme() {
        assert!(is_spine_file("subdir/README.md"));
    }

    #[test]
    fn is_spine_file_does_not_match_source_file() {
        assert!(!is_spine_file("src/main.rs"));
    }
}
