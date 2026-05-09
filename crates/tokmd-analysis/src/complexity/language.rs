//! Language compatibility helpers for complexity analysis.

/// Map language strings to complexity-compatible names.
pub(super) fn map_language_for_complexity(lang: &str) -> &str {
    match lang.to_lowercase().as_str() {
        "rust" => "rust",
        "javascript" | "jsx" => "javascript",
        "typescript" | "tsx" => "typescript",
        "python" => "python",
        "go" => "go",
        "c" => "c",
        "c++" | "cpp" => "c++",
        "java" => "java",
        "c#" | "csharp" => "c#",
        "php" => "php",
        "ruby" => "ruby",
        _ => lang,
    }
}

/// Languages that support complexity analysis.
pub(super) fn is_complexity_lang(lang: &str) -> bool {
    matches!(
        lang.to_lowercase().as_str(),
        "rust"
            | "javascript"
            | "typescript"
            | "python"
            | "go"
            | "c"
            | "c++"
            | "java"
            | "c#"
            | "php"
            | "ruby"
    )
}
