//! Repository archetype inference for analysis receipts.
//!
//! This module preserves the former `analysis archetype module` seam inside the
//! `tokmd-analysis` owner crate.

mod rules;

use std::collections::BTreeSet;

use tokmd_analysis_types::Archetype;
use tokmd_types::{ExportData, FileKind};

pub(crate) fn detect_archetype(export: &ExportData) -> Option<Archetype> {
    let mut files: BTreeSet<String> = BTreeSet::new();
    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        files.insert(row.path.replace('\\', "/"));
    }

    rules::detect(&files)
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

    fn export_with_paths(paths: &[&str]) -> ExportData {
        let rows = paths
            .iter()
            .map(|p| FileRow {
                path: (*p).to_string(),
                module: "(root)".to_string(),
                lang: "Rust".to_string(),
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
            module_roots: vec!["crates".to_string()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        }
    }

    #[test]
    fn detect_archetype_normalizes_parent_paths_and_ignores_child_rows() {
        let mut export = export_with_paths(&["Cargo.toml", "packages\\foo\\src\\lib.rs"]);
        export.rows.push(FileRow {
            path: "src/main.rs".to_string(),
            module: "(root)".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes: 10,
            tokens: 2,
        });

        let archetype = detect_archetype(&export).unwrap();
        assert_eq!(archetype.kind, "Rust workspace");
        assert!(
            archetype
                .evidence
                .iter()
                .any(|e| e == "packages/foo/src/lib.rs")
        );
    }
}

#[cfg(test)]
mod tests;
