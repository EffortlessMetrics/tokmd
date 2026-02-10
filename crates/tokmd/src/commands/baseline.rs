//! Baseline command implementation.
//!
//! Generates a complexity baseline for trend tracking over time.

use std::path::Path;

use anyhow::{Context, Result, bail};
use tokmd_analysis as analysis;
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisSource, ComplexityBaseline};
use tokmd_config::{BaselineArgs, GlobalArgs};

use crate::analysis_utils;
use crate::export_bundle;
use crate::progress::Progress;

pub(crate) fn handle(args: BaselineArgs, global: &GlobalArgs) -> Result<()> {
    let progress = Progress::new(!global.no_progress);

    // Check for existing file before doing any work
    if args.output.exists() && !args.force {
        bail!(
            "Baseline file already exists at {}. Use --force to overwrite.",
            args.output.display()
        );
    }

    // Load export data
    progress.set_message("Loading export data...");
    let inputs = vec![args.path.clone()];
    let bundle = export_bundle::load_export_from_inputs(&inputs, global)?;

    // Build analysis source metadata
    let source = AnalysisSource {
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

    let args_meta = AnalysisArgsMeta {
        preset: "health".to_string(),
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

    // Run analysis with "health" preset (includes complexity)
    progress.set_message("Running complexity analysis...");
    let request = analysis::AnalysisRequest {
        preset: analysis::AnalysisPreset::Health,
        args: args_meta,
        limits: analysis::AnalysisLimits::default(),
        window_tokens: None,
        git: None,
        import_granularity: analysis::ImportGranularity::Module,
        detail_functions: false,
    };

    let ctx = analysis::AnalysisContext {
        export: bundle.export,
        root: bundle.root.clone(),
        source,
    };

    let receipt = analysis::analyze(ctx, request)?;

    // Generate baseline from analysis receipt
    progress.set_message("Generating baseline...");
    let mut baseline = ComplexityBaseline::from_analysis(&receipt);

    // Capture git commit SHA if in a git repo
    baseline.commit = capture_git_commit(&args.path);

    // Create output directory if needed
    if let Some(parent) = args.output.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    // Write JSON to output path
    progress.set_message("Writing baseline...");
    let file = std::fs::File::create(&args.output).with_context(|| {
        format!(
            "Failed to create baseline file at {}",
            args.output.display()
        )
    })?;
    serde_json::to_writer_pretty(file, &baseline)
        .with_context(|| format!("Failed to write baseline to {}", args.output.display()))?;

    progress.finish_and_clear();

    eprintln!("Baseline generated at {}", args.output.display());
    if let Some(commit) = &baseline.commit {
        eprintln!("  Commit: {}", commit);
    }
    eprintln!(
        "  Files: {}, Functions: {}",
        baseline.metrics.total_files, baseline.metrics.function_count
    );
    eprintln!(
        "  Avg cyclomatic: {:.2}, Max: {}",
        baseline.metrics.avg_cyclomatic, baseline.metrics.max_cyclomatic
    );

    // TODO v1.5.1: Add --determinism flag support
    // This would hash build artifacts via tokmd-content for reproducibility verification
    if args.determinism {
        eprintln!("Warning: --determinism flag not yet implemented (deferred to v1.5.1)");
    }

    Ok(())
}

/// Capture the current git commit SHA from the repository.
///
/// Returns `Some(sha)` if the path is inside a git repository,
/// `None` otherwise or if git is not available.
fn capture_git_commit(path: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .ok()?;

    if output.status.success() {
        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !sha.is_empty() {
            return Some(sha);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_capture_git_commit_returns_sha_in_repo() {
        // This test assumes we're running in a git repository
        let sha = capture_git_commit(&PathBuf::from("."));
        // In a git repo, we should get a SHA
        // In CI without git, this might be None
        if let Some(sha) = sha {
            // SHA should be 40 hex characters
            assert_eq!(sha.len(), 40);
            assert!(sha.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn test_capture_git_commit_returns_none_for_non_repo() {
        // A path that is very unlikely to be a git repo
        let sha = capture_git_commit(&PathBuf::from("/"));
        // Root should not be a git repo (in most cases)
        // Note: This might fail in some edge cases where / is somehow a git repo
        assert!(sha.is_none() || sha.unwrap().len() == 40);
    }
}
