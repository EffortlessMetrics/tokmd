//! Writers for the map and selected code bundle handoff payloads.

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

        match ctx_file.policy {
            InclusionPolicy::Full => {
                bytes += write_full_file(&mut writer, ctx_file, &file_path, compress)?;
            }
            InclusionPolicy::HeadTail => {
                bytes += write_head_tail_file(&mut writer, ctx_file, &file_path, compress)?;
            }
            InclusionPolicy::Summary | InclusionPolicy::Skip => {
                bytes += write_skipped_file(&mut writer, ctx_file)?;
            }
        }
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
    let mut bytes = write_file_header(writer, &ctx_file.path)?;

    if compress {
        let file = File::open(file_path)
            .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line =
                line.with_context(|| format!("Failed to read file: {}", file_path.display()))?;
            if !line.trim().is_empty() {
                writeln!(writer, "{}", line)?;
                bytes += line.len() as u64 + 1;
            }
        }
        writeln!(writer)?;
        bytes += 1;
    } else {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
        writer.write_all(content.as_bytes())?;
        bytes += content.len() as u64;
        if !content.ends_with('\n') {
            writeln!(writer)?;
            bytes += 1;
        }
        writeln!(writer)?;
        bytes += 1;
    }

    Ok(bytes)
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

    writeln!(writer)?;
    bytes += 1;

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
    let header = format!("// === {path} ===\n");
    writer.write_all(header.as_bytes())?;
    Ok(header.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_types::{ChildIncludeMode, FileRow};

    #[test]
    fn map_jsonl_writes_parent_rows_only() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("map.jsonl");
        let export = ExportData {
            rows: vec![
                FileRow {
                    path: "src/lib.rs".to_string(),
                    module: "src".to_string(),
                    lang: "Rust".to_string(),
                    kind: FileKind::Parent,
                    code: 1,
                    comments: 0,
                    blanks: 0,
                    lines: 1,
                    bytes: 10,
                    tokens: 3,
                },
                FileRow {
                    path: "src/lib.rs:Markdown".to_string(),
                    module: "src".to_string(),
                    lang: "Markdown".to_string(),
                    kind: FileKind::Child,
                    code: 99,
                    comments: 0,
                    blanks: 0,
                    lines: 99,
                    bytes: 99,
                    tokens: 99,
                },
            ],
            module_roots: vec![],
            module_depth: 2,
            children: ChildIncludeMode::ParentsOnly,
        };

        let bytes = write_map_jsonl(&path, &export).expect("write map");
        let contents = fs::read_to_string(path).expect("read map");

        assert!(bytes > 0);
        assert_eq!(contents.lines().count(), 1);
        assert!(contents.contains("src/lib.rs"));
        assert!(!contents.contains("Markdown"));
    }
}
