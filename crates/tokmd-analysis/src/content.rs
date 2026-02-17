use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tokmd_analysis_types::{
    DuplicateGroup, DuplicateReport, DuplicationDensityReport, ImportEdge, ImportReport,
    ModuleDuplicationDensityRow, TodoReport, TodoTagRow,
};
use tokmd_types::{ExportData, FileKind, FileRow};

use crate::analysis::{AnalysisLimits, ImportGranularity};
use crate::util::{normalize_path, round_f64};

const DEFAULT_MAX_FILE_BYTES: u64 = 128 * 1024;
const IMPORT_MAX_LINES: usize = 200;

#[cfg(feature = "content")]
pub(crate) fn build_todo_report(
    root: &Path,
    files: &[PathBuf],
    limits: &AnalysisLimits,
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

#[cfg(feature = "content")]
pub(crate) fn build_duplicate_report(
    root: &Path,
    files: &[PathBuf],
    export: &ExportData,
    limits: &AnalysisLimits,
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

    let mut modules: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
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
    })
}

#[cfg(feature = "content")]
pub(crate) fn build_import_report(
    root: &Path,
    files: &[PathBuf],
    export: &ExportData,
    granularity: ImportGranularity,
    limits: &AnalysisLimits,
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
        if !is_import_lang(&row.lang) {
            continue;
        }
        let path = root.join(rel);
        let lines = match tokmd_content::read_lines(&path, IMPORT_MAX_LINES, per_file_limit) {
            Ok(lines) => lines,
            Err(_) => continue,
        };
        total_bytes += lines.iter().map(|l| l.len() as u64).sum::<u64>();
        let imports = parse_imports(&row.lang, &lines);
        if imports.is_empty() {
            continue;
        }
        let source = match granularity {
            ImportGranularity::Module => row.module.clone(),
            ImportGranularity::File => row.path.clone(),
        };
        for import in imports {
            let target = normalize_import_target(&import);
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

#[cfg(feature = "content")]
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

#[cfg(feature = "content")]
fn is_import_lang(lang: &str) -> bool {
    matches!(
        lang.to_lowercase().as_str(),
        "rust" | "javascript" | "typescript" | "python" | "go"
    )
}

#[cfg(feature = "content")]
fn parse_imports(lang: &str, lines: &[String]) -> Vec<String> {
    match lang.to_lowercase().as_str() {
        "rust" => parse_rust_imports(lines),
        "javascript" | "typescript" => parse_js_imports(lines),
        "python" => parse_py_imports(lines),
        "go" => parse_go_imports(lines),
        _ => vec![],
    }
}

#[cfg(feature = "content")]
fn parse_rust_imports(lines: &[String]) -> Vec<String> {
    let mut imports = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("use ")
            && let Some(rest) = trimmed.strip_prefix("use ")
        {
            let rest = rest.trim_end_matches(';').trim();
            let target = rest.split("::").next().unwrap_or(rest).to_string();
            imports.push(target);
        } else if trimmed.starts_with("mod ")
            && let Some(rest) = trimmed.strip_prefix("mod ")
        {
            let target = rest.trim_end_matches(';').trim().to_string();
            imports.push(target);
        }
    }
    imports
}

#[cfg(feature = "content")]
fn parse_js_imports(lines: &[String]) -> Vec<String> {
    let mut imports = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("import ")
            && let Some(target) = extract_quoted(trimmed)
        {
            imports.push(target);
        }
        if let Some(idx) = trimmed.find("require(")
            && let Some(target) = extract_quoted(&trimmed[idx..])
        {
            imports.push(target);
        }
    }
    imports
}

#[cfg(feature = "content")]
fn parse_py_imports(lines: &[String]) -> Vec<String> {
    let mut imports = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("import ")
            && let Some(rest) = trimmed.strip_prefix("import ")
        {
            let target = rest.split_whitespace().next().unwrap_or(rest).to_string();
            imports.push(target);
        } else if trimmed.starts_with("from ")
            && let Some(rest) = trimmed.strip_prefix("from ")
        {
            let target = rest.split_whitespace().next().unwrap_or(rest).to_string();
            imports.push(target);
        }
    }
    imports
}

#[cfg(feature = "content")]
fn parse_go_imports(lines: &[String]) -> Vec<String> {
    let mut imports = Vec::new();
    let mut in_block = false;
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("import (") {
            in_block = true;
            continue;
        }
        if in_block {
            if trimmed.starts_with(')') {
                in_block = false;
                continue;
            }
            if let Some(target) = extract_quoted(trimmed) {
                imports.push(target);
            }
            continue;
        }
        if trimmed.starts_with("import ")
            && let Some(target) = extract_quoted(trimmed)
        {
            imports.push(target);
        }
    }
    imports
}

#[cfg(feature = "content")]
fn extract_quoted(text: &str) -> Option<String> {
    let mut chars = text.chars();
    let mut quote = None;
    for c in chars.by_ref() {
        if c == '"' || c == '\'' {
            quote = Some(c);
            break;
        }
    }
    let quote = quote?;
    let mut out = String::new();
    for c in chars {
        if c == quote {
            break;
        }
        out.push(c);
    }
    if out.is_empty() { None } else { Some(out) }
}

#[cfg(feature = "content")]
fn normalize_import_target(target: &str) -> String {
    let trimmed = target.trim();
    if trimmed.starts_with('.') {
        return "local".to_string();
    }
    let trimmed = trimmed.trim_matches('"').trim_matches('\'');
    trimmed
        .split(['/', ':', '.'])
        .next()
        .unwrap_or(trimmed)
        .to_string()
}
