//! Rust source masking for lightweight function-complexity analysis.

/// Masks Rust source spans that should not contribute to complexity.
#[derive(Default)]
pub(super) struct RustCodeMask {
    in_string: bool,
    in_char: bool,
    block_comment_depth: usize,
    raw_string_hashes: Option<usize>,
}

impl RustCodeMask {
    pub(super) fn code_only_line(&mut self, line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut code = String::with_capacity(line.len());
        let mut i = 0;

        while i < chars.len() {
            if let Some(hashes) = self.raw_string_hashes {
                if raw_string_closes_at(&chars, i, hashes) {
                    self.raw_string_hashes = None;
                    i += 1 + hashes;
                } else {
                    i += 1;
                }
                continue;
            }

            if self.block_comment_depth > 0 {
                if starts_pair(&chars, i, '/', '*') {
                    self.block_comment_depth += 1;
                    i += 2;
                    continue;
                }
                if starts_pair(&chars, i, '*', '/') {
                    self.block_comment_depth -= 1;
                    i += 2;
                    continue;
                }
                i += 1;
                continue;
            }

            if !self.in_string && !self.in_char {
                if starts_pair(&chars, i, '/', '/') {
                    break;
                }
                if starts_pair(&chars, i, '/', '*') {
                    self.block_comment_depth = 1;
                    i += 2;
                    continue;
                }
                if let Some((opening_len, hashes)) = raw_string_opens_at(&chars, i) {
                    self.raw_string_hashes = Some(hashes);
                    i += opening_len;
                    continue;
                }
            }

            let c = chars[i];

            if !self.in_char && c == '"' && !is_escaped(&chars, i) {
                self.in_string = !self.in_string;
                i += 1;
                continue;
            }

            if !self.in_string && c == '\'' && !is_escaped(&chars, i) {
                if self.in_char {
                    self.in_char = false;
                    i += 1;
                    continue;
                }
                if starts_char_literal(&chars, i) {
                    self.in_char = true;
                    i += 1;
                    continue;
                }
            }

            if self.in_string || self.in_char {
                i += 1;
                continue;
            }

            code.push(c);
            i += 1;
        }

        code
    }
}

fn starts_pair(chars: &[char], i: usize, first: char, second: char) -> bool {
    chars.get(i) == Some(&first) && chars.get(i + 1) == Some(&second)
}

fn is_escaped(chars: &[char], i: usize) -> bool {
    if i == 0 {
        return false;
    }

    let mut slash_count = 0;
    let mut cursor = i;
    while cursor > 0 && chars[cursor - 1] == '\\' {
        slash_count += 1;
        cursor -= 1;
    }
    slash_count % 2 == 1
}

fn starts_char_literal(chars: &[char], i: usize) -> bool {
    matches!(
        (chars.get(i + 1), chars.get(i + 2), chars.get(i + 3)),
        (Some('\\'), Some(_), Some('\'')) | (Some(_), Some('\''), _)
    )
}

fn raw_string_opens_at(chars: &[char], i: usize) -> Option<(usize, usize)> {
    let mut cursor = match (chars.get(i), chars.get(i + 1)) {
        (Some('b'), Some('r')) => i + 2,
        (Some('r'), _) => i + 1,
        _ => return None,
    };

    let hash_start = cursor;
    while chars.get(cursor) == Some(&'#') {
        cursor += 1;
    }

    if chars.get(cursor) == Some(&'"') {
        let hashes = cursor - hash_start;
        Some((cursor - i + 1, hashes))
    } else {
        None
    }
}

fn raw_string_closes_at(chars: &[char], i: usize, hashes: usize) -> bool {
    chars.get(i) == Some(&'"') && (0..hashes).all(|offset| chars.get(i + 1 + offset) == Some(&'#'))
}
