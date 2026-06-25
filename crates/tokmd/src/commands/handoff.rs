//! Handoff command: Bundle codebase for LLM handoff.
//!
//! Creates a `.handoff/` directory with four artifacts:
//! - `manifest.json`: Bundle metadata, budgets, capabilities
//! - `map.jsonl`: Complete file inventory (streaming)
//! - `intelligence.json`: Tree + hotspots + complexity + derived
//! - `code.txt`: Token-budgeted code bundle

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cli;
use anyhow::{Context, Result, bail};
use tokmd_model as model;
use tokmd_scan as scan;
use tokmd_scan::{add_exclude_pattern, normalize_exclude_pattern};
use tokmd_types::{
    FileKind, HANDOFF_SCHEMA_VERSION, HandoffExcludedPath, HandoffManifest, ToolInfo,
};

use crate::context_pack;
use crate::progress::Progress;

mod capabilities;
mod intelligence;
mod output;

use capabilities::{detect_capabilities, should_compute_git};
use intelligence::build_intelligence;
use output::{
    HandoffLinkInputs, HandoffWorkOrderInputs, write_link_artifacts, write_manifest_json,
    write_payloads, write_work_order,
};

const DEFAULT_TREE_DEPTH: usize = 4;

/// Handle the handoff command.
pub(crate) fn handle(args: cli::HandoffArgs, global: &cli::GlobalArgs) -> Result<()> {
    let progress = Progress::new(!global.no_progress);

    let paths = args
        .paths
        .clone()
        .unwrap_or_else(|| vec![PathBuf::from(".")]);

    // Check output directory
    if args.out_dir.exists() {
        let is_empty = args
            .out_dir
            .read_dir()
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false);
        if !is_empty && !args.force {
            bail!(
                "Output directory is not empty: {}. Use --force to overwrite.",
                args.out_dir.display()
            );
        }
    }

    // Parse budget
    let budget = context_pack::parse_budget(&args.budget)?;

    let root = paths.first().cloned().unwrap_or_else(|| PathBuf::from("."));

    // Scan and create export data
    progress.set_message("Scanning codebase...");
    let mut scan_args = global.clone();
    let excluded_paths = exclude_output_dir(&root, &args.out_dir, &mut scan_args);
    let scan_opts = tokmd_settings::ScanOptions::from(&scan_args);
    let languages = scan::scan(&paths, &scan_opts)?;
    let module_roots = args.module_roots.clone().unwrap_or_default();
    let module_depth = args.module_depth.unwrap_or(2);

    progress.set_message("Building export data...");
    let export = model::create_export_data(
        &languages,
        &module_roots,
        module_depth,
        tokmd_types::ChildIncludeMode::ParentsOnly,
        None,
        0, // no min_code filter
        0, // no max_rows limit
    );

    // Detect capabilities
    progress.set_message("Detecting capabilities...");
    let capabilities = detect_capabilities(&root, &args);

    // Compute git scores if needed
    progress.set_message("Computing git scores...");
    let git_scores = if should_compute_git(&capabilities) {
        tokmd_core::context_git::compute_git_scores(
            &root,
            &export.rows,
            args.max_commits,
            args.max_commit_files,
        )
    } else {
        None
    };

    // Select files for code bundle
    progress.set_message("Selecting files for code bundle...");
    let select_result = context_pack::select_files_with_options(
        &export.rows,
        budget,
        args.strategy,
        args.rank_by,
        git_scores.as_ref(),
        &context_pack::SelectOptions {
            no_smart_exclude: args.no_smart_exclude,
            max_file_pct: args.max_file_pct,
            max_file_tokens: args.max_file_tokens,
            ..Default::default()
        },
    );
    let selected = select_result.selected;
    let smart_excluded_files = select_result.smart_excluded;

    let used_tokens: usize = selected
        .iter()
        .map(|f| f.effective_tokens.unwrap_or(f.tokens))
        .sum();
    let utilization = if budget > 0 {
        (used_tokens as f64 / budget as f64) * 100.0
    } else {
        0.0
    };

    // Build intelligence
    progress.set_message("Building intelligence...");
    let intelligence = build_intelligence(&export, &args, &capabilities, git_scores.as_ref());

    // Write output directory
    progress.set_message("Writing handoff bundle...");
    fs::create_dir_all(&args.out_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            args.out_dir.display()
        )
    })?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let mut payloads = write_payloads(
        &args.out_dir,
        &export,
        &intelligence,
        &selected,
        args.compress,
    )?;
    let packet_local_proof_route = discover_packet_local_proof_route(
        args.proof_route.as_deref(),
        args.review_packet_dir.as_deref(),
    );
    let proof_route = args
        .proof_route
        .as_deref()
        .or(packet_local_proof_route.as_deref());
    let link_inputs = HandoffLinkInputs {
        review_packet_dir: args.review_packet_dir.as_deref(),
        review_packet_check: args.review_packet_check.as_deref(),
        affected: args.affected.as_deref(),
        proof_plan: args.proof_plan.as_deref(),
        proof_route,
        evidence_packet: args.evidence_packet.as_deref(),
    };
    let mut link_artifacts = write_link_artifacts(&args.out_dir, &link_inputs)?;
    payloads.artifacts.append(&mut link_artifacts);
    let total_files = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .count();
    let input_paths: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
    let strategy = format!("{:?}", args.strategy).to_lowercase();
    let rank_by = format!("{:?}", args.rank_by).to_lowercase();
    let intelligence_preset = format!("{:?}", args.preset).to_lowercase();
    let work_order_artifact = write_work_order(
        &args.out_dir,
        &HandoffWorkOrderInputs {
            inputs: &input_paths,
            budget_tokens: budget,
            used_tokens,
            utilization_pct: round_f64(utilization, 2),
            strategy: &strategy,
            rank_by: &rank_by,
            intelligence_preset: &intelligence_preset,
            total_files,
            selected: &selected,
            links: &link_inputs,
        },
    )?;
    payloads.artifacts.push(work_order_artifact);

    // Compute token estimation and audit
    let total_file_bytes: usize = selected.iter().map(|f| f.bytes).sum();
    let token_estimation = tokmd_types::TokenEstimationMeta::from_bytes(total_file_bytes, 4.0);
    let code_audit =
        tokmd_types::TokenAudit::from_output(payloads.code_bytes, total_file_bytes as u64);

    // Write manifest.json
    let manifest = HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: timestamp,
        tool: ToolInfo::current(),
        mode: "handoff".to_string(),
        inputs: input_paths,
        output_dir: args.out_dir.display().to_string(),
        budget_tokens: budget,
        used_tokens,
        utilization_pct: round_f64(utilization, 2),
        strategy,
        rank_by,
        capabilities: capabilities.clone(),
        artifacts: payloads.artifacts,
        included_files: selected.clone(),
        excluded_paths: excluded_paths.clone(),
        excluded_patterns: scan_args.excluded.clone(),
        smart_excluded_files,
        total_files,
        bundled_files: selected.len(),
        intelligence_preset,
        rank_by_effective: if select_result.fallback_reason.is_some() {
            Some(select_result.rank_by_effective.clone())
        } else {
            None
        },
        fallback_reason: select_result.fallback_reason.clone(),
        excluded_by_policy: select_result.excluded_by_policy.clone(),
        token_estimation: Some(token_estimation),
        code_audit: Some(code_audit),
    };

    let manifest_bytes = write_manifest_json(&args.out_dir, &manifest)?;

    progress.finish_and_clear();

    // Print summary
    eprintln!("Wrote handoff bundle to {}", args.out_dir.display());
    eprintln!("  - manifest.json ({} bytes)", manifest_bytes);
    eprintln!("  - map.jsonl ({} bytes)", payloads.map_bytes);
    eprintln!(
        "  - intelligence.json ({} bytes)",
        payloads.intelligence_bytes
    );
    eprintln!("  - code.txt ({} bytes)", payloads.code_bytes);
    eprintln!(
        "  - Token usage: {}/{} ({:.1}%)",
        used_tokens, budget, utilization
    );
    eprintln!(
        "  - Files: {}/{} bundled",
        selected.len(),
        manifest.total_files
    );

    Ok(())
}

