use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use tokmd_analysis_types::FileStatRow;

pub(crate) fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(any(feature = "git", feature = "content"))]
pub(crate) fn normalize_path(path: &str, root: &Path) -> String {
    let mut out = path.replace('\\', "/");
    if let Ok(stripped) = Path::new(&out).strip_prefix(root) {
        out = stripped.to_string_lossy().replace('\\', "/");
    }
    if let Some(stripped) = out.strip_prefix("./") {
        out = stripped.to_string();
    }
    out
}

pub(crate) fn path_depth(path: &str) -> usize {
    path.split('/')
        .filter(|seg| !seg.is_empty())
        .count()
        .max(1)
}

pub(crate) fn is_test_path(path: &str) -> bool {
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

pub(crate) fn is_infra_lang(lang: &str) -> bool {
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

pub(crate) fn percentile(sorted: &[usize], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (pct * (sorted.len() as f64 - 1.0)).ceil() as usize;
    sorted[idx.min(sorted.len() - 1)] as f64
}

pub(crate) fn gini_coefficient(sorted: &[usize]) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let n = sorted.len() as f64;
    let sum: f64 = sorted.iter().map(|v| *v as f64).sum();
    if sum == 0.0 {
        return 0.0;
    }
    let mut accum = 0.0;
    for (i, value) in sorted.iter().enumerate() {
        let i = i as f64 + 1.0;
        accum += (2.0 * i - n - 1.0) * (*value as f64);
    }
    accum / (n * sum)
}

pub(crate) fn safe_ratio(numer: usize, denom: usize) -> f64 {
    if denom == 0 {
        0.0
    } else {
        round_f64(numer as f64 / denom as f64, 4)
    }
}

pub(crate) fn round_f64(value: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (value * factor).round() / factor
}

pub(crate) fn empty_file_row() -> FileStatRow {
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