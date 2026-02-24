use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
use tokmd_analysis_types::{
    DuplicateGroup, DuplicateReport, DuplicationDensityReport, ImportEdge, ImportReport,
    ModuleDuplicationDensityRow, TodoReport, TodoTagRow,
};
use tokmd_types::{ExportData, FileKind, FileRow};

use tokmd_analysis_util::normalize_path;
use tokmd_math::round_f64;

const DEFAULT_MAX_FILE_BYTES: u64 = 128 * 1024;
const IMPORT_MAX_LINES: usize = 200;

#[derive(Debug, Clone, Copy)]
pub enum ImportGranularity {
    Module,
    File,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ContentLimits {
    pub max_bytes: Option<u64>,
    pub max_file_bytes: Option<u64>,
}

pub fn build_todo_report(
    root: &Path,
    files: &[PathBuf],
    limits: &ContentLimits,
    total_code: usize,
) -> Result<TodoReport> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let tags = ["TODO", "FIXME", "HACK", "XXX"];
    let mut total_bytes = 0u64;
    let max_total = limits.max_bytes;
    let per_file_limit = limits.max_file_bytes.unwrap_or(DEFAULT_MAX_FILE_BYTES) as usize;

    for rel in files {
        if max_total.is_some_and(|limit| total_bytes >= limit) {
            break;
        }
        let path = root.join(rel);
        let bytes = tokmd_content::read_head(&path, per_file_limit)?;
        total_bytes += bytes.len() as u64;
        if !tokmd_content::is_text_like(&bytes) {
            continue;
        }
        let text = String::from_utf8_lossy(&bytes);
        for (tag, count) in tokmd_content::count_tags(&text, &tags) {
            *counts.entry(tag).or_insert(0) += count;
        }
    }

    let total: usize = counts.values().sum();
    let kloc = if total_code == 0 {
        0.0
    } else {
        total_code as f64 / 1000.0
    };
    let density = if kloc == 0.0 {
        0.0
    } else {
        round_f64(total as f64 / kloc, 2)
    };

    let tags = counts
        .into_iter()
        .map(|(tag, count)| TodoTagRow { tag, count })
        .collect();

    Ok(TodoReport {
        total,
        density_per_kloc: density,
        tags,
    })
}

