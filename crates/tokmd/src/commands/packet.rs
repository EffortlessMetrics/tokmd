//! `tokmd packet generate` — thin evidence packet orchestrator.
//!
//! This command owns no new analysis model. It coordinates the existing
//! `analyze`, `context`, `syntax`, and `evidence-packet` surfaces so a complete
//! `sensors/tokmd/` packet can be produced from one command, keeping the same
//! base/head refs and path scope across every generated artifact.

use std::path::Path;

use anyhow::{Context, Result};

use crate::analysis_utils;
use crate::cli;
use crate::commands::{analyze, context, evidence_packet};
use crate::progress;

pub(crate) fn handle(args: cli::PacketArgs, global: &cli::GlobalArgs) -> Result<()> {
    match args.command {
        cli::PacketCommand::Generate(generate) => generate_packet(generate, global),
    }
}

fn generate_packet(args: cli::PacketGenerateArgs, global: &cli::GlobalArgs) -> Result<()> {
    let out_dir = args.out.clone();
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create packet directory {}", out_dir.display()))?;

    let analyze_md = out_dir.join("analyze.md");
    let analyze_json = out_dir.join("analyze.json");
    let context_md = out_dir.join("context.md");
    let syntax_json = out_dir.join("syntax.json");
    let manifest = out_dir.join("manifest.json");

    // Regenerating into an existing packet directory must not blend stale
    // optional evidence with a fresh run. `evidence-packet` auto-detects
    // `syntax.json` in the packet directory, so a prior run's file would
    // otherwise be silently included as if it belonged to this invocation
    // (under `--no-syntax`, or under `--syntax` when this run fails to write a
    // fresh one). Only ever removes prior-run output, never current evidence.
    if syntax_json.exists() {
        std::fs::remove_file(&syntax_json)
            .with_context(|| format!("failed to clear stale {}", syntax_json.display()))?;
    }

    // Orchestrator-level progress: the live spinner is owned by each delegated
    // sub-command (analyze/context each create their own), so the packet frames
    // every stage with machine-readable `tokmd.progress` events on stderr only
    // (subject to TOKMD_PROGRESS_EVENTS) without drawing a competing spinner.
    // stdout stays reserved for the evidence-packet manifest.

    // 1. analyze: one analysis pass, rendered to both the JSON and Markdown
    //    artifacts so the receipts cannot disagree.
    progress::emit_stage("Generating analysis receipt...");
    let analyze_args = analyze_args(&args);
    let receipt = analyze::build_receipt(&analyze_args, global)
        .context("failed to build analysis receipt for packet")?;
    analysis_utils::write_analysis_to_path(
        &receipt,
        &analyze_json,
        tokmd_types::AnalysisFormat::Json,
    )
    .with_context(|| format!("failed to write {}", analyze_json.display()))?;
    analysis_utils::write_analysis_to_path(&receipt, &analyze_md, tokmd_types::AnalysisFormat::Md)
        .with_context(|| format!("failed to write {}", analyze_md.display()))?;

    // 2. context: reuse the context command with output redirected to the packet.
    progress::emit_stage("Generating context artifact...");
    context::handle(context_args(&args, &context_md), global)
        .context("failed to generate packet context artifact")?;

    // 3. syntax: optional advisory evidence. When requested, always reference
    //    the artifact so that a failed or unavailable syntax step degrades the
    //    packet to `partial` with a named missing-artifact warning (the
    //    evidence-packet producer contract), rather than silently reporting
    //    `complete`. The stale-clear above guarantees the referenced file
    //    reflects only this run.
    let syntax_arg = if args.want_syntax() {
        progress::emit_stage("Generating syntax evidence...");
        if let Err(err) = write_syntax(&args, &syntax_json) {
            eprintln!("warning: syntax evidence unavailable: {err}");
        }
        Some(syntax_json.clone())
    } else {
        None
    };

    // 4. evidence-packet: index and validate the artifacts. This prints the
    //    manifest JSON and exits nonzero when the packet status is `failed`.
    progress::emit_stage("Indexing evidence packet...");
    let packet_args = cli::EvidencePacketArgs {
        preset: args.preset,
        base: args.base.clone(),
        head: args.head.clone(),
        output: manifest,
        analyze_md: Some(analyze_md),
        analyze_json: Some(analyze_json),
        context_md: Some(context_md),
        syntax_json: syntax_arg,
        context_budget: args.context_budget.clone(),
        paths: args.paths.clone(),
    };
    evidence_packet::handle(packet_args)?;
    progress::emit_stage_finish();
    Ok(())
}

