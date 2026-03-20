use std::path::PathBuf;

use serde_json::Value;
use serde_json::json;
use tokei::{Config, Languages};
use tokmd_model::{
    collect_file_rows, create_export_data, create_export_data_from_rows, create_lang_report,
    create_lang_report_from_rows, create_module_report, create_module_report_from_rows,
    unique_parent_file_count, unique_parent_file_count_from_rows,
};
use tokmd_types::{ChildIncludeMode, ChildrenMode, FileKind, FileRow};

fn scan_path(path: &str) -> Languages {
    let mut languages = Languages::new();
    let paths = vec![PathBuf::from(path)];
    let cfg = Config::default();
    languages.get_statistics(&paths, &[], &cfg);
    languages
}

fn crate_src_path() -> String {
    format!("{}/src", env!("CARGO_MANIFEST_DIR"))
}

fn to_json<T: serde::Serialize>(value: &T) -> Value {
    serde_json::to_value(value).unwrap()
}

fn reversed_rows(mut rows: Vec<FileRow>) -> Vec<FileRow> {
    rows.reverse();
    rows
}

fn fixture_rows() -> Vec<FileRow> {
    vec![
        FileRow {
            path: "src/app.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 100,
            comments: 10,
            blanks: 5,
            lines: 115,
            bytes: 400,
            tokens: 100,
        },
        FileRow {
            path: "web/page.html".to_string(),
            module: "web".to_string(),
            lang: "HTML".to_string(),
            kind: FileKind::Parent,
            code: 50,
            comments: 7,
            blanks: 5,
            lines: 62,
            bytes: 300,
            tokens: 75,
        },
        FileRow {
            path: "web/page.html".to_string(),
            module: "web".to_string(),
            lang: "JavaScript".to_string(),
            kind: FileKind::Child,
            code: 10,
            comments: 1,
            blanks: 1,
            lines: 12,
            bytes: 0,
            tokens: 0,
        },
    ]
}

#[test]
fn create_lang_report_from_rows_manual_collapse_fixture() {
    let report = create_lang_report_from_rows(&fixture_rows(), 0, false, ChildrenMode::Collapse);

    assert_eq!(
        to_json(&report),
        json!({
            "rows": [
                {
                    "lang": "Rust",
                    "code": 100,
                    "lines": 115,
                    "files": 1,
                    "bytes": 400,
                    "tokens": 100,
                    "avg_lines": 115
                },
                {
                    "lang": "HTML",
                    "code": 50,
                    "lines": 62,
                    "files": 1,
                    "bytes": 300,
                    "tokens": 75,
                    "avg_lines": 62
                }
            ],
            "total": {
                "code": 150,
                "lines": 177,
                "files": 2,
                "bytes": 700,
                "tokens": 175,
                "avg_lines": 89
            },
            "with_files": false,
            "children": "collapse",
            "top": 0
        })
    );
}

#[test]
fn create_lang_report_from_rows_manual_separate_fixture() {
    let report = create_lang_report_from_rows(&fixture_rows(), 0, false, ChildrenMode::Separate);

    assert_eq!(
        to_json(&report),
        json!({
            "rows": [
                {
                    "lang": "Rust",
                    "code": 100,
                    "lines": 115,
                    "files": 1,
                    "bytes": 400,
                    "tokens": 100,
                    "avg_lines": 115
                },
                {
                    "lang": "HTML",
                    "code": 40,
                    "lines": 50,
                    "files": 1,
                    "bytes": 300,
                    "tokens": 75,
                    "avg_lines": 50
                },
                {
                    "lang": "JavaScript (embedded)",
                    "code": 10,
                    "lines": 12,
                    "files": 1,
                    "bytes": 0,
                    "tokens": 0,
                    "avg_lines": 12
                }
            ],
            "total": {
                "code": 150,
                "lines": 177,
                "files": 2,
                "bytes": 700,
                "tokens": 175,
                "avg_lines": 89
            },
            "with_files": false,
            "children": "separate",
            "top": 0
        })
    );
}

#[test]
fn create_module_report_from_rows_manual_fixture() {
    let module_roots = vec!["src".to_string(), "web".to_string()];
    let report = create_module_report_from_rows(
        &fixture_rows(),
        &module_roots,
        2,
        ChildIncludeMode::Separate,
        0,
    );

    assert_eq!(
        to_json(&report),
        json!({
            "rows": [
                {
                    "module": "src",
                    "code": 100,
                    "lines": 115,
                    "files": 1,
                    "bytes": 400,
                    "tokens": 100,
                    "avg_lines": 115
                },
                {
                    "module": "web",
                    "code": 60,
                    "lines": 74,
                    "files": 1,
                    "bytes": 300,
                    "tokens": 75,
                    "avg_lines": 74
                }
            ],
            "total": {
                "code": 160,
                "lines": 189,
                "files": 2,
                "bytes": 700,
                "tokens": 175,
                "avg_lines": 95
            },
            "module_roots": ["src", "web"],
            "module_depth": 2,
            "children": "separate",
            "top": 0
        })
    );
}

