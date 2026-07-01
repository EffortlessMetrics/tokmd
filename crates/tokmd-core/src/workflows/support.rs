//! Shared helpers for public workflow owner modules.

use std::path::{Path, PathBuf};

use anyhow::Result;
use tokmd_settings::ScanOptions;
use tokmd_types::{ChildIncludeMode, FileRow};

use crate::InMemoryFile;
use crate::settings::ScanSettings;

/// Convert `ScanSettings` to lower-tier scan options.
#[inline]
pub(crate) fn settings_to_scan_options(scan: &ScanSettings) -> ScanOptions {
    scan.options.clone()
}

pub(crate) fn scan_paths_or_current_dir(scan: &ScanSettings) -> Vec<PathBuf> {
    if scan.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        scan.paths.iter().map(PathBuf::from).collect()
    }
}

/// When exactly one scan root is provided, strip it from host file paths before
/// module-key aggregation so single-root scans match archive/virtual relative paths.
pub(crate) fn single_scan_root_strip_prefix(paths: &[PathBuf]) -> Option<&Path> {
    if paths.len() == 1 {
        paths.first().map(|path| path.as_path())
    } else {
        None
    }
}

pub(crate) fn deterministic_in_memory_scan_options(scan_opts: &ScanOptions) -> ScanOptions {
    let mut effective = scan_opts.clone();
    // Explicit in-memory inputs are authoritative; they should not depend on
    // host cwd config discovery or be filtered back out by hidden/exclude rules.
    effective.config = tokmd_types::ConfigMode::None;
    effective.hidden = true;
    effective.excluded.clear();
    effective
}

pub(crate) fn collect_pure_in_memory_rows(
    inputs: &[InMemoryFile],
    scan_opts: &ScanOptions,
    module_roots: &[String],
    module_depth: usize,
    children: ChildIncludeMode,
) -> Result<(Vec<PathBuf>, Vec<FileRow>)> {
    let paths = tokmd_scan::normalize_in_memory_paths(inputs)?;
    let config = tokmd_scan::config_from_scan_options(scan_opts);
    let row_inputs: Vec<tokmd_model::InMemoryRowInput<'_>> = paths
        .iter()
        .zip(inputs)
        .map(|(path, input)| {
            tokmd_model::InMemoryRowInput::new(path.as_path(), input.bytes.as_slice())
        })
        .collect();
    let rows = tokmd_model::collect_in_memory_file_rows(
        &row_inputs,
        module_roots,
        module_depth,
        children,
        &config,
    );
    Ok((paths, rows))
}

pub(crate) fn strip_virtual_export_prefix(
    rows: Vec<FileRow>,
    strip_prefix: &str,
    module_roots: &[String],
    module_depth: usize,
) -> Vec<FileRow> {
    rows.into_iter()
        .map(|mut row| {
            let normalized =
                tokmd_model::normalize_path(Path::new(&row.path), Some(Path::new(strip_prefix)));
            row.path = normalized.clone();
            row.module = tokmd_model::module_key(&normalized, module_roots, module_depth);
            row
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_scan_root_strip_prefix_uses_only_root() {
        let one = vec![PathBuf::from("/repo")];
        assert_eq!(
            single_scan_root_strip_prefix(&one),
            Some(Path::new("/repo"))
        );
        let many = vec![PathBuf::from("/a"), PathBuf::from("/b")];
        assert_eq!(single_scan_root_strip_prefix(&many), None);
    }

    #[test]
    fn settings_to_scan_options_preserves_values() {
        let scan = ScanSettings {
            paths: vec!["src".to_string()],
            options: ScanOptions {
                excluded: vec!["target".to_string()],
                hidden: true,
                no_ignore: true,
                ..Default::default()
            },
        };

        let opts = settings_to_scan_options(&scan);
        assert_eq!(opts.excluded, vec!["target"]);
        assert!(opts.hidden);
        assert!(opts.no_ignore);
    }
}
