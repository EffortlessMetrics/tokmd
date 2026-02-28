use anyhow::Result;
use tokmd_analysis as analysis;
use tokmd_badge::badge_svg;
use tokmd_config as cli;

use crate::analysis_utils;
use crate::export_bundle;

pub(crate) fn handle(args: cli::BadgeArgs, global: &cli::GlobalArgs) -> Result<()> {
    let metric = args.metric;
    let mut preset = args.preset.unwrap_or(cli::AnalysisPreset::Receipt);
    if metric == cli::BadgeMetric::Hotspot && args.preset.is_none() {
        preset = cli::AnalysisPreset::Risk;
    }
    let git_flag = if args.git {
        Some(true)
    } else if args.no_git {
        Some(false)
    } else if metric == cli::BadgeMetric::Hotspot {
        Some(true)
    } else {
        None
    };

    let bundle = export_bundle::load_export_from_inputs(&args.inputs, global)?;
    let source = tokmd_analysis_types::AnalysisSource {
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
    let args_meta = tokmd_analysis_types::AnalysisArgsMeta {
        preset: analysis_utils::preset_to_string(preset),
        format: "badge".to_string(),
        window_tokens: None,
        git: git_flag,
        max_files: None,
        max_bytes: None,
        max_file_bytes: None,
        max_commits: args.max_commits,
        max_commit_files: args.max_commit_files,
        import_granularity: "module".to_string(),
    };
    let request = analysis::AnalysisRequest {
        preset: analysis_utils::map_preset(preset)?,
        args: args_meta,
        limits: analysis::AnalysisLimits {
            max_files: None,
            max_bytes: None,
            max_file_bytes: None,
            max_commits: args.max_commits,
            max_commit_files: args.max_commit_files,
        },
        window_tokens: None,
        git: git_flag,
        import_granularity: analysis::ImportGranularity::Module,
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.80,
        near_dup_max_files: 2000,
        near_dup_scope: analysis::NearDupScope::Module,
        near_dup_max_pairs: None,
        near_dup_exclude: Vec::new(),
    };
    let ctx = analysis::AnalysisContext {
        export: bundle.export,
        root: bundle.root,
        source,
    };
    let receipt = analysis::analyze(ctx, request)?;

    let value = match metric {
        cli::BadgeMetric::Lines => receipt
            .derived
            .as_ref()
            .map(|d| d.totals.lines.to_string())
            .unwrap_or_else(|| "0".to_string()),
        cli::BadgeMetric::Tokens => receipt
            .derived
            .as_ref()
            .map(|d| d.totals.tokens.to_string())
            .unwrap_or_else(|| "0".to_string()),
        cli::BadgeMetric::Bytes => receipt
            .derived
            .as_ref()
            .map(|d| d.totals.bytes.to_string())
            .unwrap_or_else(|| "0".to_string()),
        cli::BadgeMetric::Doc => receipt
            .derived
            .as_ref()
            .map(|d| format!("{:.1}%", d.doc_density.total.ratio * 100.0))
            .unwrap_or_else(|| "0%".to_string()),
        cli::BadgeMetric::Blank => receipt
            .derived
            .as_ref()
            .map(|d| format!("{:.1}%", d.whitespace.total.ratio * 100.0))
            .unwrap_or_else(|| "0%".to_string()),
        cli::BadgeMetric::Hotspot => receipt
            .git
            .as_ref()
            .and_then(|g| g.hotspots.first())
            .map(|h| h.score.to_string())
            .unwrap_or_else(|| "n/a".to_string()),
    };

    let label = badge_metric_label(metric);
    let svg = badge_svg(label, &value);

    if let Some(output) = args.output {
        std::fs::write(output, svg)?;
    } else {
        print!("{}", svg);
    }

    Ok(())
}

fn badge_metric_label(metric: cli::BadgeMetric) -> &'static str {
    match metric {
        cli::BadgeMetric::Lines => "lines",
        cli::BadgeMetric::Tokens => "tokens",
        cli::BadgeMetric::Bytes => "bytes",
        cli::BadgeMetric::Doc => "doc",
        cli::BadgeMetric::Blank => "blank",
        cli::BadgeMetric::Hotspot => "hotspot",
    }
}
