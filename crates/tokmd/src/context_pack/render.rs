//! Bundle text rendering helpers for context and handoff output.

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use tokmd_types::{ContextFileRow, InclusionPolicy};

use crate::cli;

/// A writer wrapper that counts bytes written.
pub(crate) struct CountingWriter<W: Write> {
    inner: W,
    bytes: u64,
}

impl<W: Write> CountingWriter<W> {
    pub(crate) fn new(inner: W) -> Self {
        Self { inner, bytes: 0 }
    }

    pub(crate) fn bytes(&self) -> u64 {
        self.bytes
    }
}

impl<W: Write> Write for CountingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.bytes += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Format list output as a markdown table.
pub(crate) fn format_list_output(
    selected: &[ContextFileRow],
    budget: usize,
    used_tokens: usize,
    utilization: f64,
    strategy: cli::ContextStrategy,
) -> String {
    let mut out = String::new();
    out.push_str("# Context Pack\n\n");
    out.push_str(&format!("Budget: {} tokens\n", budget));
    out.push_str(&format!(
        "Used: {} tokens ({:.1}%)\n",
        used_tokens, utilization
    ));
    out.push_str(&format!("Files: {}\n", selected.len()));
    out.push_str(&format!("Strategy: {:?}\n\n", strategy));
    out.push_str("|Path|Module|Lang|Tokens|Code|\n");
    out.push_str("|---|---|---|---:|---:|\n");
    for file in selected {
        out.push_str(&format!(
            "|{}|{}|{}|{}|{}|\n",
            file.path, file.module, file.lang, file.tokens, file.code
        ));
    }
    out
}

/// Write bundle output (concatenated file contents) directly to a writer.
///
/// Streams file content to avoid loading the entire bundle into memory and
/// dispatches based on file inclusion policy (Full / HeadTail / Skip).
pub(crate) fn write_bundle_output<W: Write>(
    w: &mut W,
    selected: &[ContextFileRow],
    compress: bool,
) -> anyhow::Result<()> {
    for file in selected {
        let path = PathBuf::from(&file.path);
        if !path.exists() {
            continue;
        }

        match file.policy {
            InclusionPolicy::Full => {
                writeln!(w, "// === {} ===", file.path)?;

                if compress {
                    let f = File::open(&path)
                        .with_context(|| format!("Failed to open file: {}", path.display()))?;
                    let reader = BufReader::new(f);
                    for line in reader.lines() {
                        let line = line
                            .with_context(|| format!("Failed to read file: {}", path.display()))?;
                        if !line.trim().is_empty() {
                            writeln!(w, "{line}")?;
                        }
                    }
                    writeln!(w)?;
                } else {
                    let mut f = File::open(&path)
                        .with_context(|| format!("Failed to open file: {}", path.display()))?;
                    let mut buf = [0u8; 16 * 1024];
                    let mut last: Option<u8> = None;
                    loop {
                        let n = f.read(&mut buf)?;
                        if n == 0 {
                            break;
                        }
                        last = Some(buf[n - 1]);
                        w.write_all(&buf[..n])?;
                    }
                    if last != Some(b'\n') {
                        w.write_all(b"\n")?;
                    }
                    w.write_all(b"\n")?;
                }
            }
            InclusionPolicy::HeadTail => {
                writeln!(w, "// === {} ===", file.path)?;
                write_head_tail(w, &path, file, compress)?;
                writeln!(w)?;
            }
            InclusionPolicy::Summary | InclusionPolicy::Skip => {
                writeln!(
                    w,
                    "// === {} [skipped: {}] ===",
                    file.path,
                    file.policy_reason.as_deref().unwrap_or("policy")
                )?;
                writeln!(w)?;
            }
        }
    }
    Ok(())
}

/// Write head and tail lines of a file.
///
/// Computes target lines from effective_tokens / (tokens / max(1, lines)),
/// splits 60% head / 40% tail, and emits with an omission separator.
pub(crate) fn write_head_tail<W: Write>(
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

    // Compute target line count from effective tokens.
    let eff = file.effective_tokens.unwrap_or(file.tokens);
    let tpl = file.tokens as f64 / total_lines.max(1) as f64;
    let target_lines = if tpl > 0.0 {
        (eff as f64 / tpl).ceil() as usize
    } else {
        total_lines
    };

    if target_lines >= total_lines {
        // No need to truncate - write full content.
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

    // Head.
    for line in all_lines.iter().take(head_count) {
        if compress && line.trim().is_empty() {
            continue;
        }
        writeln!(w, "{line}")?;
    }

    // Separator.
    if omitted > 0 {
        writeln!(w, "// ... [{omitted} lines omitted] ...")?;
    }

    // Tail.
    let tail_start = total_lines.saturating_sub(tail_count);
    for line in all_lines.iter().skip(tail_start) {
        if compress && line.trim().is_empty() {
            continue;
        }
        writeln!(w, "{line}")?;
    }

    Ok(())
}
