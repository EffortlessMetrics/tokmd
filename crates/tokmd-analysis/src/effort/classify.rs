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

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_types::{FileKind as TokmdFileKind, FileRow};

    fn make_row(path: &str) -> FileRow {
        FileRow {
            path: path.to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: TokmdFileKind::Parent,
            code: 10,
            comments: 0,
            blanks: 0,
            lines: 10,
            bytes: 40,
            tokens: 20,
        }
    }

    #[test]
    fn class_kind_confidence_boost_pins_known_vs_unknown() {
        assert_eq!(ClassKind::Generated.confidence_boost(), 1.0);
        assert_eq!(ClassKind::Vendored.confidence_boost(), 1.0);
        assert_eq!(ClassKind::Unknown.confidence_boost(), 0.0);
    }

    #[test]
    fn classify_row_uses_gitattributes_rule_first_for_generated_match() {
        // A matching gitattributes rule short-circuits the heuristic.
        let rules = vec![GitAttrRule {
            pattern: "src/lib.rs".to_string(),
            kind: ClassKind::Generated,
            source: "test".to_string(),
        }];
        let row = make_row("src/lib.rs");
        let (class, kind) = classify_row(Path::new(""), "src/lib.rs", &rules, &row);
        assert_eq!(class, ClassKind::Generated);
        assert!(matches!(kind, FileKind::Generated));
    }

    #[test]
    fn classify_row_uses_gitattributes_rule_for_vendored_match() {
        let rules = vec![GitAttrRule {
            pattern: "vendor/foo.rs".to_string(),
            kind: ClassKind::Vendored,
            source: "test".to_string(),
        }];
        let row = make_row("vendor/foo.rs");
        let (class, kind) = classify_row(Path::new(""), "vendor/foo.rs", &rules, &row);
        assert_eq!(class, ClassKind::Vendored);
        assert!(matches!(kind, FileKind::Vendored));
    }

    #[test]
    fn classify_row_unknown_rule_returns_core_filekind() {
        // ClassKind::Unknown rules map to FileKind::Core (the documented
        // catch-all in the match arm).
        let rules = vec![GitAttrRule {
            pattern: "src/lib.rs".to_string(),
            kind: ClassKind::Unknown,
            source: "test".to_string(),
        }];
        let row = make_row("src/lib.rs");
        let (class, kind) = classify_row(Path::new(""), "src/lib.rs", &rules, &row);
        assert_eq!(class, ClassKind::Unknown);
        assert!(matches!(kind, FileKind::Core));
    }

    #[test]
    fn classify_row_falls_back_to_heuristic_when_no_rule_matches() {
        // Empty rule set forces the heuristic path. `_test.rs` suffix routes
        // through `looks_test_path` to FileKind::Tests / ClassKind::Unknown.
        let rules: Vec<GitAttrRule> = Vec::new();
        let row = make_row("src/foo_test.rs");
        let (class, kind) = classify_row(Path::new(""), "src/foo_test.rs", &rules, &row);
        assert_eq!(class, ClassKind::Unknown);
        assert!(matches!(kind, FileKind::Tests));
    }

    #[test]
    fn classify_row_heuristic_flags_generated_artifacts() {
        // A generated-looking path should fall through to the heuristic and
        // be tagged as ClassKind::Generated even without a gitattributes rule.
        let rules: Vec<GitAttrRule> = Vec::new();
        let row = make_row("target/generated/bundle.min.js");
        let (class, kind) = classify_row(
            Path::new(""),
            "target/generated/bundle.min.js",
            &rules,
            &row,
        );
        assert_eq!(class, ClassKind::Generated);
        assert!(matches!(kind, FileKind::Generated));
    }
}
