use std::fs;
use tokmd_config::{ChildIncludeMode, ExportFormat, GlobalArgs, RedactMode};
use tokmd_format::write_export;
use tokmd_types::{ExportArgs, ExportData, FileKind, FileRow};

#[test]
fn verify_csv_output() {
    let rows = vec![
        FileRow {
            path: "test.rs".to_string(),
            module: "test".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 2,
            blanks: 1,
            lines: 13,
            bytes: 100,
            tokens: 50,
        },
        FileRow {
            path: "child.rs".to_string(),
            module: "test".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 5,
            comments: 0,
            blanks: 0,
            lines: 5,
            bytes: 50,
            tokens: 25,
        },
    ];

    let export_data = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 0,
        children: ChildIncludeMode::Separate,
    };

    let global_args = GlobalArgs::default();

    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("verify_tokmd_export.csv");

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
    assert_eq!(lines.len(), 3); // Header + 2 rows
    assert_eq!(
        lines[0],
        "path,module,lang,kind,code,comments,blanks,lines,bytes,tokens"
    );
    assert_eq!(lines[1], "test.rs,test,Rust,parent,10,2,1,13,100,50");
    assert_eq!(lines[2], "child.rs,test,Rust,child,5,0,0,5,50,25");

    fs::remove_file(file_path).unwrap();
}
