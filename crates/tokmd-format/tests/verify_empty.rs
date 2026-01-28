use tokmd_format::{write_export};
use tokmd_types::{ExportArgs, ExportData};
use tokmd_config::{GlobalArgs, RedactMode, ChildIncludeMode, ExportFormat};
use std::fs;

#[test]
fn verify_empty_csv_output() {
    let export_data = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 0,
        children: ChildIncludeMode::Separate,
    };

    let global_args = GlobalArgs::default();

    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("verify_tokmd_export_empty.csv");

    let export_args = ExportArgs {
        paths: vec![],
        format: ExportFormat::Csv,
        out: Some(file_path.clone()),
        module_roots: vec![],
        module_depth: 0,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    };

    write_export(&export_data, &global_args, &export_args).unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    println!("CSV Content:\n{}", content);

    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 1); // Header only
    assert_eq!(lines[0], "path,module,lang,kind,code,comments,blanks,lines,bytes,tokens");

    fs::remove_file(file_path).unwrap();
}
