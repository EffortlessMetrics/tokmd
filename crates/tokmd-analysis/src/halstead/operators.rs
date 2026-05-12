//! Language support and operator dispatch for Halstead analysis.

mod sets;

/// Languages that support Halstead analysis.
pub(crate) fn is_halstead_lang(lang: &str) -> bool {
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

/// Operators for a given language.
pub(crate) fn operators_for_lang(lang: &str) -> &'static [&'static str] {
    match lang.to_lowercase().as_str() {
        "rust" => sets::RUST,
        "javascript" | "typescript" => sets::JAVASCRIPT_TYPESCRIPT,
        "python" => sets::PYTHON,
        "go" => sets::GO,
        "c" | "c++" | "java" | "c#" | "php" => sets::C_FAMILY,
        "ruby" => sets::RUBY,
        _ => &[],
    }
}
