use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tokmd_analysis_types::{EntropyClass, EntropyFinding, EntropyReport};
use tokmd_types::{ExportData, FileKind, FileRow};

use crate::analysis::AnalysisLimits;
use crate::util::normalize_path;

const DEFAULT_SAMPLE_BYTES: usize = 1024;
const MAX_SUSPECTS: usize = 50;

pub(crate) fn build_entropy_report(
    root: &Path,
    files: &[PathBuf],
    export: &ExportData,
    limits: &AnalysisLimits,
) -> Result<EntropyReport> {
    let mut row_map: BTreeMap<String, &FileRow> = BTreeMap::new();
    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        row_map.insert(normalize_path(&row.path, root), row);
    }

    let mut suspects = Vec::new();
    let mut total_bytes = 0u64;
    let max_total = limits.max_bytes;
    let per_file_limit = limits
        .max_file_bytes
        .unwrap_or(DEFAULT_SAMPLE_BYTES as u64) as usize;

    for rel in files {
        if max_total.is_some_and(|max| total_bytes >= max) {
            break;
        }
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let module = row_map
            .get(&rel_str)
            .map(|r| r.module.clone())
            .unwrap_or_else(|| "(unknown)".to_string());

        let path = root.join(rel);
        let bytes = tokmd_content::read_head_tail(&path, per_file_limit)?;
        total_bytes += bytes.len() as u64;
        if bytes.is_empty() {
            continue;
        }
        let entropy = tokmd_content::entropy_bits_per_byte(&bytes);
        let class = classify_entropy(entropy);
        if class != EntropyClass::Normal {
            suspects.push(EntropyFinding {
                path: rel_str,
                module,
                entropy_bits_per_byte: entropy,
                sample_bytes: bytes.len() as u32,
                class,
            });
        }
    }

    suspects.sort_by(|a, b| {
        b.entropy_bits_per_byte
            .partial_cmp(&a.entropy_bits_per_byte)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });
    suspects.truncate(MAX_SUSPECTS);

    Ok(EntropyReport { suspects })
}

fn classify_entropy(entropy: f32) -> EntropyClass {
    if entropy > 7.5 {
        EntropyClass::High
    } else if entropy >= 6.5 {
        EntropyClass::Suspicious
    } else if entropy < 2.0 {
        EntropyClass::Low
    } else {
        EntropyClass::Normal
    }
}

#[cfg(all(test, feature = "content", feature = "walk"))]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use tokmd_config::ChildIncludeMode;
    use tokmd_types::{ExportData, FileKind, FileRow};

    fn export_for_paths(paths: &[&str]) -> ExportData {
        let rows = paths
            .iter()
            .map(|p| FileRow {
                path: (*p).to_string(),
                module: "(root)".to_string(),
                lang: "Text".to_string(),
                kind: FileKind::Parent,
                code: 1,
                comments: 0,
                blanks: 0,
                lines: 1,
                bytes: 10,
                tokens: 2,
            })
            .collect();
        ExportData {
            rows,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        }
    }

    fn write_repeated(path: &Path, byte: u8, len: usize) {
        let data = vec![byte; len];
        fs::write(path, data).unwrap();
    }

    fn write_pseudorandom(path: &Path, len: usize) {
        let mut data = Vec::with_capacity(len);
        let mut x = 0x12345678u32;
        for _ in 0..len {
            x = x.wrapping_mul(1664525).wrapping_add(1013904223);
            data.push((x & 0xFF) as u8);
        }
        fs::write(path, data).unwrap();
    }

    #[test]
    fn detects_low_and_high_entropy() {
        let dir = tempdir().unwrap();
        let low = dir.path().join("low.txt");
        let high = dir.path().join("high.bin");
        write_repeated(&low, b'A', 1024);
        write_pseudorandom(&high, 1024);

        let export = export_for_paths(&["low.txt", "high.bin"]);
        let files = vec![PathBuf::from("low.txt"), PathBuf::from("high.bin")];
        let report = build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default())
            .unwrap();

        assert!(report
            .suspects
            .iter()
            .any(|f| f.path == "low.txt" && f.class == EntropyClass::Low));
        assert!(report
            .suspects
            .iter()
            .any(|f| f.path == "high.bin" && f.class == EntropyClass::High));
    }
}
