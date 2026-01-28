use std::path::PathBuf;

use anyhow::{Context, Result};
use tokmd_analysis as analysis;
use tokmd_analysis_types as analysis_types;
use tokmd_config as cli;
use tokmd_format as format;
use tokmd_model as model;
use tokmd_scan as scan;

use crate::analysis_utils;

pub(crate) fn handle(args: cli::RunArgs, global: &cli::GlobalArgs) -> Result<()> {
    // 1. Scan once
    let languages = scan::scan(&args.paths, global)?;

    // 2. Determine output directory
    let output_dir = if let Some(d) = args.output_dir {
        std::fs::create_dir_all(&d).context("Failed to create output directory")?;
        d
    } else {
        let run_id = args.name.unwrap_or_else(|| format!("run-{}", now_ms()));
        let local_runs = PathBuf::from(".runs/tokmd").join(&run_id);

        // Try repo-local first, fall back to OS state dir if creation fails
        if std::fs::create_dir_all(&local_runs).is_ok() {
            local_runs
        } else {
            let state_dir = dirs::state_dir()
                .or_else(dirs::data_local_dir)
                .unwrap_or_else(std::env::temp_dir);
            let fallback = state_dir.join("tokmd").join("runs").join(run_id);
            std::fs::create_dir_all(&fallback).context("Failed to create output directory")?;
            fallback
        }
    };
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

    // Get redact mode - applies to scan args in all receipts (lang.json, module.json, export.jsonl)
    let redact_mode = args.redact.unwrap_or(cli::RedactMode::None);
    let scan_args = make_scan_args(&args.paths, global, redact_mode);

    // 4. Write artifacts using tokmd-format for consistency

    // Write lang.json
    let lang_path = output_dir.join("lang.json");
    let lang_args_meta = tokmd_types::LangArgsMeta {
        format: "json".to_string(),
        top: 0,
        with_files: false,
        children: cli::ChildrenMode::Collapse,
    };
    format::write_lang_json_to_file(&lang_path, &lang_report, &scan_args, &lang_args_meta)
        .context("Failed to write lang.json")?;

    // Write module.json
    let module_path = output_dir.join("module.json");
    let module_args_meta = tokmd_types::ModuleArgsMeta {
        format: "json".to_string(),
        top: 0,
        module_roots: vec!["crates".to_string(), "packages".to_string()],
        module_depth: 2,
        children: cli::ChildIncludeMode::Separate,
    };
    format::write_module_json_to_file(&module_path, &module_report, &scan_args, &module_args_meta)
        .context("Failed to write module.json")?;

    // Write export.jsonl (with redaction support)
    let export_path = output_dir.join("export.jsonl");
    let export_args_meta = tokmd_types::ExportArgsMeta {
        format: cli::ExportFormat::Jsonl,
        module_roots: vec!["crates".to_string(), "packages".to_string()],
        module_depth: 2,
        children: cli::ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: redact_mode,
        strip_prefix: None,
        strip_prefix_redacted: false,
    };
    format::write_export_jsonl_to_file(&export_path, &export_data, &scan_args, &export_args_meta)
        .context("Failed to write export.jsonl")?;

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
            inputs: args.paths.iter().map(|p| normalize_scan_input(p)).collect(),
            export_path: Some("export.jsonl".to_string()),
            base_receipt_path: Some("export.jsonl".to_string()),
            export_schema_version: Some(tokmd_types::SCHEMA_VERSION),
            export_generated_at_ms: None,
            base_signature: None,
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

/// Normalize a path to forward slashes and strip leading `./` for cross-platform stability.
fn normalize_scan_input(p: &std::path::Path) -> String {
    let s = p.display().to_string().replace('\\', "/");
    s.strip_prefix("./").unwrap_or(&s).to_string()
}

fn make_scan_args(
    paths: &[PathBuf],
    global: &cli::GlobalArgs,
    redact: cli::RedactMode,
) -> tokmd_types::ScanArgs {
    let should_redact = redact == cli::RedactMode::Paths || redact == cli::RedactMode::All;
    let excluded_redacted = should_redact && !global.excluded.is_empty();

    let mut args = tokmd_types::ScanArgs {
        paths: paths.iter().map(|p| normalize_scan_input(p)).collect(),
        excluded: if should_redact {
            global
                .excluded
                .iter()
                .map(|p| format::short_hash(p))
                .collect()
        } else {
            global.excluded.clone()
        },
        excluded_redacted,
        config: global.config,
        hidden: global.hidden,
        no_ignore: global.no_ignore,
        no_ignore_parent: global.no_ignore || global.no_ignore_parent,
        no_ignore_dot: global.no_ignore || global.no_ignore_dot,
        no_ignore_vcs: global.no_ignore || global.no_ignore_vcs,
        treat_doc_strings_as_comments: global.treat_doc_strings_as_comments,
    };

    if should_redact {
        args.paths = args
            .paths
            .iter()
            .map(|p| format::redact_path(p))
            .collect();
    }

    args
}
