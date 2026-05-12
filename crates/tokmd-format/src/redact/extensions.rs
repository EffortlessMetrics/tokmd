//! Safe extension policy for redacted path output.

const SAFE_PATH_EXTENSIONS: &[&str] = &[
    "astro", "bash", "c", "cc", "cjs", "clj", "cljs", "cpp", "cs", "css", "csv", "cxx", "dart",
    "erl", "ex", "exs", "fish", "fs", "fsx", "gif", "go", "gz", "h", "hpp", "hrl", "htm", "html",
    "java", "jpeg", "jpg", "js", "json", "jsonl", "jsx", "kt", "kts", "lock", "lua", "md", "mjs",
    "otf", "pdf", "php", "pl", "pm", "png", "ps1", "py", "pyi", "r", "rb", "rs", "scala", "scss",
    "sh", "sql", "svelte", "svg", "swift", "toml", "ts", "tsv", "tsx", "ttf", "txt", "vue", "wasm",
    "webp", "woff", "woff2", "xml", "yaml", "yml", "zsh",
];

pub(super) fn safe_path_extension(ext: &str) -> Option<&str> {
    let lower = ext.to_ascii_lowercase();
    SAFE_PATH_EXTENSIONS
        .binary_search(&lower.as_str())
        .ok()
        .map(|_| ext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_path_extensions_are_strictly_sorted() {
        for w in SAFE_PATH_EXTENSIONS.windows(2) {
            assert!(
                w[0] < w[1],
                "SAFE_PATH_EXTENSIONS must be strictly sorted alphabetically. Out of order: {:?} >= {:?}",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn all_safe_path_extensions_are_preserved() {
        for &ext in SAFE_PATH_EXTENSIONS {
            let path = format!("file.{}", ext);
            let redacted = super::super::redact_path(&path);
            assert!(
                redacted.ends_with(&format!(".{}", ext)),
                "Extension {:?} was not preserved during redaction! Redacted: {}",
                ext,
                redacted
            );
        }
    }
}
