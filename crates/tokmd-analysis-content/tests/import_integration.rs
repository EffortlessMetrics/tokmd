use std::collections::BTreeMap;
use std::path::PathBuf;

use tokmd_analysis_content::{ContentLimits, ImportGranularity, build_import_report};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

fn file_row(path: &str, module: &str, lang: &str) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code: 1,
        comments: 0,
        blanks: 0,
        lines: 1,
        bytes: 64,
        tokens: 8,
    }
}

#[test]
fn given_mixed_language_files_when_building_module_import_report_then_edges_are_aggregated() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    let src_dir = root.join("src");
    let web_dir = root.join("web");
    std::fs::create_dir_all(&src_dir).expect("src dir");
    std::fs::create_dir_all(&web_dir).expect("web dir");

    std::fs::write(
        src_dir.join("lib.rs"),
        "use serde_json::Value;\nmod util;\n",
    )
    .expect("write rust file");
    std::fs::write(
        web_dir.join("index.ts"),
        "import React from \"react\";\nconst util = require(\"./util/helpers\");\n",
    )
    .expect("write ts file");

    let files = vec![PathBuf::from("src/lib.rs"), PathBuf::from("web/index.ts")];
    let export = ExportData {
        rows: vec![
            file_row("src/lib.rs", "src", "Rust"),
            file_row("web/index.ts", "web", "TypeScript"),
        ],
        module_roots: vec!["src".to_string(), "web".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::Module,
        &ContentLimits::default(),
    )
    .expect("import report");

    let edge_counts: BTreeMap<(String, String), usize> = report
        .edges
        .iter()
        .map(|edge| ((edge.from.clone(), edge.to.clone()), edge.count))
        .collect();

    assert_eq!(report.granularity, "module");
    assert_eq!(
        edge_counts.get(&("src".into(), "serde_json".into())),
        Some(&1)
    );
    assert_eq!(edge_counts.get(&("src".into(), "util".into())), Some(&1));
    assert_eq!(edge_counts.get(&("web".into(), "react".into())), Some(&1));
    assert_eq!(edge_counts.get(&("web".into(), "local".into())), Some(&1));
}

#[test]
fn given_file_granularity_when_building_import_report_then_from_uses_file_paths() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("pkg")).expect("pkg dir");
    std::fs::write(root.join("pkg/main.py"), "import requests\n").expect("write py");

    let files = vec![PathBuf::from("pkg/main.py")];
    let export = ExportData {
        rows: vec![file_row("pkg/main.py", "pkg", "Python")],
        module_roots: vec!["pkg".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let report = build_import_report(
        root,
        &files,
        &export,
        ImportGranularity::File,
        &ContentLimits::default(),
    )
    .expect("import report");

    assert_eq!(report.granularity, "file");
    assert_eq!(report.edges.len(), 1);
    assert_eq!(report.edges[0].from, "pkg/main.py");
    assert_eq!(report.edges[0].to, "requests");
    assert_eq!(report.edges[0].count, 1);
}
