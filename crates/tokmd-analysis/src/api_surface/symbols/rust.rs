use super::{Symbol, has_doc_comment};

pub(super) fn extract_symbols(lines: &[&str]) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // Skip lines inside string literals or comments (simple heuristic)
        if trimmed.starts_with("//") || trimmed.starts_with('*') || trimmed.starts_with("/*") {
            continue;
        }

        let is_public = is_pub_item(trimmed);
        let is_internal = is_internal_item(trimmed);

        if is_public || is_internal {
            symbols.push(Symbol {
                is_public,
                is_documented: has_doc_comment(lines, i),
            });
        }
    }

    symbols
}

fn is_pub_item(trimmed: &str) -> bool {
    // Match pub items, including pub(crate), pub(super), pub(in ...)
    if !trimmed.starts_with("pub ") && !trimmed.starts_with("pub(") {
        return false;
    }

    // Find the part after the pub qualifier
    let after_pub = if trimmed.starts_with("pub(") {
        // Find matching close paren
        if let Some(close) = trimmed.find(')') {
            trimmed[close + 1..].trim_start()
        } else {
            return false;
        }
    } else {
        // "pub " prefix
        &trimmed[4..]
    };

    // Now check for item keywords
    after_pub.starts_with("fn ")
        || after_pub.starts_with("struct ")
        || after_pub.starts_with("enum ")
        || after_pub.starts_with("trait ")
        || after_pub.starts_with("type ")
        || after_pub.starts_with("const ")
        || after_pub.starts_with("static ")
        || after_pub.starts_with("mod ")
        || after_pub.starts_with("async fn ")
        || after_pub.starts_with("unsafe fn ")
        || after_pub.starts_with("unsafe trait ")
}

fn is_internal_item(trimmed: &str) -> bool {
    // Non-pub items at start of line (no leading whitespace for top-level heuristic
    // but we keep it simple: any fn/struct/etc. without pub)
    if trimmed.starts_with("pub ") || trimmed.starts_with("pub(") {
        return false;
    }

    trimmed.starts_with("fn ")
        || trimmed.starts_with("struct ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("trait ")
        || trimmed.starts_with("type ")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("static ")
        || trimmed.starts_with("mod ")
        || trimmed.starts_with("async fn ")
        || trimmed.starts_with("unsafe fn ")
        || trimmed.starts_with("unsafe trait ")
}
