//! Export row redaction shared by dataset renderers.
//!
//! This module owns path/module redaction for file-level export rows. Format
//! writers consume the iterator so CSV, JSON, JSONL, and CycloneDX stay aligned.

use std::borrow::Cow;

use tokmd_types::{FileRow, RedactMode};

use crate::{redact_path, short_hash};

pub(super) fn redact_rows(
    rows: &[FileRow],
    mode: RedactMode,
) -> impl Iterator<Item = Cow<'_, FileRow>> {
    rows.iter().map(move |r| match mode {
        RedactMode::None => Cow::Borrowed(r),
        RedactMode::Paths => Cow::Owned(FileRow {
            path: redact_path(&r.path),
            module: r.module.clone(),
            lang: r.lang.clone(),
            kind: r.kind,
            code: r.code,
            comments: r.comments,
            blanks: r.blanks,
            lines: r.lines,
            bytes: r.bytes,
            tokens: r.tokens,
        }),
        RedactMode::All => Cow::Owned(FileRow {
            path: redact_path(&r.path),
            module: short_hash(&r.module),
            lang: r.lang.clone(),
            kind: r.kind,
            code: r.code,
            comments: r.comments,
            blanks: r.blanks,
            lines: r.lines,
            bytes: r.bytes,
            tokens: r.tokens,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use tokmd_types::FileKind;

    fn sample_file_rows() -> Vec<FileRow> {
        vec![
            FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 20,
                blanks: 10,
                lines: 130,
                bytes: 1000,
                tokens: 250,
            },
            FileRow {
                path: "tests/test.rs".to_string(),
                module: "tests".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 50,
                comments: 5,
                blanks: 5,
                lines: 60,
                bytes: 500,
                tokens: 125,
            },
        ]
    }

    #[test]
    fn redact_rows_none_mode() {
        let rows = sample_file_rows();
        let redacted: Vec<_> = redact_rows(&rows, RedactMode::None).collect();

        assert_eq!(redacted.len(), rows.len());
        assert_eq!(redacted[0].path, "src/lib.rs");
        assert_eq!(redacted[0].module, "src");
    }

    #[test]
    fn redact_rows_paths_mode() {
        let rows = sample_file_rows();
        let redacted: Vec<_> = redact_rows(&rows, RedactMode::Paths).collect();

        assert_ne!(redacted[0].path, "src/lib.rs");
        assert!(redacted[0].path.ends_with(".rs"));
        assert_eq!(redacted[0].path.len(), 16 + 3); // hash + ".rs"

        assert_eq!(redacted[0].module, "src");
    }

    #[test]
    fn redact_rows_all_mode() {
        let rows = sample_file_rows();
        let redacted: Vec<_> = redact_rows(&rows, RedactMode::All).collect();

        assert_ne!(redacted[0].path, "src/lib.rs");
        assert!(redacted[0].path.ends_with(".rs"));

        assert_ne!(redacted[0].module, "src");
        assert_eq!(redacted[0].module.len(), 16);
    }

    #[test]
    fn redact_rows_preserves_other_fields() {
        let rows = sample_file_rows();
        let redacted: Vec<_> = redact_rows(&rows, RedactMode::All).collect();

        assert_eq!(redacted[0].lang, "Rust");
        assert_eq!(redacted[0].kind, FileKind::Parent);
        assert_eq!(redacted[0].code, 100);
        assert_eq!(redacted[0].comments, 20);
        assert_eq!(redacted[0].blanks, 10);
        assert_eq!(redacted[0].lines, 130);
        assert_eq!(redacted[0].bytes, 1000);
        assert_eq!(redacted[0].tokens, 250);
    }

    proptest! {
        #[test]
        fn redact_rows_preserves_count(
            code in 0usize..10000,
            comments in 0usize..1000,
            blanks in 0usize..500
        ) {
            let rows = vec![FileRow {
                path: "test/file.rs".to_string(),
                module: "test".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code,
                comments,
                blanks,
                lines: code + comments + blanks,
                bytes: 1000,
                tokens: 250,
            }];

            for mode in [RedactMode::None, RedactMode::Paths, RedactMode::All] {
                let redacted: Vec<_> = redact_rows(&rows, mode).collect();
                prop_assert_eq!(redacted.len(), 1);
                prop_assert_eq!(redacted[0].code, code);
                prop_assert_eq!(redacted[0].comments, comments);
                prop_assert_eq!(redacted[0].blanks, blanks);
            }
        }

        #[test]
        fn redact_rows_paths_preserve_allowlisted_extensions(ext in "rs|js|ts|json|md|toml|gz") {
            let path = format!("some/path/file.{}", ext);
            let rows = vec![FileRow {
                path: path.clone(),
                module: "some".to_string(),
                lang: "Test".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 10,
                blanks: 5,
                lines: 115,
                bytes: 1000,
                tokens: 250,
            }];

            let redacted: Vec<_> = redact_rows(&rows, RedactMode::Paths).collect();
            prop_assert!(redacted[0].path.ends_with(&format!(".{}", ext)),
                "Redacted path '{}' should end with .{}", redacted[0].path, ext);
        }

        #[test]
        fn redact_rows_paths_strip_untrusted_extensions(ext in "passwd|secret|pass1234|token") {
            let path = format!("some/path/file.{}", ext);
            let rows = vec![FileRow {
                path: path.clone(),
                module: "some".to_string(),
                lang: "Test".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 10,
                blanks: 5,
                lines: 115,
                bytes: 1000,
                tokens: 250,
            }];

            let redacted: Vec<_> = redact_rows(&rows, RedactMode::Paths).collect();
            prop_assert_eq!(redacted[0].path.len(), 16);
            prop_assert!(!redacted[0].path.contains('.'));
            prop_assert!(!redacted[0].path.contains(&ext));
        }
    }
}
