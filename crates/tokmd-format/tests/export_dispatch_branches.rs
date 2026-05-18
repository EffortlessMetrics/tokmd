//! Branch-coverage tests for `tokmd_format::write_export` dispatch and
//! `write_module_json_to_file`'s `RedactMode::All` branch.
//!
//! `write_export` (and its private `write_export_to` dispatcher) cover
//! four `ExportFormat` arms. The existing test suite only exercises the
//! `Jsonl` arm end-to-end; this file rounds out the dispatch matrix and
//! pins the file-output path. The companion test asserts that the
//! module-receipt JSON writer hashes module names when redaction is set
//! to `All`.

use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;

use tokmd_format::{short_hash, write_export, write_lang_json_to_file, write_module_json_to_file};
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    ConfigMode, ExportArgs, ExportData, ExportFormat, FileKind, FileRow, LangArgsMeta, LangReport,
    LangRow, ModuleArgsMeta, ModuleReport, ModuleRow, RedactMode, ScanArgs, Totals,
};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn sample_export_data() -> ExportData {
    ExportData {
        rows: vec![FileRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 100,
            comments: 20,
            blanks: 10,
            lines: 130,
            bytes: 5000,
            tokens: 250,
        }],
        module_roots: vec!["src".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn export_args(format: ExportFormat, output: Option<PathBuf>) -> ExportArgs {
    ExportArgs {
        paths: vec![PathBuf::from(".")],
        format,
        output,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    }
}

fn scan_args_stub() -> ScanArgs {
    ScanArgs {
        paths: vec![".".to_string()],
        excluded: vec![],
        excluded_redacted: false,
        config: ConfigMode::Auto,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    }
}

fn sample_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![
            ModuleRow {
                module: "crates/foo".to_string(),
                code: 100,
                lines: 120,
                files: 1,
                bytes: 4000,
                tokens: 200,
                avg_lines: 120,
            },
            ModuleRow {
                module: "crates/bar".to_string(),
                code: 80,
                lines: 100,
                files: 1,
                bytes: 3000,
                tokens: 150,
                avg_lines: 100,
            },
        ],
        total: Totals {
            code: 180,
            lines: 220,
            files: 2,
            bytes: 7000,
            tokens: 350,
            avg_lines: 110,
        },
        module_roots: vec!["crates/foo".to_string(), "crates/bar".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

fn sample_lang_report() -> LangReport {
    LangReport {
        rows: vec![LangRow {
            lang: "Rust".to_string(),
            code: 100,
            lines: 130,
            files: 1,
            bytes: 5000,
            tokens: 250,
            avg_lines: 130,
        }],
        total: Totals {
            code: 100,
            lines: 130,
            files: 1,
            bytes: 5000,
            tokens: 250,
            avg_lines: 130,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

// ---------------------------------------------------------------------------
// write_export — ExportFormat dispatch matrix (file output path)
// ---------------------------------------------------------------------------

#[test]
fn write_export_dispatches_csv_format_to_file() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("out.csv");
    let args = export_args(ExportFormat::Csv, Some(path.clone()));

    write_export(&sample_export_data(), &ScanOptions::default(), &args)
        .expect("write_export csv succeeds");

    let out = fs::read_to_string(&path).expect("csv file readable");
    // CSV header line is fixed by the writer.
    assert!(
        out.starts_with("path,"),
        "csv must start with header: {out}"
    );
    assert!(out.contains("src/lib.rs"));
}

#[test]
fn write_export_dispatches_jsonl_format_to_file() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("out.jsonl");
    let args = export_args(ExportFormat::Jsonl, Some(path.clone()));

    write_export(&sample_export_data(), &ScanOptions::default(), &args)
        .expect("write_export jsonl succeeds");

    let out = fs::read_to_string(&path).expect("jsonl file readable");
    // Every non-empty line is a JSON object.
    let line_count = out.lines().filter(|l| !l.trim().is_empty()).count();
    assert!(line_count >= 1, "jsonl must have at least one line: {out}");
    for line in out.lines().filter(|l| !l.trim().is_empty()) {
        serde_json::from_str::<serde_json::Value>(line)
            .unwrap_or_else(|e| panic!("invalid jsonl line {line:?}: {e}"));
    }
    assert!(out.contains("src/lib.rs"));
}

#[test]
fn write_export_dispatches_json_format_to_file() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("out.json");
    let args = export_args(ExportFormat::Json, Some(path.clone()));

    write_export(&sample_export_data(), &ScanOptions::default(), &args)
        .expect("write_export json succeeds");

    let out = fs::read_to_string(&path).expect("json file readable");
    let v: serde_json::Value = serde_json::from_str(&out).expect("output is valid json");
    assert!(
        v.is_object() || v.is_array(),
        "json must parse as object or array"
    );
    assert!(out.contains("src/lib.rs"));
}

#[test]
fn write_export_dispatches_cyclonedx_format_to_file() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("out.cdx.json");
    let args = export_args(ExportFormat::Cyclonedx, Some(path.clone()));

    write_export(&sample_export_data(), &ScanOptions::default(), &args)
        .expect("write_export cyclonedx succeeds");

    let out = fs::read_to_string(&path).expect("cdx file readable");
    let v: serde_json::Value = serde_json::from_str(&out).expect("output is valid json");
    assert_eq!(
        v["bomFormat"].as_str(),
        Some("CycloneDX"),
        "cyclonedx receipt must declare bomFormat"
    );
}

// ---------------------------------------------------------------------------
// write_lang_json_to_file — happy path
// ---------------------------------------------------------------------------

#[test]
fn write_lang_json_to_file_emits_valid_receipt_json() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("lang.json");
    let scan = scan_args_stub();
    let args_meta = LangArgsMeta {
        format: "json".to_string(),
        top: 0,
        with_files: true,
        children: ChildrenMode::Collapse,
    };

    write_lang_json_to_file(&path, &sample_lang_report(), &scan, &args_meta)
        .expect("lang json file write succeeds");

    let raw = fs::read_to_string(&path).expect("lang json readable");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
    assert_eq!(v["mode"], "lang");
    assert!(v["schema_version"].is_number());
    // `LangReport` is `#[serde(flatten)]` into `LangReceipt`, so its
    // fields appear at the top level.
    assert!(v["rows"].is_array());
}

// ---------------------------------------------------------------------------
// write_module_json_to_file — RedactMode::All hashes module names
// ---------------------------------------------------------------------------

#[test]
fn write_module_json_to_file_with_redact_all_hashes_module_rows_and_roots() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("module.json");
    let scan = scan_args_stub();
    let args_meta = ModuleArgsMeta {
        format: "json".to_string(),
        module_roots: vec!["crates/foo".to_string(), "crates/bar".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
    };

    write_module_json_to_file(
        &path,
        &sample_module_report(),
        &scan,
        &args_meta,
        RedactMode::All,
    )
    .expect("module json file write succeeds");

    let raw = fs::read_to_string(&path).expect("module json readable");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("valid json");

    // Module roots in the args metadata and report should be hashed.
    let args_roots: Vec<String> = v["args"]["module_roots"]
        .as_array()
        .expect("args.module_roots is array")
        .iter()
        .map(|r| r.as_str().unwrap().to_string())
        .collect();
    assert!(
        !args_roots.iter().any(|r| r.contains('/')),
        "redacted module roots must not contain '/': {args_roots:?}"
    );
    assert_eq!(
        args_roots,
        vec![short_hash("crates/foo"), short_hash("crates/bar")]
    );

    // `ModuleReport` is flattened into `ModuleReceipt` JSON, so
    // `module_roots` and `rows` sit at the top level.
    let report_roots: Vec<String> = v["module_roots"]
        .as_array()
        .expect("module_roots is array")
        .iter()
        .map(|r| r.as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        report_roots,
        vec![short_hash("crates/foo"), short_hash("crates/bar")]
    );

    // Each row's module name should be hashed and stable.
    let rows = v["rows"].as_array().expect("rows is array");
    let row_modules: Vec<&str> = rows
        .iter()
        .map(|r| r["module"].as_str().expect("row.module is string"))
        .collect();
    assert_eq!(
        row_modules,
        vec![short_hash("crates/foo"), short_hash("crates/bar")]
    );
}

#[test]
fn write_module_json_to_file_without_redaction_preserves_module_names() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("module.json");
    let scan = scan_args_stub();
    let args_meta = ModuleArgsMeta {
        format: "json".to_string(),
        module_roots: vec!["crates/foo".to_string(), "crates/bar".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
    };

    write_module_json_to_file(
        &path,
        &sample_module_report(),
        &scan,
        &args_meta,
        RedactMode::None,
    )
    .expect("module json file write succeeds");

    let raw = fs::read_to_string(&path).expect("module json readable");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("valid json");

    let row_modules: Vec<&str> = v["rows"]
        .as_array()
        .expect("rows is array")
        .iter()
        .map(|r| r["module"].as_str().expect("row.module is string"))
        .collect();
    assert_eq!(row_modules, vec!["crates/foo", "crates/bar"]);
}

#[test]
fn write_module_json_to_file_with_redact_paths_does_not_hash_module_names() {
    // `RedactMode::Paths` only redacts scan-input paths; module names must
    // remain unchanged in the module-receipt body. Only `RedactMode::All`
    // hashes module identifiers.
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("module.json");
    let scan = scan_args_stub();
    let args_meta = ModuleArgsMeta {
        format: "json".to_string(),
        module_roots: vec!["crates/foo".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
    };

    write_module_json_to_file(
        &path,
        &sample_module_report(),
        &scan,
        &args_meta,
        RedactMode::Paths,
    )
    .expect("module json file write succeeds");

    let raw = fs::read_to_string(&path).expect("module json readable");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("valid json");

    let row_modules: Vec<&str> = v["rows"]
        .as_array()
        .expect("rows is array")
        .iter()
        .map(|r| r["module"].as_str().expect("row.module is string"))
        .collect();
    assert!(
        row_modules.iter().any(|m| m.contains('/')),
        "RedactMode::Paths must not hash module identifiers: {row_modules:?}"
    );
}
