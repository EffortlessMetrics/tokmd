//! File classification support for effort size-basis calculations.
//!
//! This module coordinates `.gitattributes` classification, generated/vendored
//! sentinels, and heuristic path tagging. Size-basis aggregation stays in the
//! parent module.

use std::path::Path;

use tokmd_types::FileRow;

mod gitattributes;
mod heuristics;

use gitattributes::matches_path_pattern;
pub(super) use gitattributes::{GitAttrRule, load_gitattributes};
use heuristics::classify_file;
pub(super) use heuristics::tag_name;

#[derive(Debug, Clone, Copy)]
pub(super) enum FileKind {
    Core,
    Infra,
    Build,
    Docs,
    Tests,
    Generated,
    Vendored,
    Api,
    Ffi,
    Ui,
    Data,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum ClassKind {
    Unknown,
    Generated,
    Vendored,
}

impl ClassKind {
    #[allow(dead_code)]
    fn confidence_boost(self) -> f64 {
        match self {
            Self::Generated | Self::Vendored => 1.0,
            Self::Unknown => 0.0,
        }
    }
}

pub(super) fn classify_row(
    root: &Path,
    path: &str,
    rules: &[GitAttrRule],
    row: &FileRow,
) -> (ClassKind, FileKind) {
    let _lower = path.to_lowercase();

    for rule in rules {
        if matches_path_pattern(path, root, &rule.pattern) {
            return (
                rule.kind,
                match rule.kind {
                    ClassKind::Generated => FileKind::Generated,
                    ClassKind::Vendored => FileKind::Vendored,
                    ClassKind::Unknown => FileKind::Core,
                },
            );
        }
    }

    let kind = classify_file(root, path, row);

    let class = match kind {
        FileKind::Generated => ClassKind::Generated,
        FileKind::Vendored => ClassKind::Vendored,
        _ => ClassKind::Unknown,
    };

    (class, kind)
}
