use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use tokmd_analysis_types::FileStatRow;

pub use tokmd_math::{gini_coefficient, percentile, round_f64, safe_ratio};

#[derive(Debug, Clone, Default)]
pub struct AnalysisLimits {
    pub max_files: Option<usize>,
    pub max_bytes: Option<u64>,
    pub max_file_bytes: Option<u64>,
    pub max_commits: Option<usize>,
    pub max_commit_files: Option<usize>,
}

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
    if let Some(stripped) = out.strip_prefix("./") {
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
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn normalize_path_replaces_backslashes_and_leading_dot_slash() {
        let root = PathBuf::from("repo");
        assert_eq!(normalize_path(r".\src\lib.rs", &root), "src/lib.rs");
    }

    #[test]
    fn normalize_path_is_deterministic() {
        let root = PathBuf::from("repo");
        let input = r".\src\main.rs";
        assert_eq!(normalize_path(input, &root), normalize_path(input, &root));
    }

    proptest! {
        #[test]
        fn path_depth_always_at_least_one(path in "\\PC*") {
            let depth = path_depth(&path);
            prop_assert!(depth >= 1, "Path depth should always be at least 1");
        }

        #[test]
        fn path_depth_counts_segments(segments in prop::collection::vec("[a-zA-Z0-9_]+", 1..10)) {
            let path = segments.join("/");
            let depth = path_depth(&path);
            prop_assert_eq!(depth, segments.len(), "Depth should equal segment count for {}", path);
        }

        #[test]
        fn path_depth_ignores_empty_segments(segments in prop::collection::vec("[a-zA-Z0-9_]+", 1..5)) {
            let path_normal = segments.join("/");
            let path_with_double = segments.join("//");
            let path_with_trailing = format!("{}/", path_normal);
            let path_with_leading = format!("/{}", path_normal);

            let d_normal = path_depth(&path_normal);
            let d_double = path_depth(&path_with_double);
            let d_trailing = path_depth(&path_with_trailing);
            let d_leading = path_depth(&path_with_leading);

            prop_assert_eq!(d_normal, d_double, "Double slashes should not add depth");
            prop_assert_eq!(d_normal, d_trailing, "Trailing slash should not add depth");
            prop_assert_eq!(d_normal, d_leading, "Leading slash should not add depth");
        }

        #[test]
        fn is_test_path_case_insensitive_for_dirs(prefix in "[a-zA-Z0-9_/]+", suffix in "[a-zA-Z0-9_/]+\\.rs") {
            let lower = format!("{}/test/{}", prefix, suffix);
            let upper = format!("{}/TEST/{}", prefix, suffix);
            let mixed = format!("{}/TeSt/{}", prefix, suffix);

            prop_assert_eq!(is_test_path(&lower), is_test_path(&upper), "Case sensitivity issue with TEST dir");
            prop_assert_eq!(is_test_path(&lower), is_test_path(&mixed), "Case sensitivity issue with TeSt dir");
        }

        #[test]
        fn is_test_path_known_test_dirs_detected(dir in prop::sample::select(vec!["test", "tests", "__tests__", "spec", "specs"])) {
            let path = format!("src/{}/foo.rs", dir);
            prop_assert!(is_test_path(&path), "Should detect test dir: {}", dir);
        }

        #[test]
        fn is_test_path_file_patterns_detected(pattern in prop::sample::select(vec!["foo_test.rs", "test_foo.rs", "foo.test.js", "foo.spec.ts"])) {
            let path = format!("src/{}", pattern);
            prop_assert!(is_test_path(&path), "Should detect test file pattern: {}", pattern);
        }

        #[test]
        fn is_infra_lang_case_insensitive(lang in prop::sample::select(vec!["json", "yaml", "toml", "markdown", "xml", "html", "css"])) {
            prop_assert!(is_infra_lang(lang), "Should detect infra lang: {}", lang);
            prop_assert!(is_infra_lang(&lang.to_uppercase()), "Should detect infra lang (upper): {}", lang.to_uppercase());
        }

        #[test]
        fn is_infra_lang_known_infra_detected(lang in prop::sample::select(vec![
            "json", "yaml", "toml", "markdown", "xml", "html", "css", "scss", "less",
            "makefile", "dockerfile", "hcl", "terraform", "nix", "cmake", "ini",
            "properties", "gitignore", "gitconfig", "editorconfig", "csv", "tsv", "svg"
        ])) {
            prop_assert!(is_infra_lang(lang), "Should detect known infra lang: {}", lang);
        }

        #[test]
        fn is_infra_lang_code_langs_not_infra(lang in prop::sample::select(vec![
            "rust", "python", "javascript", "typescript", "go", "java", "c", "cpp"
        ])) {
            prop_assert!(!is_infra_lang(lang), "Code lang should not be infra: {}", lang);
        }
    }
}