fn exclude_output_dir(
    root: &Path,
    out_dir: &Path,
    scan_args: &mut cli::GlobalArgs,
) -> Vec<HandoffExcludedPath> {
    let pattern = normalize_exclude_pattern(root, out_dir);
    if !pattern.is_empty() {
        let _ = add_exclude_pattern(&mut scan_args.excluded, pattern.clone());
    }
    vec![HandoffExcludedPath {
        path: pattern,
        reason: "output_dir".to_string(),
    }]
}

fn discover_packet_local_proof_route(
    explicit_proof_route: Option<&Path>,
    review_packet_dir: Option<&Path>,
) -> Option<PathBuf> {
    if explicit_proof_route.is_some() {
        return None;
    }

    let proof_route = review_packet_dir?
        .join("proof")
        .join("proof-pack-route.json");
    proof_route.is_file().then_some(proof_route)
}

/// Round a float to N decimal places.
fn round_f64(value: f64, decimals: u32) -> f64 {
    let factor = 10_f64.powi(decimals as i32);
    (value * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_scan::normalize_slashes as normalize_path;
    use tokmd_types::{ExportData, FileRow};

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("foo/bar"), "foo/bar");
        assert_eq!(normalize_path("foo\\bar"), "foo/bar");
        assert_eq!(normalize_path("foo\\bar\\baz"), "foo/bar/baz");
    }

    #[test]
    fn test_round_f64() {
        assert_eq!(round_f64(1.234567, 2), 1.23);
        assert_eq!(round_f64(1.234567, 4), 1.2346);
        assert_eq!(round_f64(100.0, 2), 100.0);
    }

    #[test]
    fn test_build_tree_empty() {
        let export = ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 2,
            children: tokmd_types::ChildIncludeMode::ParentsOnly,
        };
        let tree = tokmd_format::render_handoff_tree(&export, DEFAULT_TREE_DEPTH);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_build_tree_depth_limit_and_no_file_leaves() {
        let export = ExportData {
            rows: vec![FileRow {
                path: "a/b/c/file.rs".to_string(),
                module: "a".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 10,
                comments: 0,
                blanks: 0,
                lines: 10,
                bytes: 100,
                tokens: 20,
            }],
            module_roots: vec![],
            module_depth: 2,
            children: tokmd_types::ChildIncludeMode::ParentsOnly,
        };
        let tree = tokmd_format::render_handoff_tree(&export, 1);
        assert!(tree.contains("a/"));
        assert!(!tree.contains("b/"));
        assert!(!tree.contains("file.rs"));
    }
}
