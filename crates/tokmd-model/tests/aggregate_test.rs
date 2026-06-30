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

#[test]
fn fold_other_lang_calculates_avg_lines_correctly() {
    let report = create_lang_report_from_rows(
        &[
            file_row(FileKind::Parent, "Rust", 500, 125),
            file_row(FileKind::Parent, "Go", 500, 125),
            file_row(FileKind::Parent, "Python", 500, 125),
            file_row(FileKind::Parent, "C", 500, 125),
        ],
        2, // top 2
        false,
        ChildrenMode::Collapse,
    );
    assert_eq!(report.rows.len(), 3);
    assert_eq!(report.rows[2].lang, "Other");
    assert_eq!(report.rows[2].lines, 240); // 120 + 120
    assert_eq!(report.rows[2].files, 2); // 1 + 1
    // The model should use `avg(lines, files)`. avg(240, 2) = 120.
    assert_eq!(report.rows[2].avg_lines, 120);
}

#[test]
fn fold_other_lang_calculates_avg_lines_rounding() {
    let mut rows = vec![
        file_row(FileKind::Parent, "Rust", 500, 125),
        file_row(FileKind::Parent, "Go", 500, 125),
        file_row(FileKind::Parent, "Python", 500, 125),
    ];
    // make sure lines sum to 13 and files to 3 (avg = 4.33 -> rounds to 4). Or lines = 14, files = 3 (avg = 4.66 -> rounds to 5)
    rows[1].code = 4; rows[1].lines = 5;
    rows[2].code = 8; rows[2].lines = 9;

    let report = create_lang_report_from_rows(
        &rows,
        1, // top 1
        false,
        ChildrenMode::Collapse,
    );
    assert_eq!(report.rows.len(), 2);
    assert_eq!(report.rows[1].lang, "Other");
    assert_eq!(report.rows[1].lines, 14);
    assert_eq!(report.rows[1].files, 2);
    assert_eq!(report.rows[1].avg_lines, 7);




}
use tokmd_model::create_module_report_from_rows;
use tokmd_types::ChildIncludeMode;

#[test]
fn fold_other_module_calculates_avg_lines_correctly() {
    let mut rows = vec![
        file_row(FileKind::Parent, "Rust", 500, 125),
        file_row(FileKind::Parent, "Go", 500, 125),
        file_row(FileKind::Parent, "Python", 500, 125),
    ];
    rows[0].module = "crates/a".to_string();
    rows[1].module = "crates/b".to_string();
    rows[2].module = "crates/c".to_string();
    rows[1].code = 4; rows[1].lines = 5;
    rows[2].code = 8; rows[2].lines = 9;

    let report = create_module_report_from_rows(
        &rows,
        &[], // module_roots
        1,   // depth
        ChildIncludeMode::ParentsOnly,
        1,   // top 1
    );
    assert_eq!(report.rows.len(), 2);
    assert_eq!(report.rows[1].module, "Other");
    assert_eq!(report.rows[1].lines, 14); // 5 + 9
    assert_eq!(report.rows[1].files, 2); // 1 + 1
    assert_eq!(report.rows[1].avg_lines, 7);
}
