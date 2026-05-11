//! Analysis request construction and option parsing.

use anyhow::Result;
use tokmd_analysis as analysis;
use tokmd_analysis_types::AnalysisArgsMeta;

use crate::error;
use crate::settings::AnalyzeSettings;

pub(super) fn build_analysis_request(
    analyze: &AnalyzeSettings,
) -> Result<analysis::AnalysisRequest> {
    let (preset, preset_meta) = parse_analysis_preset(&analyze.preset)?;
    let (granularity, granularity_meta) = parse_import_granularity(&analyze.granularity)?;
    let effort = parse_effort_request(analyze, &preset_meta)?;

    Ok(analysis::AnalysisRequest {
        preset,
        args: AnalysisArgsMeta {
            preset: preset_meta,
            format: "json".to_string(),
            window_tokens: analyze.window,
            git: analyze.git,
            max_files: analyze.max_files,
            max_bytes: analyze.max_bytes,
            max_file_bytes: analyze.max_file_bytes,
            max_commits: analyze.max_commits,
            max_commit_files: analyze.max_commit_files,
            import_granularity: granularity_meta,
        },
        limits: analysis::AnalysisLimits {
            max_files: analyze.max_files,
            max_bytes: analyze.max_bytes,
            max_file_bytes: analyze.max_file_bytes,
            max_commits: analyze.max_commits,
            max_commit_files: analyze.max_commit_files,
        },
        window_tokens: analyze.window,
        git: analyze.git,
        import_granularity: granularity,
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.80,
        near_dup_max_files: 2000,
        near_dup_scope: analysis::NearDupScope::Module,
        near_dup_max_pairs: None,
        near_dup_exclude: Vec::new(),
        effort,
    })
}

pub(crate) fn parse_analysis_preset(value: &str) -> Result<(analysis::AnalysisPreset, String)> {
    let normalized = value.trim().to_ascii_lowercase();
    let preset = match normalized.as_str() {
        "receipt" => analysis::AnalysisPreset::Receipt,
        "estimate" => analysis::AnalysisPreset::Estimate,
        "health" => analysis::AnalysisPreset::Health,
        "risk" => analysis::AnalysisPreset::Risk,
        "supply" => analysis::AnalysisPreset::Supply,
        "architecture" => analysis::AnalysisPreset::Architecture,
        "topics" => analysis::AnalysisPreset::Topics,
        "security" => analysis::AnalysisPreset::Security,
        "identity" => analysis::AnalysisPreset::Identity,
        "git" => analysis::AnalysisPreset::Git,
        "deep" => analysis::AnalysisPreset::Deep,
        "fun" => analysis::AnalysisPreset::Fun,
        _ => {
            return Err(error::TokmdError::invalid_field(
                "preset",
                "'receipt', 'estimate', 'health', 'risk', 'supply', 'architecture', 'topics', 'security', 'identity', 'git', 'deep', or 'fun'",
            )
            .into());
        }
    };
    Ok((preset, normalized))
}

fn parse_import_granularity(value: &str) -> Result<(analysis::ImportGranularity, String)> {
    let normalized = value.trim().to_ascii_lowercase();
    let granularity = match normalized.as_str() {
        "module" => analysis::ImportGranularity::Module,
        "file" => analysis::ImportGranularity::File,
        _ => {
            return Err(
                error::TokmdError::invalid_field("granularity", "'module' or 'file'").into(),
            );
        }
    };
    Ok((granularity, normalized))
}

pub(crate) fn parse_effort_request(
    analyze: &AnalyzeSettings,
    preset: &str,
) -> Result<Option<analysis::EffortRequest>> {
    let request = analysis::EffortRequest::default();
    let requested = preset == "estimate"
        || analyze.effort_model.is_some()
        || analyze.effort_layer.is_some()
        || analyze.effort_base_ref.is_some()
        || analyze.effort_head_ref.is_some()
        || analyze.effort_monte_carlo.unwrap_or(false)
        || analyze.effort_mc_iterations.is_some()
        || analyze.effort_mc_seed.is_some();

    if !requested {
        return Ok(None);
    }

    if (analyze.effort_base_ref.is_some() && analyze.effort_head_ref.is_none())
        || (analyze.effort_base_ref.is_none() && analyze.effort_head_ref.is_some())
    {
        return Err(error::TokmdError::invalid_field(
            "effort_base_ref/effort_head_ref",
            "both effort_base_ref and effort_head_ref must be provided together",
        )
        .into());
    }

    let model = analyze
        .effort_model
        .as_deref()
        .map(parse_effort_model)
        .transpose()?
        .unwrap_or(request.model);
    let layer = analyze
        .effort_layer
        .as_deref()
        .map(parse_effort_layer)
        .transpose()?
        .unwrap_or(request.layer);

    let monte_carlo = analyze.effort_monte_carlo.unwrap_or(false);
    let mc_iterations = analyze
        .effort_mc_iterations
        .unwrap_or(request.mc_iterations);

    if mc_iterations == 0 {
        return Err(error::TokmdError::invalid_field(
            "effort_mc_iterations",
            "must be greater than 0",
        )
        .into());
    }

    Ok(Some(analysis::EffortRequest {
        model,
        layer,
        base_ref: analyze.effort_base_ref.clone(),
        head_ref: analyze.effort_head_ref.clone(),
        monte_carlo,
        mc_iterations,
        mc_seed: analyze.effort_mc_seed,
    }))
}

fn parse_effort_model(value: &str) -> Result<analysis::EffortModelKind> {
    match value.trim().to_ascii_lowercase().as_str() {
        "cocomo81-basic" => Ok(analysis::EffortModelKind::Cocomo81Basic),
        "cocomo2-early" | "ensemble" => Err(error::TokmdError::invalid_field(
            "effort_model",
            "only 'cocomo81-basic' is currently supported",
        )
        .into()),
        _ => Err(error::TokmdError::invalid_field("effort_model", "'cocomo81-basic'").into()),
    }
}

fn parse_effort_layer(value: &str) -> Result<analysis::EffortLayer> {
    match value.trim().to_ascii_lowercase().as_str() {
        "headline" => Ok(analysis::EffortLayer::Headline),
        "why" => Ok(analysis::EffortLayer::Why),
        "full" => Ok(analysis::EffortLayer::Full),
        _ => Err(
            error::TokmdError::invalid_field("effort_layer", "'headline', 'why', or 'full'").into(),
        ),
    }
}
