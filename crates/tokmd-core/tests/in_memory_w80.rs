use anyhow::Result;
use std::fs;
use std::path::Path;

use tempfile::TempDir;
use tokmd_core::{
    InMemoryFile, export_workflow, export_workflow_from_inputs, lang_workflow,
    lang_workflow_from_inputs, module_workflow_from_inputs,
    settings::{ExportSettings, LangSettings, ModuleSettings, ScanOptions, ScanSettings},
};
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
    write_file(dir.path(), "tests/basic.py", "print('ok')\n");
    dir
}

fn fixture_inputs() -> Vec<InMemoryFile> {
    vec![
        InMemoryFile::new("crates/app/src/lib.rs", "pub fn alpha() -> usize { 1 }\n"),
        InMemoryFile::new("src/main.rs", "fn main() {}\n"),
        InMemoryFile::new("tests/basic.py", "print('ok')\n"),
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
