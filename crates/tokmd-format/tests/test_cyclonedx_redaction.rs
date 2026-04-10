use serde_json::Value;
use tokmd_format::write_export_cyclonedx_to;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow, RedactMode};

#[test]
fn test_write_export_cyclonedx_honors_redact_mode() {
    let export = ExportData {
        rows: vec![FileRow {
            path: "src/secret_path/module.rs".to_string(),
            module: "secret_module".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 100,
            comments: 10,
            blanks: 5,
            lines: 115,
            bytes: 2000,
            tokens: 500,
        }],
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };

    let mut buf = Vec::new();
    write_export_cyclonedx_to(&mut buf, &export, RedactMode::Paths).unwrap();
    let json: Value = serde_json::from_slice(&buf).unwrap();
    let name = json["components"][0]["name"].as_str().unwrap();
    assert_ne!(name, "src/secret_path/module.rs");

    let mut buf = Vec::new();
    write_export_cyclonedx_to(&mut buf, &export, RedactMode::All).unwrap();
    let json: Value = serde_json::from_slice(&buf).unwrap();
    let name = json["components"][0]["name"].as_str().unwrap();
    let group = json["components"][0]["group"].as_str().unwrap();
    assert_ne!(name, "src/secret_path/module.rs");
    assert_ne!(group, "secret_module");
}
