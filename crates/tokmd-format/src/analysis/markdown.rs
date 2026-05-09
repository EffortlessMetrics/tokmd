//! Markdown renderer for tokmd analysis receipts.
//!
//! This module owns all Markdown formatting logic behind the `tokmd-format`
//! analysis facade.
//!
//! ## Effort rendering
//!
//! Effort sections are rendered in two tiers:
//!
//! 1. `receipt.effort` — preferred path for the newer effort-estimation
//!    receipt surface. Renders size basis, confidence, drivers,
//!    assumptions, and optional delta data.
//! 2. `derived.cocomo` — legacy fallback used when the richer `effort`
//!    section is absent but classic derived COCOMO data is present.
//!
//! The formatter intentionally renders whatever the receipt contains without
//! inferring missing estimate data.

use std::fmt::Write;
use tokmd_analysis_types::AnalysisReceipt;

mod api_surface;
mod archetype;
mod assets;
mod complexity;
mod corporate_fingerprint;
mod dependencies;
mod derived;
mod duplicates;
mod eco_label;
mod effort;
mod entropy;
mod git;
mod imports;
mod inputs;
mod license;
mod predictive_churn;
mod topics;

/// Render an [`AnalysisReceipt`] to a Markdown string.
///
/// This is the sole public entry point. All subsections (derived metrics,
/// effort, duplicates, complexity, etc.) are rendered internally.
pub fn render_md(receipt: &AnalysisReceipt) -> String {
    let mut out = String::new();
    out.push_str("# tokmd analysis\n\n");
    let _ = writeln!(out, "Preset: `{}`\n", receipt.args.preset);

    if !receipt.source.inputs.is_empty() {
        inputs::render_inputs(&mut out, &receipt.source.inputs);
    }

    if let Some(archetype) = &receipt.archetype {
        archetype::render_archetype(&mut out, archetype);
    }

    if let Some(topics) = &receipt.topics {
        topics::render_topic_clouds(&mut out, topics);
    }

    if let Some(entropy) = &receipt.entropy {
        entropy::render_entropy_report(&mut out, entropy);
    }

    if let Some(license) = &receipt.license {
        license::render_license_report(&mut out, license);
    }

    if let Some(fingerprint) = &receipt.corporate_fingerprint {
        corporate_fingerprint::render_corporate_fingerprint(&mut out, fingerprint);
    }

    if let Some(churn) = &receipt.predictive_churn {
        predictive_churn::render_predictive_churn(&mut out, churn);
    }

    if let Some(derived) = &receipt.derived {
        derived::render_derived_report(&mut out, derived, receipt.effort.as_ref());
    }

    if let Some(assets) = &receipt.assets {
        assets::render_asset_report(&mut out, assets);
    }

    if let Some(deps) = &receipt.deps {
        dependencies::render_dependency_report(&mut out, deps);
    }

    if let Some(git) = &receipt.git {
        git::render_git_report(&mut out, git);
    }

    if let Some(imports) = &receipt.imports {
        imports::render_import_report(&mut out, imports);
    }

    if let Some(dup) = &receipt.dup {
        duplicates::render_duplicate_report(&mut out, dup);
    }

    if let Some(cx) = &receipt.complexity {
        complexity::render_complexity_report(&mut out, cx);
    }

    if let Some(api) = &receipt.api_surface {
        api_surface::render_api_surface_report(&mut out, api);
    }

    if let Some(fun) = &receipt.fun
        && let Some(label) = &fun.eco_label
    {
        eco_label::render_eco_label(&mut out, label);
    }

    out
}

fn fmt_pct(ratio: f64) -> String {
    format!("{:.1}%", ratio * 100.0)
}

fn fmt_f64(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_analysis_types::*;

    fn minimal_receipt() -> AnalysisReceipt {
        AnalysisReceipt {
            schema_version: 2,
            generated_at_ms: 0,
            tool: tokmd_types::ToolInfo {
                name: "tokmd".to_string(),
                version: "0.0.0".to_string(),
            },
            mode: "analysis".to_string(),
            status: tokmd_types::ScanStatus::Complete,
            warnings: vec![],
            source: AnalysisSource {
                inputs: vec!["test".to_string()],
                export_path: None,
                base_receipt_path: None,
                export_schema_version: None,
                export_generated_at_ms: None,
                base_signature: None,
                module_roots: vec![],
                module_depth: 1,
                children: "collapse".to_string(),
            },
            args: AnalysisArgsMeta {
                preset: "receipt".to_string(),
                format: "md".to_string(),
                window_tokens: None,
                git: None,
                max_files: None,
                max_bytes: None,
                max_commits: None,
                max_commit_files: None,
                max_file_bytes: None,
                import_granularity: "module".to_string(),
            },
            archetype: None,
            topics: None,
            entropy: None,
            predictive_churn: None,
            corporate_fingerprint: None,
            license: None,
            derived: None,
            assets: None,
            deps: None,
            git: None,
            imports: None,
            dup: None,
            complexity: None,
            api_surface: None,
            fun: None,
            effort: None,
        }
    }

    #[test]
    fn minimal_receipt_renders_without_panic() {
        let receipt = minimal_receipt();
        let md = render_md(&receipt);
        assert!(md.starts_with("# tokmd analysis\n"));
        assert!(md.contains("Preset: `receipt`"));
        assert!(md.contains("## Inputs\n"));
    }

    #[test]
    fn fmt_pct_output_format() {
        assert_eq!(fmt_pct(0.456), "45.6%");
        assert_eq!(fmt_pct(0.0), "0.0%");
        assert_eq!(fmt_pct(1.0), "100.0%");
    }

    #[test]
    fn fmt_f64_output_format() {
        assert_eq!(fmt_f64(std::f64::consts::PI, 2), "3.14");
        assert_eq!(fmt_f64(1.0, 4), "1.0000");
    }
}