pub fn build_duplicate_report(
    root: &Path,
    files: &[PathBuf],
    export: &ExportData,
    limits: &ContentLimits,
) -> Result<DuplicateReport> {
    let mut by_size: BTreeMap<u64, Vec<PathBuf>> = BTreeMap::new();
    let size_limit = limits.max_file_bytes;

    for rel in files {
        let size = std::fs::metadata(root.join(rel))
            .map(|m| m.len())
            .unwrap_or(0);
        if size_limit.is_some_and(|limit| size > limit) {
            continue;
        }
        by_size.entry(size).or_default().push(rel.clone());
    }

    let mut path_to_module: BTreeMap<String, String> = BTreeMap::new();
    let mut module_bytes: BTreeMap<String, u64> = BTreeMap::new();
    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        let normalized = normalize_path(&row.path, root);
        path_to_module.insert(normalized, row.module.clone());
        *module_bytes.entry(row.module.clone()).or_insert(0) += row.bytes as u64;
    }

    let mut groups: Vec<DuplicateGroup> = Vec::new();
    let mut wasted_bytes = 0u64;
    let mut duplicate_files = 0usize;
    let mut duplicated_bytes = 0u64;

    let mut module_duplicate_files: BTreeMap<String, usize> = BTreeMap::new();
    let mut module_wasted_files: BTreeMap<String, usize> = BTreeMap::new();
    let mut module_duplicated_bytes: BTreeMap<String, u64> = BTreeMap::new();
    let mut module_wasted_bytes: BTreeMap<String, u64> = BTreeMap::new();

    for (size, paths) in by_size {
        if paths.len() < 2 || size == 0 {
            continue;
        }
        let mut by_hash: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for rel in paths {
            let path = root.join(&rel);
            if let Ok(hash) = hash_file_full(&path) {
                by_hash
                    .entry(hash)
                    .or_default()
                    .push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
        for (hash, mut files) in by_hash {
            if files.len() < 2 {
                continue;
            }
            files.sort();
            wasted_bytes += (files.len() as u64 - 1) * size;

            for (idx, file) in files.iter().enumerate() {
                let module = path_to_module
                    .get(file)
                    .cloned()
                    .unwrap_or_else(|| "(unknown)".to_string());
                *module_duplicate_files.entry(module.clone()).or_insert(0) += 1;
                *module_duplicated_bytes.entry(module.clone()).or_insert(0) += size;
                duplicate_files += 1;
                duplicated_bytes += size;

                if idx > 0 {
                    *module_wasted_files.entry(module.clone()).or_insert(0) += 1;
                    *module_wasted_bytes.entry(module).or_insert(0) += size;
                }
            }

            groups.push(DuplicateGroup {
                hash,
                bytes: size,
                files,
            });
        }
    }

    groups.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.hash.cmp(&b.hash)));

    let mut modules: BTreeSet<String> = BTreeSet::new();
    modules.extend(module_duplicate_files.keys().cloned());
    modules.extend(module_wasted_files.keys().cloned());

    let mut by_module: Vec<ModuleDuplicationDensityRow> = modules
        .into_iter()
        .map(|module| {
            let duplicate_files = module_duplicate_files.get(&module).copied().unwrap_or(0);
            let wasted_files = module_wasted_files.get(&module).copied().unwrap_or(0);
            let duplicated_bytes = module_duplicated_bytes.get(&module).copied().unwrap_or(0);
            let wasted_bytes = module_wasted_bytes.get(&module).copied().unwrap_or(0);
            let module_total = module_bytes.get(&module).copied().unwrap_or(0);
            let density = if module_total == 0 {
                0.0
            } else {
                round_f64(wasted_bytes as f64 / module_total as f64, 4)
            };
            ModuleDuplicationDensityRow {
                module,
                duplicate_files,
                wasted_files,
                duplicated_bytes,
                wasted_bytes,
                module_bytes: module_total,
                density,
            }
        })
        .collect();
    by_module.sort_by(|a, b| {
        b.wasted_bytes
            .cmp(&a.wasted_bytes)
            .then_with(|| a.module.cmp(&b.module))
    });

    let total_codebase_bytes: u64 = module_bytes.values().sum();
    let wasted_pct_of_codebase = if total_codebase_bytes == 0 {
        0.0
    } else {
        round_f64(wasted_bytes as f64 / total_codebase_bytes as f64, 4)
    };
    let density = DuplicationDensityReport {
        duplicate_groups: groups.len(),
        duplicate_files,
        duplicated_bytes,
        wasted_bytes,
        wasted_pct_of_codebase,
        by_module,
    };

    Ok(DuplicateReport {
        groups,
        wasted_bytes,
        strategy: "exact-blake3".to_string(),
        density: Some(density),
        near: None,
    })
}

pub fn build_import_report(
    root: &Path,
    files: &[PathBuf],
    export: &ExportData,
    granularity: ImportGranularity,
    limits: &ContentLimits,
) -> Result<ImportReport> {
    let mut map: BTreeMap<String, &FileRow> = BTreeMap::new();
    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        let key = normalize_path(&row.path, root);
        map.insert(key, row);
    }

    let mut edges: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut total_bytes = 0u64;
    let max_total = limits.max_bytes;
    let per_file_limit = limits.max_file_bytes.unwrap_or(DEFAULT_MAX_FILE_BYTES) as usize;

    for rel in files {
        if max_total.is_some_and(|limit| total_bytes >= limit) {
            break;
        }
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let row = match map.get(&rel_str) {
            Some(r) => *r,
            None => continue,
        };
        if !tokmd_analysis_imports::supports_language(&row.lang) {
            continue;
        }
        let path = root.join(rel);
        let lines = match tokmd_content::read_lines(&path, IMPORT_MAX_LINES, per_file_limit) {
            Ok(lines) => lines,
            Err(_) => continue,
        };
        total_bytes += lines.iter().map(|l| l.len() as u64).sum::<u64>();
        let imports = tokmd_analysis_imports::parse_imports(&row.lang, &lines);
        if imports.is_empty() {
            continue;
        }
        let source = match granularity {
            ImportGranularity::Module => row.module.clone(),
            ImportGranularity::File => row.path.clone(),
        };
        for import in imports {
            let target = tokmd_analysis_imports::normalize_import_target(&import);
            let key = (source.clone(), target);
            *edges.entry(key).or_insert(0) += 1;
        }
    }

    let mut edge_rows: Vec<ImportEdge> = edges
        .into_iter()
        .map(|((from, to), count)| ImportEdge { from, to, count })
        .collect();
    edge_rows.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.from.cmp(&b.from)));

    Ok(ImportReport {
        granularity: match granularity {
            ImportGranularity::Module => "module".to_string(),
            ImportGranularity::File => "file".to_string(),
        },
        edges: edge_rows,
    })
}

fn hash_file_full(path: &Path) -> Result<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; 8192];
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}
