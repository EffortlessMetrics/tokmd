use std::path::PathBuf;
use std::time::Instant;
use tokmd_config::{ChildIncludeMode, ConfigMode, ExportFormat, GlobalArgs, RedactMode};
use tokmd_format::write_export;
use tokmd_types::{ExportArgs, ExportData, FileKind, FileRow};

#[test]
fn bench_csv_export() {
    // Generate data
    let count = 200_000;
    let mut rows = Vec::with_capacity(count);
    for i in 0..count {
        rows.push(FileRow {
            path: format!("src/module/file_{}.rs", i),
            module: "src/module".to_string(),
            lang: "Rust".to_string(),
            kind: if i % 2 == 0 {
                FileKind::Parent
            } else {
                FileKind::Child
            },
            code: i * 10,
            comments: i,
            blanks: 5,
            lines: i * 11 + 5,
            bytes: i * 100,
            tokens: i * 50,
        });
    }

    let export_data = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 0,
        children: ChildIncludeMode::Separate,
    };

    let global_args = GlobalArgs {
        config: ConfigMode::None,
        excluded: vec![],
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
        verbose: 0,
    };

    let export_args = ExportArgs {
        paths: vec![],
        format: ExportFormat::Csv,
        out: Some(PathBuf::from("/dev/null")),
        module_roots: vec![],
        module_depth: 0,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    };

    let start = Instant::now();
    write_export(&export_data, &global_args, &export_args).unwrap();
    let duration = start.elapsed();

    println!("Time to write {} rows: {:.2?}", count, duration);
}
