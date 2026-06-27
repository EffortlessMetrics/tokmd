use std::path::PathBuf;
use tokmd_settings::ScanOptions;
use tokmd_types::{ChildIncludeMode, ExportArgs, ExportData, ExportFormat, RedactMode};

#[test]
fn strip_prefix_is_redacted_when_mode_is_paths_or_all() {
    let export = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };

    let args = ExportArgs {
        paths: vec![".".into()],
        format: ExportFormat::Json,
        output: None,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        meta: true,
        redact: RedactMode::Paths,
        strip_prefix: Some(PathBuf::from("my/secret/prefix")),
    };

    let global = ScanOptions::default();

    let mut buf = Vec::new();
    tokmd_format::write_export_json_to(&mut buf, &export, &global, &args).unwrap();

    let json: serde_json::Value = serde_json::from_slice(&buf).unwrap();
    let strip_prefix = json["args"]["strip_prefix"].as_str().unwrap();
    assert_ne!(strip_prefix, "my/secret/prefix", "Redaction failed");
    assert!(json["args"]["strip_prefix_redacted"].as_bool().unwrap());
}

#[test]
fn strip_prefix_is_redacted_for_jsonl_when_mode_is_paths_or_all() {
    let export = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };

    let args = ExportArgs {
        paths: vec![".".into()],
        format: ExportFormat::Jsonl,
        output: None,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        meta: true,
        redact: RedactMode::Paths,
        strip_prefix: Some(PathBuf::from("my/secret/prefix")),
    };

    let global = ScanOptions::default();

    let mut buf = Vec::new();
    tokmd_format::write_export_jsonl_to(&mut buf, &export, &global, &args).unwrap();

    let out = String::from_utf8(buf).unwrap();
    let json: serde_json::Value = serde_json::from_str(out.lines().next().unwrap()).unwrap();
    let strip_prefix = json["args"]["strip_prefix"].as_str().unwrap();
    assert_ne!(strip_prefix, "my/secret/prefix", "Redaction failed");
    assert!(json["args"]["strip_prefix_redacted"].as_bool().unwrap());
}

#[test]
fn strip_prefix_is_preserved_when_mode_is_none() {
    let export = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };

    let args = ExportArgs {
        paths: vec![".".into()],
        format: ExportFormat::Json,
        output: None,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        meta: true,
        redact: RedactMode::None,
        strip_prefix: Some(PathBuf::from("my/secret/prefix")),
    };

    let global = ScanOptions::default();

    let mut buf = Vec::new();
    tokmd_format::write_export_json_to(&mut buf, &export, &global, &args).unwrap();

    let json: serde_json::Value = serde_json::from_slice(&buf).unwrap();
    let strip_prefix = json["args"]["strip_prefix"].as_str().unwrap();
    assert_eq!(strip_prefix, "my/secret/prefix");
    assert!(json["args"].get("strip_prefix_redacted").is_none());
}
