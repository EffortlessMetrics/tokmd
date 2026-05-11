//! Receipt loading and compute-then-gate preparation for `tokmd gate`.

use std::path::Path;

use anyhow::{Context, Result};
use tokmd_analysis as analysis;
use tokmd_analysis_types as analysis_types;

use crate::analysis_utils;
use crate::cli;
use crate::export_bundle;

/// Load a receipt JSON file or compute an analysis receipt from an input path.
pub(super) fn load_or_compute_receipt(
    args: &cli::CliGateArgs,
    global: &cli::GlobalArgs,
) -> Result<serde_json::Value> {
    let input = args.input.clone().unwrap_or_else(|| ".".into());

    if input.extension().map(|e| e == "json").unwrap_or(false) && input.exists() {
        let content = std::fs::read_to_string(&input)
            .with_context(|| format!("Failed to read receipt from {}", input.display()))?;
        return serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON from {}", input.display()));
    }

    let preset = args.preset.unwrap_or_else(|| {
        if args.baseline.is_some() {
            cli::AnalysisPreset::Health
        } else {
            cli::AnalysisPreset::Receipt
        }
    });
    compute_receipt(&input, preset, global)
}

fn compute_receipt(
    input: &Path,
    preset: cli::AnalysisPreset,
    global: &cli::GlobalArgs,
) -> Result<serde_json::Value> {
    let inputs = vec![input.to_path_buf()];
    let bundle = export_bundle::load_export_from_inputs(&inputs, global)?;

    let source = analysis_types::AnalysisSource {
        inputs: inputs.iter().map(|p| p.display().to_string()).collect(),
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
        format: "json".to_string(),
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
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.80,
        near_dup_max_files: 2000,
        near_dup_scope: analysis::NearDupScope::Module,
        near_dup_max_pairs: None,
        near_dup_exclude: Vec::new(),
        effort: None,
    };

    let ctx = analysis::AnalysisContext {
        export: bundle.export,
        root: bundle.root,
        source,
    };

    let receipt = analysis::analyze(ctx, request)?;

    serde_json::to_value(receipt).context("Failed to serialize receipt to JSON")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gate_args_with_input(input: std::path::PathBuf) -> cli::CliGateArgs {
        cli::CliGateArgs {
            input: Some(input),
            policy: None,
            baseline: None,
            ratchet_config: None,
            preset: None,
            format: cli::GateFormat::Json,
            fail_fast: false,
        }
    }

    #[test]
    fn load_or_compute_receipt_reads_json_input() {
        let temp = tempfile::tempdir().unwrap();
        let receipt_path = temp.path().join("receipt.json");
        std::fs::write(&receipt_path, r#"{"schema_version": 2, "ok": true}"#).unwrap();

        let loaded = load_or_compute_receipt(
            &gate_args_with_input(receipt_path),
            &cli::GlobalArgs::default(),
        )
        .unwrap();

        assert_eq!(loaded["schema_version"], 2);
        assert_eq!(loaded["ok"], true);
    }

    #[test]
    fn load_or_compute_receipt_reports_invalid_json_input() {
        let temp = tempfile::tempdir().unwrap();
        let receipt_path = temp.path().join("receipt.json");
        std::fs::write(&receipt_path, "{not-json").unwrap();

        let err = load_or_compute_receipt(
            &gate_args_with_input(receipt_path),
            &cli::GlobalArgs::default(),
        )
        .unwrap_err();

        assert!(err.to_string().contains("Failed to parse JSON"));
    }
}