#[test]
fn create_export_data_from_rows_manual_fixture() {
    let module_roots = vec!["src".to_string(), "web".to_string()];
    let report = create_export_data_from_rows(
        fixture_rows(),
        &module_roots,
        2,
        ChildIncludeMode::Separate,
        20,
        2,
    );

    assert_eq!(
        to_json(&report),
        json!({
            "rows": [
                {
                    "path": "src/app.rs",
                    "module": "src",
                    "lang": "Rust",
                    "kind": "parent",
                    "code": 100,
                    "comments": 10,
                    "blanks": 5,
                    "lines": 115,
                    "bytes": 400,
                    "tokens": 100
                },
                {
                    "path": "web/page.html",
                    "module": "web",
                    "lang": "HTML",
                    "kind": "parent",
                    "code": 50,
                    "comments": 7,
                    "blanks": 5,
                    "lines": 62,
                    "bytes": 300,
                    "tokens": 75
                }
            ],
            "module_roots": ["src", "web"],
            "module_depth": 2,
            "children": "separate"
        })
    );
}

#[test]
fn create_lang_report_from_rows_matches_collapse_report() {
    let languages = scan_path(&crate_src_path());
    let expected = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    let rows = collect_file_rows(&languages, &[], 1, ChildIncludeMode::Separate, None);
    let actual = create_lang_report_from_rows(&rows, 0, false, ChildrenMode::Collapse);

    assert_eq!(to_json(&actual), to_json(&expected));
}

#[test]
fn create_lang_report_from_rows_matches_separate_report() {
    let languages = scan_path(&crate_src_path());
    let expected = create_lang_report(&languages, 0, false, ChildrenMode::Separate);
    let rows = collect_file_rows(&languages, &[], 1, ChildIncludeMode::Separate, None);
    let actual = create_lang_report_from_rows(&rows, 0, false, ChildrenMode::Separate);

    assert_eq!(to_json(&actual), to_json(&expected));
}

#[test]
fn create_module_report_from_rows_matches_parents_only_report() {
    let languages = scan_path(&crate_src_path());
    let module_roots = vec!["crates".to_string()];
    let expected = create_module_report(
        &languages,
        &module_roots,
        2,
        ChildIncludeMode::ParentsOnly,
        0,
    );
    let rows = collect_file_rows(
        &languages,
        &module_roots,
        2,
        ChildIncludeMode::ParentsOnly,
        None,
    );
    let actual =
        create_module_report_from_rows(&rows, &module_roots, 2, ChildIncludeMode::ParentsOnly, 0);

    assert_eq!(to_json(&actual), to_json(&expected));
}

#[test]
fn create_module_report_from_rows_matches_separate_report() {
    let languages = scan_path(&crate_src_path());
    let module_roots = vec!["crates".to_string()];
    let expected =
        create_module_report(&languages, &module_roots, 2, ChildIncludeMode::Separate, 0);
    let rows = collect_file_rows(
        &languages,
        &module_roots,
        2,
        ChildIncludeMode::Separate,
        None,
    );
    let actual =
        create_module_report_from_rows(&rows, &module_roots, 2, ChildIncludeMode::Separate, 0);

    assert_eq!(to_json(&actual), to_json(&expected));
}

#[test]
fn create_export_data_from_rows_matches_parents_only_export() {
    let languages = scan_path(&crate_src_path());
    let module_roots = vec!["crates".to_string()];
    let expected = create_export_data(
        &languages,
        &module_roots,
        2,
        ChildIncludeMode::ParentsOnly,
        None,
        10,
        25,
    );
    let rows = collect_file_rows(
        &languages,
        &module_roots,
        2,
        ChildIncludeMode::ParentsOnly,
        None,
    );
    let actual = create_export_data_from_rows(
        rows,
        &module_roots,
        2,
        ChildIncludeMode::ParentsOnly,
        10,
        25,
    );

    assert_eq!(to_json(&actual), to_json(&expected));
}

#[test]
fn create_export_data_from_rows_matches_separate_export() {
    let languages = scan_path(&crate_src_path());
    let module_roots = vec!["crates".to_string()];
    let expected = create_export_data(
        &languages,
        &module_roots,
        2,
        ChildIncludeMode::Separate,
        None,
        5,
        40,
    );
    let rows = collect_file_rows(
        &languages,
        &module_roots,
        2,
        ChildIncludeMode::Separate,
        None,
    );
    let actual =
        create_export_data_from_rows(rows, &module_roots, 2, ChildIncludeMode::Separate, 5, 40);

    assert_eq!(to_json(&actual), to_json(&expected));
}

