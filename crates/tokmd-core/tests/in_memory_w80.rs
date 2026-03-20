use anyhow::Result;
use std::fs;
use std::path::Path;

use tempfile::TempDir;
use tokmd_core::{
    InMemoryFile, export_workflow, export_workflow_from_inputs, lang_workflow,
    lang_workflow_from_inputs, module_workflow_from_inputs,
    settings::{ExportSettings, LangSettings, ModuleSettings, ScanOptions, ScanSettings},
};
#[cfg(feature = "analysis")]
use tokmd_core::{analyze_workflow_from_inputs, settings::AnalyzeSettings};
use tokmd_types::ConfigMode;

fn scan_options() -> ScanOptions {
    ScanOptions {
        excluded: vec![],
        config: ConfigMode::None,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    }
}

fn write_file(root: &Path, rel: &str, contents: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

fn fixture_dir() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    write_file(
        dir.path(),
        "crates/app/src/lib.rs",
        "pub fn alpha() -> usize { 1 }\n",
    );
    write_file(dir.path(), "src/main.rs", "fn main() {}\n");
    write_file(
        dir.path(),
        "tests/basic.py",
        "# TODO: keep smoke\nprint('ok')\n",
    );
    dir
}

fn fixture_inputs() -> Vec<InMemoryFile> {
    vec![
        InMemoryFile::new("crates/app/src/lib.rs", "pub fn alpha() -> usize { 1 }\n"),
        InMemoryFile::new("src/main.rs", "fn main() {}\n"),
        InMemoryFile::new("tests/basic.py", "# TODO: keep smoke\nprint('ok')\n"),
    ]
}

#[test]
fn lang_workflow_from_inputs_matches_path_workflow_report() -> Result<()> {
    let dir = fixture_dir();
    let scan = ScanSettings {
        paths: vec![dir.path().display().to_string()],
        options: scan_options(),
    };
    let lang = LangSettings::default();

    let expected = lang_workflow(&scan, &lang)?;
    let actual = lang_workflow_from_inputs(&fixture_inputs(), &scan.options, &lang)?;

    assert_eq!(actual.report.rows, expected.report.rows);
    assert_eq!(actual.report.total, expected.report.total);
    assert_eq!(
        actual.scan.paths,
        vec!["crates/app/src/lib.rs", "src/main.rs", "tests/basic.py"]
    );

    Ok(())
}

#[test]
fn module_workflow_from_inputs_uses_virtual_relative_module_keys() -> Result<()> {
    let module = ModuleSettings::default();

    let actual = module_workflow_from_inputs(&fixture_inputs(), &scan_options(), &module)?;

    assert_eq!(
        actual
            .report
            .rows
            .iter()
            .map(|row| row.module.as_str())
            .collect::<Vec<_>>(),
        vec!["crates/app", "src", "tests"]
    );
    assert_eq!(actual.report.total.files, 3);
    assert_eq!(actual.report.total.code, 3);

    Ok(())
}

#[test]
fn export_workflow_from_inputs_matches_path_workflow_with_virtual_strip_prefix() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(
        dir.path(),
        "repo/src/lib.rs",
        "pub fn alpha() -> usize { 1 }\n",
    );
    write_file(dir.path(), "repo/tests/basic.py", "print('ok')\n");

    let scan = ScanSettings {
        paths: vec![dir.path().display().to_string()],
        options: scan_options(),
    };
    let path_export = ExportSettings {
        strip_prefix: Some(dir.path().join("repo").display().to_string()),
        ..Default::default()
    };
    let input_export = ExportSettings {
        strip_prefix: Some("repo".to_string()),
        ..Default::default()
    };
    let inputs = vec![
        InMemoryFile::new("repo/src/lib.rs", "pub fn alpha() -> usize { 1 }\n"),
        InMemoryFile::new("repo/tests/basic.py", "print('ok')\n"),
    ];

    let expected = export_workflow(&scan, &path_export)?;
    let actual = export_workflow_from_inputs(&inputs, &scan.options, &input_export)?;

    assert_eq!(actual.data.rows, expected.data.rows);
    assert_eq!(actual.data.module_roots, expected.data.module_roots);
    assert_eq!(
        actual
            .data
            .rows
            .iter()
            .map(|row| row.path.as_str())
            .collect::<Vec<_>>(),
        vec!["src/lib.rs", "tests/basic.py"]
    );
    assert_eq!(
        actual.scan.paths,
        vec!["repo/src/lib.rs", "repo/tests/basic.py"]
    );

    Ok(())
}

#[test]
fn export_workflow_from_inputs_preserves_path_redaction() -> Result<()> {
    let export = ExportSettings {
        redact: tokmd_types::RedactMode::Paths,
        ..Default::default()
    };
    let receipt = export_workflow_from_inputs(
        &[InMemoryFile::new("src/lib.rs", "pub fn alpha() {}\n")],
        &scan_options(),
        &export,
    )?;

    assert_ne!(receipt.data.rows[0].path, "src/lib.rs");
    assert_ne!(receipt.scan.paths[0], "src/lib.rs");

    Ok(())
}

#[cfg(feature = "analysis")]
#[test]
fn analyze_workflow_from_inputs_uses_logical_inputs_and_populates_estimate_receipt() -> Result<()> {
    let analyze = AnalyzeSettings {
        preset: "estimate".to_string(),
        ..Default::default()
    };
    let actual = analyze_workflow_from_inputs(&fixture_inputs(), &scan_options(), &analyze)?;
    let actual_derived = actual
        .derived
        .as_ref()
        .expect("estimate should populate derived metrics");
    let effort = actual
        .effort
        .as_ref()
        .expect("estimate should produce effort");

    assert_eq!(actual_derived.totals.files, 3);
    assert_eq!(actual_derived.totals.code, 3);
    assert!(actual_derived.totals.bytes > 0);
    assert!(actual_derived.totals.tokens > 0);
    assert_eq!(effort.size_basis.total_lines, actual_derived.totals.code);
    assert!(effort.results.effort_pm_p50 > 0.0);
    assert_eq!(effort.model.to_string(), "cocomo81-basic");
    assert_eq!(
        actual.source.inputs,
        vec![
            "crates/app/src/lib.rs".to_string(),
            "src/main.rs".to_string(),
            "tests/basic.py".to_string(),
        ]
    );
    assert!(
        actual
            .source
            .inputs
            .iter()
            .all(|path| !path.contains("/tmp/"))
    );
    assert!(
        actual
            .source
            .inputs
            .iter()
            .all(|path| !path.contains("\\temp\\"))
    );

    Ok(())
}

#[cfg(feature = "analysis")]
#[test]
fn analyze_workflow_from_inputs_runs_health_preset_against_materialized_files() -> Result<()> {
    let analyze = AnalyzeSettings {
        preset: "health".to_string(),
        ..Default::default()
    };
    let receipt = analyze_workflow_from_inputs(&fixture_inputs(), &scan_options(), &analyze)?;
    let derived = receipt
        .derived
        .as_ref()
        .expect("health should populate derived metrics");
    let todo = derived
        .todo
        .as_ref()
        .expect("health should populate TODO data");

    assert!(todo.total > 0);
    assert!(
        todo.tags
            .iter()
            .any(|tag| tag.tag.eq_ignore_ascii_case("todo"))
    );

    Ok(())
}
