//! Streaming writers for the file inventory and token-budgeted code bundle.

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tokmd_types::{ContextFileRow, ExportData, FileKind, InclusionPolicy};

pub(super) fn write_map_jsonl(path: &Path, export: &ExportData) -> Result<u64> {
    let file =
        File::create(path).with_context(|| format!("Failed to create {}", path.display()))?;
    let mut writer = std::io::BufWriter::new(file);
    let mut bytes: u64 = 0;

    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        let json = serde_json::to_string(row)?;
        writeln!(writer, "{}", json)?;
        bytes += json.len() as u64 + 1;
    }

    writer.flush()?;
    Ok(bytes)
}

pub(super) fn write_code_bundle(
    path: &Path,
    selected: &[ContextFileRow],
    compress: bool,
) -> Result<u64> {
    let file =
        File::create(path).with_context(|| format!("Failed to create {}", path.display()))?;
    let mut writer = std::io::BufWriter::new(file);
    let mut bytes: u64 = 0;

    for ctx_file in selected {
        let file_path = PathBuf::from(&ctx_file.path);
        if !file_path.exists() {
            continue;
        }

        bytes += match ctx_file.policy {
            InclusionPolicy::Full => write_full_file(&mut writer, ctx_file, &file_path, compress)?,
            InclusionPolicy::HeadTail => {
                write_head_tail_file(&mut writer, ctx_file, &file_path, compress)?
            }
            InclusionPolicy::Summary | InclusionPolicy::Skip => {
                write_skipped_file(&mut writer, ctx_file)?
            }
        };
    }

    writer.flush()?;
    Ok(bytes)
}

fn write_full_file(
    writer: &mut impl Write,
    ctx_file: &ContextFileRow,
    file_path: &Path,
    compress: bool,
) -> Result<u64> {
    let header_bytes = write_file_header(writer, &ctx_file.path)?;
    if compress {
        Ok(header_bytes + write_compressed_file(writer, file_path)? + write_blank_line(writer)?)
    } else {
        Ok(header_bytes + write_raw_file(writer, file_path)?)
    }
}

fn write_head_tail_file(
    writer: &mut impl Write,
    ctx_file: &ContextFileRow,
    file_path: &Path,
    compress: bool,
) -> Result<u64> {
    let mut bytes = write_file_header(writer, &ctx_file.path)?;
    let mut buf = Vec::new();
    crate::context_pack::write_head_tail(&mut buf, file_path, ctx_file, compress)?;
    writer.write_all(&buf)?;
    bytes += buf.len() as u64;
    bytes += write_blank_line(writer)?;
    Ok(bytes)
}

fn write_skipped_file(writer: &mut impl Write, ctx_file: &ContextFileRow) -> Result<u64> {
    let header = format!(
        "// === {} [skipped: {}] ===\n\n",
        ctx_file.path,
        ctx_file.policy_reason.as_deref().unwrap_or("policy")
    );
    writer.write_all(header.as_bytes())?;
    Ok(header.len() as u64)
}

fn write_file_header(writer: &mut impl Write, path: &str) -> Result<u64> {
    let header = format!("// === {} ===\n", path);
    writer.write_all(header.as_bytes())?;
    Ok(header.len() as u64)
}

fn write_compressed_file(writer: &mut impl Write, file_path: &Path) -> Result<u64> {
    let file = File::open(file_path)
        .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
    let reader = BufReader::new(file);
    let mut bytes = 0;
    for line in reader.lines() {
        let line = line.with_context(|| format!("Failed to read file: {}", file_path.display()))?;
        if !line.trim().is_empty() {
            writeln!(writer, "{}", line)?;
            bytes += line.len() as u64 + 1;
        }
    }
    Ok(bytes)
}

fn write_raw_file(writer: &mut impl Write, file_path: &Path) -> Result<u64> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
    writer.write_all(content.as_bytes())?;
    let mut bytes = content.len() as u64;
    if !content.ends_with('\n') {
        bytes += write_blank_line(writer)?;
    }
    bytes += write_blank_line(writer)?;
    Ok(bytes)
}

fn write_blank_line(writer: &mut impl Write) -> Result<u64> {
    writeln!(writer)?;
    Ok(1)
}