#[test]
fn unique_parent_file_count_from_rows_matches_languages_api() {
    let languages = scan_path(&crate_src_path());
    let rows = collect_file_rows(&languages, &[], 1, ChildIncludeMode::Separate, None);

    assert_eq!(
        unique_parent_file_count_from_rows(&rows),
        unique_parent_file_count(&languages)
    );
}

#[test]
fn row_based_apis_match_empty_languages_behavior() {
    let languages = Languages::new();

    let empty_collapse = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    let empty_collapse_rows = create_lang_report_from_rows(&[], 0, false, ChildrenMode::Collapse);
    assert_eq!(to_json(&empty_collapse_rows), to_json(&empty_collapse));

    let empty_separate = create_lang_report(&languages, 0, false, ChildrenMode::Separate);
    let empty_separate_rows = create_lang_report_from_rows(&[], 0, false, ChildrenMode::Separate);
    assert_eq!(to_json(&empty_separate_rows), to_json(&empty_separate));

    let module_roots = vec!["crates".to_string()];
    let empty_module =
        create_module_report(&languages, &module_roots, 2, ChildIncludeMode::Separate, 0);
    let empty_module_rows =
        create_module_report_from_rows(&[], &module_roots, 2, ChildIncludeMode::Separate, 0);
    assert_eq!(to_json(&empty_module_rows), to_json(&empty_module));

    let empty_export = create_export_data(
        &languages,
        &module_roots,
        2,
        ChildIncludeMode::Separate,
        None,
        0,
        0,
    );
    let empty_export_rows = create_export_data_from_rows(
        Vec::new(),
        &module_roots,
        2,
        ChildIncludeMode::Separate,
        0,
        0,
    );
    assert_eq!(to_json(&empty_export_rows), to_json(&empty_export));

    assert_eq!(unique_parent_file_count_from_rows(&[]), 0);
}

#[test]
fn create_lang_report_from_rows_is_deterministic_for_shuffled_input() {
    let languages = scan_path(&crate_src_path());
    let rows = collect_file_rows(&languages, &[], 1, ChildIncludeMode::Separate, None);
    let reversed = reversed_rows(rows.clone());

    let collapse_a = create_lang_report_from_rows(&rows, 0, false, ChildrenMode::Collapse);
    let collapse_b = create_lang_report_from_rows(&reversed, 0, false, ChildrenMode::Collapse);
    assert_eq!(to_json(&collapse_a), to_json(&collapse_b));

    let separate_a = create_lang_report_from_rows(&rows, 0, false, ChildrenMode::Separate);
    let separate_b = create_lang_report_from_rows(&reversed, 0, false, ChildrenMode::Separate);
    assert_eq!(to_json(&separate_a), to_json(&separate_b));
}

#[test]
fn create_module_report_from_rows_is_deterministic_for_shuffled_input() {
    let languages = scan_path(&crate_src_path());
    let module_roots = vec!["crates".to_string()];
    let rows = collect_file_rows(
        &languages,
        &module_roots,
        2,
        ChildIncludeMode::Separate,
        None,
    );
    let reversed = reversed_rows(rows.clone());

    let a = create_module_report_from_rows(&rows, &module_roots, 2, ChildIncludeMode::Separate, 0);
    let b =
        create_module_report_from_rows(&reversed, &module_roots, 2, ChildIncludeMode::Separate, 0);

    assert_eq!(to_json(&a), to_json(&b));
}

#[test]
fn create_export_data_from_rows_is_deterministic_for_shuffled_input() {
    let languages = scan_path(&crate_src_path());
    let module_roots = vec!["crates".to_string()];
    let rows = collect_file_rows(
        &languages,
        &module_roots,
        2,
        ChildIncludeMode::Separate,
        None,
    );
    let reversed = reversed_rows(rows.clone());

    let a = create_export_data_from_rows(rows, &module_roots, 2, ChildIncludeMode::Separate, 0, 0);
    let b =
        create_export_data_from_rows(reversed, &module_roots, 2, ChildIncludeMode::Separate, 0, 0);

    assert_eq!(to_json(&a), to_json(&b));
}

#[test]
fn unique_parent_file_count_from_rows_ignores_children_and_duplicates() {
    let rows = vec![
        FileRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 2,
            blanks: 1,
            lines: 13,
            bytes: 40,
            tokens: 10,
        },
        FileRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 2,
            blanks: 1,
            lines: 13,
            bytes: 40,
            tokens: 10,
        },
        FileRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "JavaScript".to_string(),
            kind: FileKind::Child,
            code: 3,
            comments: 0,
            blanks: 0,
            lines: 3,
            bytes: 0,
            tokens: 0,
        },
        FileRow {
            path: "tests/test.rs".to_string(),
            module: "tests".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 8,
            comments: 1,
            blanks: 1,
            lines: 10,
            bytes: 32,
            tokens: 8,
        },
    ];

    assert_eq!(unique_parent_file_count_from_rows(&rows), 2);
    assert_eq!(
        unique_parent_file_count_from_rows(&reversed_rows(rows.clone())),
        2
    );
}
