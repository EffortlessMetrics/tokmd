use anyhow::{Context, Result, bail};
use tokmd_analysis as analysis;
use tokmd_analysis_types as analysis_types;
use tokmd_config as cli;

use crate::analysis_explain;
use crate::analysis_utils;
use crate::export_bundle;
use tokmd_progress::Progress;

pub(crate) fn handle(args: cli::CliAnalyzeArgs, global: &cli::GlobalArgs) -> Result<()> {
    if let Some(key) = args.explain.as_deref() {
        let normalized = key.trim().to_ascii_lowercase();
        if normalized == "list" || normalized == "all" || normalized == "keys" {
            print!("{}", analysis_explain::catalog());
            return Ok(());
        }

        if let Some(explanation) = analysis_explain::lookup(key) {
            println!("{}", explanation);
            return Ok(());
        }

        bail!(
            "Unknown metric/finding key '{}'. Use --explain list to see supported keys.",
            key
        );
    }

    let progress = Progress::new(!global.no_progress);

    let preset = args.preset.unwrap_or(cli::AnalysisPreset::Receipt);
    let format = args.format.unwrap_or(cli::AnalysisFormat::Md);
    let git_flag = if args.git {
        Some(true)
    } else if args.no_git {
        Some(false)
    } else {
        None
    };
    let granularity = args.granularity.unwrap_or(cli::ImportGranularity::Module);

    progress.set_message("Loading export data...");
    let bundle = export_bundle::load_export_from_inputs(&args.inputs, global)?;
    let source = analysis_types::AnalysisSource {
        inputs: args
            .inputs
            .iter()
            .map(|p| p.display().to_string())
            .collect(),
        export_path: bundle.export_path.as_ref().map(|p| p.display().to_string()),
        base_receipt_path: bundle.export_path.as_ref().map(|p| p.display().to_string()),
        export_schema_version: bundle.meta.schema_version,
        export_generated_at_ms: bundle.meta.generated_at_ms,
        base_signature: None,
        module_roots: bundle.meta.module_roots.clone(),
        module_depth: bundle.meta.module_depth,
        children: analysis_utils::child_include_to_string(bundle.meta.children),
    };
    let args_meta = analysis_types::AnalysisArgsMeta {
        preset: analysis_utils::preset_to_string(preset),
        format: analysis_utils::format_to_string(format),
        window_tokens: args.window,
        git: git_flag,
        max_files: args.max_files,
        max_bytes: args.max_bytes,
        max_file_bytes: args.max_file_bytes,
        max_commits: args.max_commits,
        max_commit_files: args.max_commit_files,
        import_granularity: analysis_utils::granularity_to_string(granularity),
    };
    let near_dup_scope = match args.near_dup_scope {
        Some(cli::NearDupScope::Module) | None => analysis::NearDupScope::Module,
        Some(cli::NearDupScope::Lang) => analysis::NearDupScope::Lang,
        Some(cli::NearDupScope::Global) => analysis::NearDupScope::Global,
    };
    let request = analysis::AnalysisRequest {
        preset: analysis_utils::map_preset(preset),
        args: args_meta,
        limits: analysis::AnalysisLimits {
            max_files: args.max_files,
            max_bytes: args.max_bytes,
            max_file_bytes: args.max_file_bytes,
            max_commits: args.max_commits,
            max_commit_files: args.max_commit_files,
        },
        window_tokens: args.window,
        git: git_flag,
        import_granularity: analysis_utils::map_granularity(granularity),
        detail_functions: args.detail_functions,
        near_dup: args.near_dup,
        near_dup_threshold: args.near_dup_threshold,
        near_dup_max_files: args.near_dup_max_files,
        near_dup_scope,
        near_dup_max_pairs: Some(args.near_dup_max_pairs),
        near_dup_exclude: args.near_dup_exclude.clone(),
    };
    let ctx = analysis::AnalysisContext {
        export: bundle.export,
        root: bundle.root,
        source,
    };
    progress.set_message("Running analysis...");
    let receipt = analysis::analyze(ctx, request)?;

    progress.finish_and_clear();

    if let Some(output_dir) = args.output_dir {
        std::fs::create_dir_all(&output_dir)
            .context("Failed to create analysis output directory")?;
        analysis_utils::write_analysis_output(&receipt, &output_dir, format)?;
    } else {
        analysis_utils::write_analysis_stdout(&receipt, format)?;
    }

    Ok(())
}
