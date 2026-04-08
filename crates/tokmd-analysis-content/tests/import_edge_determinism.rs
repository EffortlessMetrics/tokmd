use std::path::PathBuf;
use tokmd_analysis_content::{ContentLimits, ImportGranularity, build_import_report};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

fn make_row(path: &str, module: &str, lang: &str) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code: 1,
        comments: 0,
        blanks: 0,
        lines: 1,
        bytes: 10,
        tokens: 2,
    }
}

fn write_temp_files(files: &[(&str, &str)]) -> (tempfile::TempDir, Vec<PathBuf>) {
    let tmp = tempfile::tempdir().unwrap();
    let mut paths = Vec::new();
    for (name, content) in files {
        let p = tmp.path().join(name);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&p, content).unwrap();
        paths.push(PathBuf::from(name));
    }
    (tmp, paths)
}

#[test]
fn import_edges_are_deterministically_sorted_by_destination() {
    let files = vec![
        (
            "src/file_a.rs",
            "use first_crate::foo;\nuse second_crate::bar;\n",
        ),
        ("src/file_b.rs", "pub fn b() {}\n"),
        ("src/file_c.rs", "pub fn c() {}\n"),
    ];

    let (tmp, paths) = write_temp_files(&files);

    let rows = vec![
        make_row("src/file_a.rs", "crate", "Rust"),
        make_row("src/file_b.rs", "crate", "Rust"),
        make_row("src/file_c.rs", "crate", "Rust"),
    ];

    let export = ExportData {
        rows,
        children: ChildIncludeMode::ParentsOnly,
        module_roots: vec![],
        module_depth: 1,
    };
    let limits = ContentLimits {
        max_bytes: None,
        max_file_bytes: None,
    };

    let report = build_import_report(
        tmp.path(),
        &paths,
        &export,
        ImportGranularity::File,
        &limits,
    )
    .unwrap();

    let edges = report.edges;
    assert_eq!(edges.len(), 2, "Should have two import edges");

    // They have identical count (1) and identical 'from' ("src/file_a.rs").
    // The tie breaker should order them by 'to' alphabetically.
    assert_eq!(edges[0].from, "src/file_a.rs");
    assert_eq!(edges[0].to, "first_crate");
    assert_eq!(edges[0].count, 1);

    assert_eq!(edges[1].from, "src/file_a.rs");
    assert_eq!(edges[1].to, "second_crate");
    assert_eq!(edges[1].count, 1);
}
