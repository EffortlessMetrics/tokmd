use std::path::PathBuf;

use anyhow::{Context, Result};
use tokmd_analysis as analysis;
use tokmd_analysis_types as analysis_types;
use tokmd_config as cli;
use tokmd_model as model;
use tokmd_scan as scan;

use crate::analysis_utils;

pub(crate) fn handle(args: cli::RunArgs, global: &cli::GlobalArgs) -> Result<()> {
    // 1. Scan once
    let languages = scan::scan(&args.paths, global)?;

    // 2. Determine output directory
    let output_dir = if let Some(d) = args.output_dir {
        d
    } else {
        let state_dir = dirs::state_dir()
            .or_else(dirs::data_local_dir)
            .unwrap_or_else(std::env::temp_dir);
        let run_id = args.name.unwrap_or_else(|| format!("run-{}", now_ms()));
        state_dir.join("tokmd").join("runs").join(run_id)
    };

    std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;
    println!("Writing run artifacts to: {}", output_dir.display());

    // 3. Generate Reports
    let lang_report = model::create_lang_report(&languages, 0, false, cli::ChildrenMode::Collapse);
    let module_report = model::create_module_report(
        &languages,
        &["crates".to_string(), "packages".to_string()],
        2,
        cli::ChildIncludeMode::Separate,
        0,
    );
    let export_data = model::create_export_data(
        &languages,
        &["crates".to_string(), "packages".to_string()],
        2,
        cli::ChildIncludeMode::Separate,
        None,
        0,
        0,
    );

    // 4. Write artifacts
    let lang_path = output_dir.join("lang.json");
    let module_path = output_dir.join("module.json");

    {
        let receipt = tokmd_types::LangReceipt {
            schema_version: tokmd_types::SCHEMA_VERSION,
            generated_at_ms: now_ms(),
            tool: tokmd_types::ToolInfo::current(),
            mode: "lang".to_string(),
            status: tokmd_types::ScanStatus::Complete,
            warnings: vec![],
            scan: make_scan_args(&args.paths, global),
            args: tokmd_types::LangArgsMeta {
                format: "json".to_string(),
                top: 0,
                with_files: false,
                children: cli::ChildrenMode::Collapse,
            },
            report: lang_report,
        };
        let f = std::fs::File::create(&lang_path)?;
        serde_json::to_writer(f, &receipt)?;
    }
    {
        let receipt = tokmd_types::ModuleReceipt {
            schema_version: tokmd_types::SCHEMA_VERSION,
            generated_at_ms: now_ms(),
            tool: tokmd_types::ToolInfo::current(),
            mode: "module".to_string(),
            status: tokmd_types::ScanStatus::Complete,
            warnings: vec![],
            scan: make_scan_args(&args.paths, global),
            args: tokmd_types::ModuleArgsMeta {
                format: "json".to_string(),
                top: 0,
                module_roots: vec!["crates".to_string(), "packages".to_string()],
                module_depth: 2,
                children: cli::ChildIncludeMode::Separate,
            },
            report: module_report,
        };
        let f = std::fs::File::create(&module_path)?;
        serde_json::to_writer(f, &receipt)?;
    }

    // 4. Write export.jsonl
    let export_path = output_dir.join("export.jsonl");
    {
        let f = std::fs::File::create(&export_path).context("Failed to create export.jsonl")?;
        let mut writer = std::io::BufWriter::new(f);
        use std::io::Write;

        // Header
        let header = serde_json::json!({
            "type": "meta",
            "schema_version": tokmd_types::SCHEMA_VERSION,
            "generated_at_ms": now_ms(),
            "tool": tokmd_types::ToolInfo::current(),
            "mode": "export",
            "status": "complete",
            "warnings": [],
            "scan": make_scan_args(&args.paths, global),
            "args": tokmd_types::ExportArgsMeta {
               format: cli::ExportFormat::Jsonl,
               module_roots: vec!["crates".to_string(), "packages".to_string()],
               module_depth: 2,
               children: cli::ChildIncludeMode::Separate,
               min_code: 0,
               max_rows: 0,
               redact: cli::RedactMode::None,
               strip_prefix: None,
            }
        });
        serde_json::to_writer(&mut writer, &header)?;
        writeln!(&mut writer)?;

        // Rows
        for row in &export_data.rows {
            let mut val = serde_json::to_value(row)?;
            if let Some(obj) = val.as_object_mut() {
                obj.insert("type".to_string(), "row".into());
            }
            serde_json::to_writer(&mut writer, &val)?;
            writeln!(&mut writer)?;
        }
    }

    // 5. Write receipt.json
    let receipt = tokmd_types::RunReceipt {
        schema_version: tokmd_types::SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        lang_file: "lang.json".to_string(),
        module_file: "module.json".to_string(),
        export_file: "export.jsonl".to_string(),
    };
    let receipt_path = output_dir.join("receipt.json");
    let f = std::fs::File::create(&receipt_path)?;
    serde_json::to_writer(f, &receipt)?;

    if let Some(preset) = args.analysis {
        let source = analysis_types::AnalysisSource {
            inputs: args
                .paths
                .iter()
                .map(|p| p.display().to_string())
                .collect(),
            export_path: Some("export.jsonl".to_string()),
            export_schema_version: Some(tokmd_types::SCHEMA_VERSION),
            export_generated_at_ms: None,
            module_roots: export_data.module_roots.clone(),
            module_depth: export_data.module_depth,
            children: analysis_utils::child_include_to_string(export_data.children),
        };
        let args_meta = analysis_types::AnalysisArgsMeta {
            preset: analysis_utils::preset_to_string(preset),
            format: "md+json".to_string(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_file_bytes: None,
            max_commits: None,
            max_commit_files: None,
            import_granularity: "module".to_string(),
        };
        let request = analysis::AnalysisRequest {
            preset: analysis_utils::map_preset(preset),
            args: args_meta,
            limits: analysis::AnalysisLimits::default(),
            window_tokens: None,
            git: None,
            import_granularity: analysis::ImportGranularity::Module,
        };
        let ctx = analysis::AnalysisContext {
            export: export_data.clone(),
            root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            source,
        };
        let receipt = analysis::analyze(ctx, request)?;
        analysis_utils::write_analysis_output(&receipt, &output_dir, cli::AnalysisFormat::Md)?;
        analysis_utils::write_analysis_output(&receipt, &output_dir, cli::AnalysisFormat::Json)?;
    }

    Ok(())
}

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn make_scan_args(paths: &[PathBuf], global: &cli::GlobalArgs) -> tokmd_types::ScanArgs {
    tokmd_types::ScanArgs {
        paths: paths.iter().map(|p| p.display().to_string()).collect(),
        excluded: global.excluded.clone(),
        config: global.config,
        hidden: global.hidden,
        no_ignore: global.no_ignore,
        no_ignore_parent: global.no_ignore || global.no_ignore_parent,
        no_ignore_dot: global.no_ignore || global.no_ignore_dot,
        no_ignore_vcs: global.no_ignore || global.no_ignore_vcs,
        treat_doc_strings_as_comments: global.treat_doc_strings_as_comments,
    }
}
