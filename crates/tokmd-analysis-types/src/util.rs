use std::path::{Path, PathBuf};
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::time::{SystemTime, UNIX_EPOCH};

use crate::FileStatRow;

#[derive(Debug, Clone, Default)]
pub struct AnalysisLimits {
    pub max_files: Option<usize>,
    pub max_bytes: Option<u64>,
    pub max_file_bytes: Option<u64>,
    pub max_commits: Option<usize>,
    pub max_commit_files: Option<usize>,
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub fn now_ms() -> u128 {
    // `std::time` is not implemented on bare wasm. Keep analysis receipts
    // deterministic until the browser path provides a host clock explicitly.
    0
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub fn normalize_path(path: &str, root: &Path) -> String {
    let mut out = path.replace('\\', "/");
    if let Ok(stripped) = Path::new(&out).strip_prefix(root) {
        out = stripped.to_string_lossy().replace('\\', "/");
    }
    while let Some(stripped) = out.strip_prefix("./") {
        out = stripped.to_string();
    }
    out
}

pub fn path_depth(path: &str) -> usize {
    path.split('/').filter(|seg| !seg.is_empty()).count().max(1)
}

pub fn is_test_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    if lower.contains("/test/") || lower.contains("/tests/") || lower.contains("__tests__") {
        return true;
    }
    if lower.contains("/spec/") || lower.contains("/specs/") {
        return true;
    }
    let name = lower.rsplit('/').next().unwrap_or(&lower);
    name.contains("_test")
        || name.contains(".test.")
        || name.contains(".spec.")
        || name.starts_with("test_")
        || name.ends_with("_test.rs")
}

pub fn is_infra_lang(lang: &str) -> bool {
    let l = lang.to_lowercase();
    matches!(
        l.as_str(),
        "json"
            | "yaml"
            | "toml"
            | "markdown"
            | "xml"
            | "html"
            | "css"
            | "scss"
            | "less"
            | "makefile"
            | "dockerfile"
            | "hcl"
            | "terraform"
            | "nix"
            | "cmake"
            | "ini"
            | "properties"
            | "gitignore"
            | "gitconfig"
            | "editorconfig"
            | "csv"
            | "tsv"
            | "svg"
    )
}

pub fn empty_file_row() -> FileStatRow {
    FileStatRow {
        path: String::new(),
        module: String::new(),
        lang: String::new(),
        code: 0,
        comments: 0,
        blanks: 0,
        lines: 0,
        bytes: 0,
        tokens: 0,
        doc_pct: None,
        bytes_per_line: None,
        depth: 0,
    }
}

pub fn normalize_root(root: &Path) -> PathBuf {
    root.canonicalize().unwrap_or_else(|_| root.to_path_buf())
}

#[cfg(test)]
mod tests;
