pub(in crate::complexity) fn detect_fn_spans_python(lines: &[&str]) -> Vec<(usize, usize, String)> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            let name = extract_python_fn_name(trimmed);
            let base_indent = lines[i].len() - lines[i].trim_start().len();

            let mut start = i;
            {
                let mut k = i;
                while k > 0 {
                    let prev_line = lines[k - 1];
                    let prev_trimmed = prev_line.trim();

                    if prev_trimmed.is_empty() {
                        k -= 1;
                        continue;
                    }

                    if prev_trimmed.starts_with('#') {
                        k -= 1;
                        continue;
                    }

                    let prev_indent = prev_line.len() - prev_line.trim_start().len();
                    if prev_indent == base_indent && prev_trimmed.starts_with('@') {
                        start = k - 1;
                        k -= 1;
                    } else {
                        break;
                    }
                }
            }
            let mut end = i;
            let mut j = i + 1;
            while j < lines.len() {
                let lt = lines[j].trim();
                if lt.is_empty() || lt.starts_with('#') {
                    j += 1;
                    continue;
                }
                let indent = lines[j].len() - lines[j].trim_start().len();
                if indent <= base_indent {
                    break;
                }
                end = j;
                j += 1;
            }
            spans.push((start, end, name));
            i = end + 1;
        } else {
            i += 1;
        }
    }
    spans
}

fn extract_python_fn_name(line: &str) -> String {
    let keyword = if line.contains("async def ") {
        "async def "
    } else {
        "def "
    };
    if let Some(idx) = line.find(keyword) {
        let after = &line[idx + keyword.len()..];
        let name: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return name;
        }
    }
    "<unknown>".to_string()
}
