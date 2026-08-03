use anyhow::{Context, Result};

pub(crate) fn update_action_default_version(
    content: &str,
    new_version: &str,
) -> Result<(String, String)> {
    let mut result = String::with_capacity(content.len());
    let mut in_version_input = false;
    let mut old_version = None;

    for line in content.lines() {
        if line.trim() == "version:" && line.starts_with("  ") {
            in_version_input = true;
            result.push_str(line);
            result.push('\n');
            continue;
        }
        if in_version_input {
            let trimmed = line.trim();
            if let Some(value) = trimmed.strip_prefix("default:") {
                let (value, suffix) = value
                    .split_once(" #")
                    .map_or((value, ""), |(value, suffix)| (value, suffix));
                let old = value.trim().trim_matches(['\'', '"']).to_string();
                if old.is_empty() {
                    anyhow::bail!("inputs.version.default in action.yml is empty");
                }
                old_version = Some(old);
                let indent_end = line.len() - line.trim_start().len();
                let indent = line
                    .get(..indent_end)
                    .context("action.yml default indentation is outside the current line")?;
                if suffix.is_empty() {
                    result.push_str(&format!("{indent}default: '{new_version}'\n"));
                } else {
                    result.push_str(&format!("{indent}default: '{new_version}' #{suffix}\n"));
                }
                in_version_input = false;
                continue;
            }
            if line.starts_with("  ") && !line.starts_with("    ") {
                in_version_input = false;
            }
        }
        result.push_str(line);
        result.push('\n');
    }

    let old_version = old_version.context("Missing inputs.version.default in action.yml")?;
    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }
    Ok((result, old_version))
}

pub(crate) fn extract_action_default_version(content: &str) -> Option<&str> {
    let mut in_version_input = false;
    for line in content.lines() {
        if line.trim() == "version:" && line.starts_with("  ") {
            in_version_input = true;
            continue;
        }
        if !in_version_input {
            continue;
        }
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("default:") {
            let value = value
                .split_once(" #")
                .map_or(value, |(value, _)| value)
                .trim();
            return Some(value.trim_matches(['\'', '"']));
        }
        if line.starts_with("  ") && !line.starts_with("    ") {
            in_version_input = false;
        }
    }
    None
}
