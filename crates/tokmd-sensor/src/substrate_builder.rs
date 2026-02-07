//! Substrate builder: runs a tokei scan once and builds a `RepoSubstrate`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;
use tokmd_settings::ScanOptions;
use tokmd_substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};
use tokmd_types::ChildIncludeMode;

/// Build a `RepoSubstrate` from a scan of the given repo root.
///
/// This function runs tokei once, aggregates the results, and optionally
/// marks files that appear in the given diff range.
pub fn build_substrate(
    repo_root: &str,
    scan_options: &ScanOptions,
    module_roots: &[String],
    module_depth: usize,
    diff_range: Option<DiffRange>,
) -> Result<RepoSubstrate> {
    let paths = vec![PathBuf::from(repo_root)];

    // Run tokei scan
    let languages = tokmd_scan::scan(&paths, scan_options)?;

    // Build file rows using the model layer
    let file_rows = tokmd_model::collect_file_rows(
        &languages,
        module_roots,
        module_depth,
        ChildIncludeMode::ParentsOnly,
        Some(std::path::Path::new(repo_root)),
    );

    // Normalize changed_files through the same path normalization used for file rows,
    // so both sides use identical path representation regardless of scan/git root differences.
    let strip_prefix = std::path::Path::new(repo_root);
    let normalized_changed: Vec<String> = diff_range
        .as_ref()
        .map(|dr| {
            dr.changed_files
                .iter()
                .map(|s| tokmd_model::normalize_path(std::path::Path::new(s), Some(strip_prefix)))
                .collect()
        })
        .unwrap_or_default();
    let changed_set: std::collections::HashSet<&str> =
        normalized_changed.iter().map(|s| s.as_str()).collect();

    // Convert file rows to substrate files
    let files: Vec<SubstrateFile> = file_rows
        .iter()
        .map(|row| SubstrateFile {
            path: row.path.clone(),
            lang: row.lang.clone(),
            code: row.code,
            lines: row.lines,
            bytes: row.bytes,
            tokens: row.tokens,
            module: row.module.clone(),
            in_diff: changed_set.contains(row.path.as_str()),
        })
        .collect();

    // Aggregate per-language summary
    let mut lang_summary: BTreeMap<String, LangSummary> = BTreeMap::new();
    for f in &files {
        let entry = lang_summary.entry(f.lang.clone()).or_insert(LangSummary {
            files: 0,
            code: 0,
            lines: 0,
            bytes: 0,
            tokens: 0,
        });
        entry.files += 1;
        entry.code += f.code;
        entry.lines += f.lines;
        entry.bytes += f.bytes;
        entry.tokens += f.tokens;
    }

    // Compute totals
    let total_tokens: usize = files.iter().map(|f| f.tokens).sum();
    let total_bytes: usize = files.iter().map(|f| f.bytes).sum();
    let total_code_lines: usize = files.iter().map(|f| f.code).sum();

    Ok(RepoSubstrate {
        repo_root: repo_root.to_string(),
        files,
        lang_summary,
        diff_range,
        total_tokens,
        total_bytes,
        total_code_lines,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_settings::ScanOptions;

    #[test]
    fn build_substrate_scans_self() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let substrate = build_substrate(
            &format!("{}/src", manifest_dir),
            &ScanOptions::default(),
            &[],
            2,
            None,
        )
        .unwrap();

        assert!(!substrate.files.is_empty(), "should find some files");
        assert!(
            substrate.lang_summary.contains_key("Rust"),
            "should find Rust"
        );
        assert!(substrate.total_code_lines > 0);
        assert!(substrate.diff_range.is_none());
    }

    #[test]
    fn build_substrate_with_diff_range() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let scan_root = format!("{}/src", manifest_dir);
        // After strip_prefix, file rows have paths like "lib.rs", "substrate_builder.rs".
        // Provide changed_files relative to scan_root so normalization produces matching paths.
        let diff = DiffRange {
            base: "main".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec![format!("{}/lib.rs", scan_root)],
            commit_count: 1,
            insertions: 5,
            deletions: 2,
        };
        let substrate = build_substrate(
            &scan_root,
            &ScanOptions::default(),
            &[],
            2,
            Some(diff),
        )
        .unwrap();

        assert!(substrate.diff_range.is_some());
        let diff_files: Vec<&str> = substrate
            .files
            .iter()
            .filter(|f| f.in_diff)
            .map(|f| f.path.as_str())
            .collect();
        assert!(
            !diff_files.is_empty(),
            "at least one file should be marked in_diff"
        );
        assert!(
            diff_files.contains(&"lib.rs"),
            "lib.rs should be marked in_diff, got: {:?}",
            diff_files
        );
    }
}
