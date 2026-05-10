use tokmd_model::{create_lang_report_from_rows};
use tokmd_types::{ChildrenMode, FileKind, FileRow};

#[test]
fn test_create_lang_report_from_rows_collapse_child_no_parent() {
    let rows = vec![
        FileRow {
            kind: FileKind::Child,
            path: "test.md".to_string(),
            lang: "Rust".to_string(),
            code: 100,
            lines: 120,
            blanks: 10,
            comments: 10,
            bytes: 500,
            tokens: 300,
            module: "".to_string(),
        }
    ];

    let report = create_lang_report_from_rows(&rows, 10, true, ChildrenMode::Collapse);

    assert_eq!(report.rows.len(), 1);
    assert_eq!(report.rows[0].lang, "Rust");
    assert_eq!(report.rows[0].code, 100);
    assert_eq!(report.rows[0].lines, 120);
    assert_eq!(report.rows[0].files, 1);

    // Test that bytes and tokens are correctly propagated when collapsing children without a parent
    assert_eq!(report.rows[0].bytes, 500);
    assert_eq!(report.rows[0].tokens, 300);
}

#[test]
fn test_create_lang_report_from_rows_separate_child() {
    let rows = vec![
        FileRow {
            kind: FileKind::Child,
            path: "test.md".to_string(),
            lang: "Rust".to_string(),
            code: 100,
            lines: 120,
            blanks: 10,
            comments: 10,
            bytes: 500,
            tokens: 300,
            module: "".to_string(),
        }
    ];

    let report = create_lang_report_from_rows(&rows, 10, true, ChildrenMode::Separate);

    assert_eq!(report.rows.len(), 1);
    assert_eq!(report.rows[0].lang, "Rust (embedded)");
    assert_eq!(report.rows[0].code, 100);
    assert_eq!(report.rows[0].lines, 120);
    assert_eq!(report.rows[0].files, 1);

    // Test that bytes and tokens are propagated for Separate children
    assert_eq!(report.rows[0].bytes, 500);
    assert_eq!(report.rows[0].tokens, 300);
}
