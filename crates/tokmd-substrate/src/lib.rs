//! # tokmd-substrate
//!
//! **Tier 0 (Pure Data)**
//!
//! Shared context that eliminates redundant I/O across sensors.
//! The substrate is built once (scan + git diff) and shared with
//! all sensors that run against the same repository.
//!
//! ## What belongs here
//! * `RepoSubstrate`, `SubstrateFile`, `LangSummary`, `DiffRange`
//! * Pure data types with Serde derive
//!
//! ## What does NOT belong here
//! * I/O operations (substrate building is in tokmd-sensor)
//! * Business logic or analysis

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Shared context for a scanned repository.
///
/// Built once from a tokei scan (and optionally git diff), then
/// passed to every sensor that needs file-level context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSubstrate {
    /// Normalized repo root path (forward slashes).
    pub repo_root: String,
    /// All scanned files, sorted by path.
    pub files: Vec<SubstrateFile>,
    /// Per-language aggregates.
    pub lang_summary: BTreeMap<String, LangSummary>,
    /// Git diff context (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_range: Option<DiffRange>,
    /// Total estimated tokens across all files.
    pub total_tokens: usize,
    /// Total bytes across all files.
    pub total_bytes: usize,
    /// Total lines of code across all files.
    pub total_code_lines: usize,
}

/// A single file in the substrate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstrateFile {
    /// Repo-relative path (forward slashes).
    pub path: String,
    /// Detected language.
    pub lang: String,
    /// Lines of code.
    pub code: usize,
    /// Total lines.
    pub lines: usize,
    /// File size in bytes.
    pub bytes: usize,
    /// Estimated token count.
    pub tokens: usize,
    /// Pre-computed module key.
    pub module: String,
    /// Whether this file was modified in the current diff range.
    pub in_diff: bool,
}

/// Per-language summary in the substrate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangSummary {
    /// Number of files.
    pub files: usize,
    /// Lines of code.
    pub code: usize,
    /// Total lines.
    pub lines: usize,
    /// Total bytes.
    pub bytes: usize,
    /// Estimated tokens.
    pub tokens: usize,
}

/// Git diff range context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffRange {
    /// Base ref (e.g., "main", "v1.0.0").
    pub base: String,
    /// Head ref (e.g., "HEAD", "feature-branch").
    pub head: String,
    /// Files changed in the diff.
    pub changed_files: Vec<String>,
    /// Number of commits in the range.
    pub commit_count: usize,
    /// Total insertions.
    pub insertions: usize,
    /// Total deletions.
    pub deletions: usize,
}

impl RepoSubstrate {
    /// Get files modified in the current diff range.
    pub fn diff_files(&self) -> impl Iterator<Item = &SubstrateFile> {
        self.files.iter().filter(|f| f.in_diff)
    }

    /// Get files for a specific language.
    pub fn files_for_lang(&self, lang: &str) -> impl Iterator<Item = &SubstrateFile> {
        self.files.iter().filter(move |f| f.lang == lang)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_substrate() -> RepoSubstrate {
        RepoSubstrate {
            repo_root: "/repo".to_string(),
            files: vec![
                SubstrateFile {
                    path: "src/lib.rs".to_string(),
                    lang: "Rust".to_string(),
                    code: 100,
                    lines: 120,
                    bytes: 3000,
                    tokens: 750,
                    module: "src".to_string(),
                    in_diff: true,
                },
                SubstrateFile {
                    path: "src/main.rs".to_string(),
                    lang: "Rust".to_string(),
                    code: 50,
                    lines: 60,
                    bytes: 1500,
                    tokens: 375,
                    module: "src".to_string(),
                    in_diff: false,
                },
            ],
            lang_summary: BTreeMap::from([(
                "Rust".to_string(),
                LangSummary {
                    files: 2,
                    code: 150,
                    lines: 180,
                    bytes: 4500,
                    tokens: 1125,
                },
            )]),
            diff_range: Some(DiffRange {
                base: "main".to_string(),
                head: "HEAD".to_string(),
                changed_files: vec!["src/lib.rs".to_string()],
                commit_count: 3,
                insertions: 10,
                deletions: 5,
            }),
            total_tokens: 1125,
            total_bytes: 4500,
            total_code_lines: 150,
        }
    }

    #[test]
    fn serde_roundtrip() {
        let sub = sample_substrate();
        let json = serde_json::to_string(&sub).unwrap();
        let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
        assert_eq!(back.files.len(), 2);
        assert_eq!(back.total_code_lines, 150);
        assert!(back.diff_range.is_some());
    }

    #[test]
    fn diff_files_filter() {
        let sub = sample_substrate();
        let diff: Vec<_> = sub.diff_files().collect();
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].path, "src/lib.rs");
    }

    #[test]
    fn files_for_lang_filter() {
        let sub = sample_substrate();
        let rust_files: Vec<_> = sub.files_for_lang("Rust").collect();
        assert_eq!(rust_files.len(), 2);
        let go_files: Vec<_> = sub.files_for_lang("Go").collect();
        assert_eq!(go_files.len(), 0);
    }

    #[test]
    fn btreemap_ordering() {
        let sub = sample_substrate();
        let keys: Vec<_> = sub.lang_summary.keys().collect();
        // BTreeMap ensures deterministic ordering
        assert_eq!(keys, vec!["Rust"]);
    }
}
