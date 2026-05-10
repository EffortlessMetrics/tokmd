//! Context packing helpers for LLM context window optimization.

use std::io::Write;
use std::path::Path;

use tokmd_types::ContextFileRow;

mod budget;
mod select;

pub use budget::parse_budget;
pub use select::{SelectOptions, SelectResult, select_files_with_options};

/// Write head and tail lines of a file.
///
/// Computes target lines from effective_tokens / (tokens / max(1, lines)),
/// splits 60% head / 40% tail, and emits with an omission separator.
pub fn write_head_tail<W: Write>(
    w: &mut W,
    path: &Path,
    file: &ContextFileRow,
    compress: bool,
) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;

    let all_lines: Vec<&str> = content.lines().collect();
    let total_lines = all_lines.len();

    if total_lines == 0 {
        return Ok(());
    }

    // Compute target line count from effective tokens
    let eff = file.effective_tokens.unwrap_or(file.tokens);
    let tpl = file.tokens as f64 / total_lines.max(1) as f64;
    let target_lines = if tpl > 0.0 {
        (eff as f64 / tpl).ceil() as usize
    } else {
        total_lines
    };

    if target_lines >= total_lines {
        // No need to truncate - write full content
        for line in &all_lines {
            if compress && line.trim().is_empty() {
                continue;
            }
            writeln!(w, "{line}")?;
        }
        return Ok(());
    }

    let head_count = (target_lines as f64 * 0.6).ceil() as usize;
    let tail_count = target_lines.saturating_sub(head_count);
    let omitted = total_lines.saturating_sub(head_count + tail_count);

    // Head
    for line in all_lines.iter().take(head_count) {
        if compress && line.trim().is_empty() {
            continue;
        }
        writeln!(w, "{line}")?;
    }

    // Separator
    if omitted > 0 {
        writeln!(w, "// ... [{omitted} lines omitted] ...")?;
    }

    // Tail
    let tail_start = total_lines.saturating_sub(tail_count);
    for line in all_lines.iter().skip(tail_start) {
        if compress && line.trim().is_empty() {
            continue;
        }
        writeln!(w, "{line}")?;
    }

    Ok(())
}
