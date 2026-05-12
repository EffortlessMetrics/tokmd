//! Validated scan root handling and caller-facing report path rebasing.

use anyhow::Result;
use std::path::{Path, PathBuf};
use tokei::Languages;

use crate::path::ValidatedRoot;

pub(crate) fn validated_scan_roots(paths: &[PathBuf]) -> Result<Vec<ValidatedRoot>> {
    paths
        .iter()
        .map(ValidatedRoot::new)
        .collect::<std::result::Result<_, _>>()
        .map_err(Into::into)
}

pub(crate) fn rebase_report_paths(languages: &mut Languages, roots: &[ValidatedRoot]) {
    for language in languages.values_mut() {
        for report in &mut language.reports {
            report.name = rebase_report_path(&report.name, roots);
        }
        for reports in language.children.values_mut() {
            for report in reports {
                report.name = rebase_report_path(&report.name, roots);
            }
        }
    }
}

fn rebase_report_path(path: &Path, roots: &[ValidatedRoot]) -> PathBuf {
    roots
        .iter()
        .filter_map(|root| {
            path.strip_prefix(root.canonical())
                .ok()
                .map(|relative| (root, relative))
        })
        .max_by_key(|(root, _)| root.canonical().components().count())
        .map_or_else(
            || path.to_path_buf(),
            |(root, relative)| {
                if relative.as_os_str().is_empty() {
                    root.input().to_path_buf()
                } else {
                    root.input().join(relative)
                }
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn validated_scan_roots_resolve_parent_segments_before_walking() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let root = dir.path().join("repo");
        let src = root.join("src");
        fs::create_dir_all(&src)?;
        fs::write(src.join("lib.rs"), "pub fn lib() {}\n")?;

        let aliased_root = src.join("..");
        let roots = validated_scan_roots(&[aliased_root])?;

        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].canonical(), fs::canonicalize(&root)?);
        Ok(())
    }
}
