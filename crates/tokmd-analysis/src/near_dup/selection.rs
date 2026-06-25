use std::collections::BTreeMap;

use anyhow::Result;
use globset::{Glob, GlobSetBuilder};

use tokmd_analysis_types::NearDupScope;
use tokmd_types::{ExportData, FileKind, FileRow};

use super::NearDupLimits;

pub(super) struct SelectedFiles<'a> {
    pub(super) files: Vec<&'a FileRow>,
    pub(super) eligible_files: usize,
    pub(super) files_skipped: usize,
    pub(super) excluded_by_pattern: usize,
    pub(super) max_file_bytes: u64,
}

/// Select parent files eligible for near-duplicate fingerprinting.
pub(super) fn select_files<'a>(
    export: &'a ExportData,
    max_files: usize,
    limits: &NearDupLimits,
    exclude_patterns: &[String],
) -> Result<SelectedFiles<'a>> {
    let _total_budget = limits.max_bytes;
    let max_file_bytes = limits.max_file_bytes.unwrap_or(512_000);
    let glob_set = build_glob_set(exclude_patterns)?;

    let mut files: Vec<&FileRow> = export
        .rows
        .iter()
        .filter(|row| row.kind == FileKind::Parent)
        .filter(|row| (row.bytes as u64) <= max_file_bytes)
        .collect();

    let excluded_by_pattern = if let Some(ref glob_set) = glob_set {
        let before = files.len();
        files.retain(|row| !glob_set.is_match(&row.path));
        before - files.len()
    } else {
        0
    };

    files.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.path.cmp(&b.path)));

    let eligible_files = files.len();
    let files_skipped = if files.len() > max_files {
        let skipped = files.len() - max_files;
        files.truncate(max_files);
        skipped
    } else {
        0
    };

    Ok(SelectedFiles {
        files,
        eligible_files,
        files_skipped,
        excluded_by_pattern,
        max_file_bytes,
    })
}

fn build_glob_set(patterns: &[String]) -> Result<Option<globset::GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }

    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(Some(builder.build()?))
}

/// Partition file indices by the requested near-duplicate comparison scope.
pub(super) fn partition_files(files: &[&FileRow], scope: NearDupScope) -> Vec<Vec<usize>> {
    match scope {
        NearDupScope::Global => vec![(0..files.len()).collect()],
        NearDupScope::Module => partition_by(files, |row| &row.module),
        NearDupScope::Lang => partition_by(files, |row| &row.lang),
    }
}

fn partition_by<'a, F>(files: &[&'a FileRow], key_for: F) -> Vec<Vec<usize>>
where
    F: Fn(&'a FileRow) -> &'a str,
{
    let mut map: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (index, row) in files.iter().enumerate() {
        map.entry(key_for(row)).or_default().push(index);
    }
    map.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_types::ChildIncludeMode;

    fn row(path: &str, module: &str, lang: &str, code: usize, bytes: usize) -> FileRow {
        FileRow {
            path: path.to_string(),
            lang: lang.to_string(),
            module: module.to_string(),
            code,
            comments: 0,
            blanks: 0,
            lines: code,
            bytes,
            tokens: code,
            kind: FileKind::Parent,
        }
    }

    fn export(rows: Vec<FileRow>) -> ExportData {
        ExportData {
            rows,
            module_roots: Vec::new(),
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        }
    }

    #[test]
    fn select_files_sorts_by_code_then_path_and_caps_count() {
        let export = export(vec![
            row("b.rs", "src", "Rust", 20, 100),
            row("a.rs", "src", "Rust", 20, 100),
            row("c.rs", "src", "Rust", 5, 100),
        ]);

        let selected = select_files(&export, 2, &NearDupLimits::default(), &[]).unwrap();

        assert_eq!(selected.eligible_files, 3);
        assert_eq!(selected.files_skipped, 1);
        assert_eq!(selected.files[0].path, "a.rs");
        assert_eq!(selected.files[1].path, "b.rs");
    }

    #[test]
    fn select_files_excludes_patterns_before_capping() {
        let export = export(vec![
            row("src/a.rs", "src", "Rust", 30, 100),
            row("vendor/a.rs", "vendor", "Rust", 25, 100),
            row("src/b.rs", "src", "Rust", 20, 100),
        ]);

        let selected = select_files(
            &export,
            1,
            &NearDupLimits::default(),
            &["vendor/**".to_string()],
        )
        .unwrap();

        assert_eq!(selected.excluded_by_pattern, 1);
        assert_eq!(selected.eligible_files, 2);
        assert_eq!(selected.files_skipped, 1);
        assert_eq!(selected.files[0].path, "src/a.rs");
    }

    #[test]
    fn partition_files_groups_by_scope_deterministically() {
        let rows = [
            row("src/b.rs", "src", "Rust", 20, 100),
            row("tests/a.rs", "tests", "Rust", 20, 100),
            row("web/app.js", "web", "JavaScript", 20, 100),
        ];
        let files: Vec<&FileRow> = rows.iter().collect();

        assert_eq!(
            partition_files(&files, NearDupScope::Global),
            vec![vec![0, 1, 2]]
        );
        assert_eq!(
            partition_files(&files, NearDupScope::Module),
            vec![vec![0], vec![1], vec![2]]
        );
        assert_eq!(
            partition_files(&files, NearDupScope::Lang),
            vec![vec![2], vec![0, 1]]
        );
    }
}