/// Build analyze arguments scoped to the packet request.
///
/// The non-`Option` fields mirror the `tokmd analyze` clap defaults so the
/// generated receipts match a direct `analyze` invocation.
fn analyze_args(args: &cli::PacketGenerateArgs) -> cli::CliAnalyzeArgs {
    // Effort refs are only meaningful for effort-aware presets (Estimate /
    // BunUb). Setting them for every preset forces an effort request, which then
    // trips validate_effort_refs in non-git builds even when the preset has no
    // notion of effort (e.g. topics, receipt). This mirrors the estimate-preset
    // gating in analyze::parse_effort_request.
    let wants_effort = matches!(
        args.preset,
        cli::AnalysisPreset::Estimate | cli::AnalysisPreset::BunUb
    );
    cli::CliAnalyzeArgs {
        inputs: args.paths.clone(),
        preset: Some(args.preset),
        // The receipt is rendered to both formats; record JSON in the
        // machine-readable artifact's metadata.
        format: Some(cli::AnalysisFormat::Json),
        window: None,
        git: false,
        no_git: false,
        output_dir: None,
        max_files: None,
        max_bytes: None,
        max_file_bytes: None,
        max_commits: None,
        max_commit_files: None,
        granularity: None,
        effort_model: None,
        effort_layer: None,
        effort_base_ref: wants_effort.then(|| args.base.clone()),
        effort_head_ref: wants_effort.then(|| args.head.clone()),
        monte_carlo: false,
        mc_iterations: None,
        mc_seed: None,
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.80,
        near_dup_max_files: 2000,
        near_dup_scope: None,
        near_dup_max_pairs: 10000,
        near_dup_exclude: Vec::new(),
        explain: None,
    }
}

/// Build context arguments writing to the packet `context.md`.
///
/// The non-`Option` fields mirror the `tokmd context` clap defaults.
fn context_args(args: &cli::PacketGenerateArgs, output: &Path) -> cli::CliContextArgs {
    cli::CliContextArgs {
        paths: Some(args.paths.clone()),
        budget: args.context_budget.clone(),
        strategy: cli::ContextStrategy::default(),
        rank_by: cli::ValueMetric::default(),
        output_mode: cli::ContextOutput::default(),
        compress: false,
        no_smart_exclude: false,
        module_roots: None,
        module_depth: None,
        git: false,
        no_git: false,
        max_commits: 1000,
        max_commit_files: 100,
        output: Some(output.to_path_buf()),
        // Regenerating a packet should overwrite the previous context artifact.
        force: true,
        bundle_dir: None,
        max_output_bytes: 10_485_760,
        log: None,
        max_file_pct: 0.15,
        max_file_tokens: None,
        require_git_scores: false,
    }
}

#[cfg(feature = "ast")]
fn write_syntax(args: &cli::PacketGenerateArgs, path: &Path) -> Result<()> {
    use crate::commands::syntax;

    let syntax_args = cli::SyntaxArgs {
        max_bytes: tokmd_analysis::ast::DEFAULT_MAX_SYNTAX_BYTES,
        include_generated_vendor: false,
        paths: args.paths.clone(),
    };
    let packet = syntax::build_syntax_packet(&syntax_args, &[])?;
    let json = serde_json::to_string_pretty(&packet)?;
    std::fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

#[cfg(not(feature = "ast"))]
fn write_syntax(_args: &cli::PacketGenerateArgs, _path: &Path) -> Result<()> {
    anyhow::bail!("syntax evidence requires the `ast` feature")
}
