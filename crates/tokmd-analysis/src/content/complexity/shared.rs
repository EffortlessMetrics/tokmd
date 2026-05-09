//! Shared source-text predicates for content complexity analysis.

/// Get the indentation level (number of leading whitespace characters).
pub(super) fn get_indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// Check if line is a comment.
pub(super) fn is_comment_line(trimmed: &str, lang: &str) -> bool {
    match lang {
        "python" | "py" => trimmed.starts_with('#'),
        _ => {
            trimmed.starts_with("//")
                || trimmed.starts_with("/*")
                || trimmed.starts_with('*')
                || trimmed.starts_with("*/")
        }
    }
}

/// Count occurrences of keyword ensuring it's a word boundary.
pub(super) fn count_keyword(line: &str, keyword: &str) -> usize {
    let mut count = 0;
    let mut pos = 0;

    while let Some(idx) = line[pos..].find(keyword) {
        let abs_pos = pos + idx;
        // Check it's at word boundary (not part of larger identifier)
        let before_ok = abs_pos == 0
            || !line[..abs_pos]
                .chars()
                .last()
                .map(|c| c.is_alphanumeric() || c == '_')
                .unwrap_or(false);

        if before_ok {
            count += 1;
        }
        pos = abs_pos + keyword.len();
    }

    count
}
