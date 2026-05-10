use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tokmd_analysis_types::{ApiExportItem, ApiSurfaceReport, LangApiSurface, ModuleApiRow};
use tokmd_types::{ExportData, FileKind, FileRow};

use tokmd_analysis_types::{AnalysisLimits, normalize_path};

#[cfg(test)]
#[path = "tests.rs"]
mod moved_tests;

mod symbols;

const DEFAULT_MAX_FILE_BYTES: u64 = 128 * 1024;
const MAX_TOP_EXPORTERS: usize = 20;
const MAX_BY_MODULE: usize = 50;

// -------
// Main
// -------

/// Build the API surface report by scanning source files for public/internal symbols.
pub(crate) fn build_api_surface_report(
    root: &Path,
    files: &[PathBuf],
    export: &ExportData,
    limits: &AnalysisLimits,
) -> Result<ApiSurfaceReport> {
    // Build lookup from normalized path -> FileRow
    let mut row_map: BTreeMap<String, &FileRow> = BTreeMap::new();
    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        row_map.insert(normalize_path(&row.path, root), row);
    }

    let per_file_limit = limits.max_file_bytes.unwrap_or(DEFAULT_MAX_FILE_BYTES) as usize;
    let mut total_bytes = 0u64;

    // Accumulators
    let mut total_items = 0usize;
    let mut public_items = 0usize;
    let mut internal_items = 0usize;
    let mut documented_public = 0usize;

    // Per-language accumulators
    let mut lang_totals: BTreeMap<&str, (usize, usize, usize)> = BTreeMap::new(); // (total, public, internal)

    // Per-module accumulators
    let mut module_totals: BTreeMap<&str, (usize, usize)> = BTreeMap::new(); // (total, public)

    // Top exporters
    let mut exporters: Vec<ApiExportItem> = Vec::new();

    for rel in files {
        if limits.max_bytes.is_some_and(|limit| total_bytes >= limit) {
            break;
        }

        let rel_str = normalize_path(&rel.to_string_lossy(), root);
        let row = match row_map.get(&rel_str) {
            Some(r) => *r,
            None => continue,
        };

        if !symbols::is_api_surface_lang(&row.lang) {
            continue;
        }

        let path = root.join(rel);
        let bytes = match crate::content::io::read_head(&path, per_file_limit) {
            Ok(b) => b,
            Err(_) => continue,
        };
        total_bytes += bytes.len() as u64;

        if !crate::content::io::is_text_like(&bytes) {
            continue;
        }

        let text = String::from_utf8_lossy(&bytes);
        let symbols = symbols::extract_symbols(&row.lang, &text);

        if symbols.is_empty() {
            continue;
        }

        let file_public: usize = symbols.iter().filter(|s| s.is_public).count();
        let file_internal: usize = symbols.iter().filter(|s| !s.is_public).count();
        let file_documented: usize = symbols
            .iter()
            .filter(|s| s.is_public && s.is_documented)
            .count();
        let file_total = symbols.len();

        total_items += file_total;
        public_items += file_public;
        internal_items += file_internal;
        documented_public += file_documented;

        // Per-language
        let entry = lang_totals.entry(row.lang.as_str()).or_insert((0, 0, 0));
        entry.0 += file_total;
        entry.1 += file_public;
        entry.2 += file_internal;

        // Per-module
        let mod_entry = module_totals.entry(row.module.as_str()).or_insert((0, 0));
        mod_entry.0 += file_total;
        mod_entry.1 += file_public;

        // Track top exporters
        if file_public > 0 {
            exporters.push(ApiExportItem {
                path: rel_str,
                lang: row.lang.clone(),
                public_items: file_public,
                total_items: file_total,
            });
        }
    }

    // Build per-language map
    let by_language: BTreeMap<String, LangApiSurface> = lang_totals
        .into_iter()
        .map(|(lang, (total, public, internal))| {
            let public_ratio = if total == 0 {
                0.0
            } else {
                round_f64(public as f64 / total as f64, 4)
            };
            (
                lang.to_owned(),
                LangApiSurface {
                    total_items: total,
                    public_items: public,
                    internal_items: internal,
                    public_ratio,
                },
            )
        })
        .collect();

    // Build per-module vec, sorted by total items descending
    let mut by_module: Vec<ModuleApiRow> = module_totals
        .into_iter()
        .map(|(module, (total, public))| {
            let public_ratio = if total == 0 {
                0.0
            } else {
                round_f64(public as f64 / total as f64, 4)
            };
            ModuleApiRow {
                module: module.to_owned(),
                total_items: total,
                public_items: public,
                public_ratio,
            }
        })
        .collect();
    by_module.sort_by(|a, b| {
        b.total_items
            .cmp(&a.total_items)
            .then_with(|| a.module.cmp(&b.module))
    });
    by_module.truncate(MAX_BY_MODULE);

    // Sort top exporters by public_items descending, then by path
    exporters.sort_by(|a, b| {
        b.public_items
            .cmp(&a.public_items)
            .then_with(|| a.path.cmp(&b.path))
    });
    exporters.truncate(MAX_TOP_EXPORTERS);

    let public_ratio = if total_items == 0 {
        0.0
    } else {
        round_f64(public_items as f64 / total_items as f64, 4)
    };

    let documented_ratio = if public_items == 0 {
        0.0
    } else {
        round_f64(documented_public as f64 / public_items as f64, 4)
    };

    Ok(ApiSurfaceReport {
        total_items,
        public_items,
        internal_items,
        public_ratio,
        documented_ratio,
        by_language,
        by_module,
        top_exporters: exporters,
    })
}

fn round_f64(val: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (val * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------
    // round_f64
    // -------

    #[test]
    fn test_round() {
        assert_eq!(round_f64(0.12345, 4), 0.1235);
        assert_eq!(round_f64(0.5, 0), 1.0);
        assert_eq!(round_f64(1.0, 4), 1.0);
    }

    #[test]
    fn test_round_zero() {
        assert_eq!(round_f64(0.0, 4), 0.0);
    }

    #[test]
    fn test_round_small_fraction() {
        assert_eq!(round_f64(0.3333, 2), 0.33);
        assert_eq!(round_f64(0.6667, 2), 0.67);
    }
}
