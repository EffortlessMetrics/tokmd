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
    let row_opt = report.rows.iter().find(|r| r.lang == "Other");
    assert!(row_opt.is_some());
    if let Some(row) = row_opt {
        assert_eq!(row.lines, 240); // 120 + 120
        assert_eq!(row.files, 2); // 1 + 1
        // The model should use `avg(lines, files)`. avg(240, 2) = 120.
        assert_eq!(row.avg_lines, 120);
    }
}

#[test]
fn fold_other_lang_calculates_avg_lines_rounding() {
    let mut rows = vec![
        file_row(FileKind::Parent, "Rust", 500, 125),
        file_row(FileKind::Parent, "Go", 500, 125),
        file_row(FileKind::Parent, "Python", 500, 125),
    ];
    if let Some(r1) = rows.get_mut(1) {
        r1.code = 4;
        r1.lines = 5;
    }
    if let Some(r2) = rows.get_mut(2) {
        r2.code = 8;
        r2.lines = 9;
    }

    let report = create_lang_report_from_rows(
        &rows,
        1, // top 1
        false,
        ChildrenMode::Collapse,
    );
    assert_eq!(report.rows.len(), 2);
    let row_opt = report.rows.iter().find(|r| r.lang == "Other");
    assert!(row_opt.is_some());
    if let Some(row) = row_opt {
        assert_eq!(row.lines, 14); // 5 + 9
        assert_eq!(row.files, 2); // 1 + 1
        // `avg(14, 2)` = 7.
        assert_eq!(row.avg_lines, 7);
    }
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
    if let Some(r0) = rows.get_mut(0) {
        r0.module = "crates/a".to_string();
    }
    if let Some(r1) = rows.get_mut(1) {
        r1.module = "crates/b".to_string();
        r1.code = 4;
        r1.lines = 5;
    }
    if let Some(r2) = rows.get_mut(2) {
        r2.module = "crates/c".to_string();
        r2.code = 8;
        r2.lines = 9;
    }

    let report = create_module_report_from_rows(
        &rows,
        &[], // module_roots
        1,   // depth
        ChildIncludeMode::ParentsOnly,
        1, // top 1
    );
    assert_eq!(report.rows.len(), 2);
    let row_opt = report.rows.iter().find(|r| r.module == "Other");
    assert!(row_opt.is_some());
    if let Some(row) = row_opt {
        assert_eq!(row.lines, 14); // 5 + 9
        assert_eq!(row.files, 2); // 1 + 1
        assert_eq!(row.avg_lines, 7);
    }
}
