//! Source function counting helpers for complexity reports.

pub(super) fn count_functions(lang: &str, text: &str) -> (usize, usize) {
    let lines: Vec<&str> = text.lines().collect();
    match lang.to_lowercase().as_str() {
        "rust" => count_rust_functions(&lines),
        "javascript" | "typescript" => count_js_functions(&lines),
        "python" => count_python_functions(&lines),
        "go" => count_go_functions(&lines),
        "c" | "c++" | "java" | "c#" | "php" => count_c_style_functions(&lines),
        "ruby" => count_ruby_functions(&lines),
        _ => (0, 0),
    }
}

/// Check if a trimmed line starts a Rust function definition.
///
/// Handles all visibility qualifiers including `pub(in path::here)`,
/// optional `async`, `unsafe`, `const`, and `extern "ABI"` modifiers.
pub(super) fn is_rust_fn_start(trimmed: &str) -> bool {
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

pub(super) fn count_rust_functions(lines: &[&str]) -> (usize, usize) {
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

fn count_js_functions(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut in_fn = false;
    let mut fn_start = 0;
    let mut brace_depth = 0usize;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        let is_fn_start = trimmed.starts_with("function ")
            || trimmed.starts_with("async function ")
            || trimmed.contains("=> {")
            || (trimmed.contains("(")
                && trimmed.contains(") {")
                && !trimmed.starts_with("if ")
                && !trimmed.starts_with("while ")
                && !trimmed.starts_with("for ")
                && !trimmed.starts_with("switch "));

        if !in_fn && is_fn_start {
            count += 1;
            in_fn = true;
            fn_start = i;
            brace_depth = 0;
        }

        if in_fn {
            brace_depth += line.chars().filter(|&c| c == '{').count();
            brace_depth = brace_depth.saturating_sub(line.chars().filter(|&c| c == '}').count());

            if brace_depth == 0 && line.contains('}') {
                let fn_len = i - fn_start + 1;
                max_len = max_len.max(fn_len);
                in_fn = false;
            }
        }
    }

    (count, max_len)
}

pub(super) fn count_python_functions(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut fn_start = 0;
    let mut fn_indent = 0;
    let mut in_fn = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            if in_fn {
                let fn_len = i - fn_start;
                max_len = max_len.max(fn_len);
            }
            count += 1;
            in_fn = true;
            fn_start = i;
            fn_indent = line.len() - line.trim_start().len();
        } else if in_fn && !trimmed.is_empty() && !trimmed.starts_with('#') {
            let current_indent = line.len() - line.trim_start().len();
            if current_indent <= fn_indent
                && !trimmed.starts_with("def ")
                && !trimmed.starts_with("async def ")
            {
                let fn_len = i - fn_start;
                max_len = max_len.max(fn_len);
                in_fn = false;
            }
        }
    }

    if in_fn {
        let fn_len = lines.len() - fn_start;
        max_len = max_len.max(fn_len);
    }

    (count, max_len)
}

fn count_go_functions(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut in_fn = false;
    let mut fn_start = 0;
    let mut brace_depth = 0usize;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if !in_fn && trimmed.starts_with("func ") {
            count += 1;
            in_fn = true;
            fn_start = i;
            brace_depth = 0;
        }

        if in_fn {
            brace_depth += line.chars().filter(|&c| c == '{').count();
            brace_depth = brace_depth.saturating_sub(line.chars().filter(|&c| c == '}').count());

            if brace_depth == 0 && line.contains('}') {
                let fn_len = i - fn_start + 1;
                max_len = max_len.max(fn_len);
                in_fn = false;
            }
        }
    }

    (count, max_len)
}

fn count_c_style_functions(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut in_fn = false;
    let mut fn_start = 0;
    let mut brace_depth = 0usize;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        let looks_like_fn = trimmed.ends_with(") {")
            || (trimmed.ends_with(')') && i + 1 < lines.len() && lines[i + 1].trim() == "{");

        let is_control = trimmed.starts_with("if ")
            || trimmed.starts_with("if(")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("while(")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("for(")
            || trimmed.starts_with("switch ")
            || trimmed.starts_with("switch(");

        if !in_fn && looks_like_fn && !is_control {
            count += 1;
            in_fn = true;
            fn_start = i;
            brace_depth = 0;
        }

        if in_fn {
            brace_depth += line.chars().filter(|&c| c == '{').count();
            brace_depth = brace_depth.saturating_sub(line.chars().filter(|&c| c == '}').count());

            if brace_depth == 0 && line.contains('}') {
                let fn_len = i - fn_start + 1;
                max_len = max_len.max(fn_len);
                in_fn = false;
            }
        }
    }

    (count, max_len)
}

fn count_ruby_functions(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut fn_start = 0;
    let mut in_fn = false;
    let mut depth = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("def ") {
            if !in_fn {
                count += 1;
                in_fn = true;
                fn_start = i;
                depth = 1;
            } else {
                depth += 1;
            }
        } else if in_fn {
            if trimmed.starts_with("do")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("module ")
                || trimmed.starts_with("begin")
                || trimmed.starts_with("if ")
                || trimmed.starts_with("unless ")
                || trimmed.starts_with("case ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("until ")
                || trimmed.starts_with("for ")
            {
                depth += 1;
            }
            if trimmed == "end" || trimmed.starts_with("end ") {
                depth -= 1;
                if depth == 0 {
                    let fn_len = i - fn_start + 1;
                    max_len = max_len.max(fn_len);
                    in_fn = false;
                }
            }
        }
    }

    (count, max_len)
}
