/// Check if a trimmed line starts a Rust function definition.
///
/// Handles all visibility qualifiers including `pub(in path::here)`,
/// optional `async`, `unsafe`, `const`, and `extern "ABI"` modifiers.
pub(in crate::complexity) fn is_rust_fn_start(trimmed: &str) -> bool {
    // Fast path: find "fn " in the line.
    let Some(fn_pos) = trimmed.find("fn ") else {
        return false;
    };

    // Everything before "fn " must be valid qualifiers.
    let prefix = trimmed[..fn_pos].trim();
    if prefix.is_empty() {
        return true;
    }

    // Parse prefix: valid tokens are pub/pub(...), async, unsafe, const, extern "...".
    let mut rest = prefix;
    while !rest.is_empty() {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        if rest.starts_with("pub(") {
            if let Some(close) = rest.find(')') {
                rest = &rest[close + 1..];
            } else {
                return false;
            }
        } else if let Some(r) = rest.strip_prefix("pub") {
            rest = r;
        } else if let Some(r) = rest.strip_prefix("async") {
            rest = r;
        } else if let Some(r) = rest.strip_prefix("unsafe") {
            rest = r;
        } else if let Some(r) = rest.strip_prefix("const") {
            rest = r;
        } else if rest.starts_with("extern") {
            rest = rest["extern".len()..].trim_start();
            if rest.starts_with('"') {
                if let Some(close) = rest[1..].find('"') {
                    rest = &rest[close + 2..];
                } else {
                    return false;
                }
            }
        } else {
            return false;
        }
    }

    true
}

pub(in crate::complexity) fn count_rust_functions(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut in_fn = false;
    let mut fn_start = 0;
    let mut brace_depth: i32 = 0;
    let mut in_string = false;
    let mut in_block_comment = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if !in_fn && is_rust_fn_start(trimmed) {
            count += 1;
            in_fn = true;
            fn_start = i;
            brace_depth = 0;
        }

        if in_fn {
            let chars: Vec<char> = line.chars().collect();
            let mut j = 0;
            while j < chars.len() {
                let c = chars[j];
                let next = chars.get(j + 1).copied();

                if in_block_comment {
                    if c == '*' && next == Some('/') {
                        in_block_comment = false;
                        j += 2;
                        continue;
                    }
                    j += 1;
                    continue;
                }

                if c == '/' && next == Some('/') {
                    break;
                }

                if c == '/' && next == Some('*') {
                    in_block_comment = true;
                    j += 2;
                    continue;
                }

                if c == '"' && (j == 0 || chars[j - 1] != '\\') {
                    in_string = !in_string;
                    j += 1;
                    continue;
                }

                if !in_string && !in_block_comment {
                    if c == '{' {
                        brace_depth += 1;
                    } else if c == '}' {
                        brace_depth = brace_depth.saturating_sub(1);
                        if brace_depth == 0 {
                            let fn_len = i - fn_start + 1;
                            max_len = max_len.max(fn_len);
                            in_fn = false;
                            break;
                        }
                    }
                }
                j += 1;
            }
        }
    }

    (count, max_len)
}
