use tokmd_model::create_lang_report_from_rows;
use tokmd_types::{ChildrenMode, FileKind, FileRow};

fn file_row(kind: FileKind, lang: &str, bytes: usize, tokens: usize) -> FileRow {
    FileRow {
        path: "docs/example.md".to_string(),
        lang: lang.to_string(),
        code: 100,
        lines: 120,
        blanks: 10,
        comments: 10,
        bytes,
        tokens,
        module: "docs".to_string(),
        kind,
    }
}

#[test]
fn collapse_mode_keeps_orphan_child_bytes_and_tokens() {
    let report = create_lang_report_from_rows(
        &[file_row(FileKind::Child, "Rust", 500, 125)],
        0,
        true,
        ChildrenMode::Collapse,
    );

    assert_eq!(report.rows.len(), 1);
    assert_eq!(report.rows[0].lang, "Rust");
    assert_eq!(report.rows[0].code, 100);
    assert_eq!(report.rows[0].lines, 120);
    assert_eq!(report.rows[0].files, 1);
    assert_eq!(report.rows[0].bytes, 500);
    assert_eq!(report.rows[0].tokens, 125);
    assert_eq!(report.total.bytes, 500);
    assert_eq!(report.total.tokens, 125);
}

#[test]
fn separate_mode_does_not_count_child_bytes_or_tokens() {
    let report = create_lang_report_from_rows(
        &[file_row(FileKind::Child, "Rust", 500, 125)],
        0,
        true,
        ChildrenMode::Separate,
    );

    assert_eq!(report.rows.len(), 1);
    assert_eq!(report.rows[0].lang, "Rust (embedded)");
    assert_eq!(report.rows[0].code, 100);
    assert_eq!(report.rows[0].lines, 120);
    assert_eq!(report.rows[0].files, 1);
    assert_eq!(report.rows[0].bytes, 0);
    assert_eq!(report.rows[0].tokens, 0);
    assert_eq!(report.total.bytes, 0);
    assert_eq!(report.total.tokens, 0);
}
