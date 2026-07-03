use crate::cli::PerfSmokeArgs;
use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Instant;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::time::{SystemTime, UNIX_EPOCH};
use tokmd_analysis::io_trace::{self, IoTraceReport};
use tokmd_core::settings::{
    AnalyzeSettings, ExportSettings, LangSettings, ModuleSettings, ScanSettings,
};
use tokmd_core::{
    WorkflowTiming, analyze_workflow, timed_export_workflow, timed_lang_workflow,
    timed_module_workflow,
};

const PERF_SMOKE_SCHEMA: &str = "tokmd.perf_smoke.v1";
const ANALYSIS_TIMING_SCHEMA: &str = "tokmd.analysis_workflow_timing.v1";
const IO_TRACE_SCHEMA: &str = "tokmd.io_open_trace.v1";

#[derive(Debug, Serialize)]
struct PerfSmokeReceipt {
    schema: String,
    schema_version: u32,
    generated_at_ms: u128,
    repo: String,
    sha: String,
    target: PerfSmokeTarget,
    workflows: Vec<WorkflowTiming>,
    analysis_workflows: Vec<AnalysisWorkflowTiming>,
    status: PerfSmokeStatus,
}

#[derive(Debug, Serialize)]
struct PerfSmokeTarget {
    path_count: usize,
    paths_redacted: bool,
}

#[derive(Debug, Serialize)]
struct PerfSmokeStatus {
    ok: bool,
    workflow_count: usize,
    core_workflow_count: usize,
    analysis_workflow_count: usize,
}

#[derive(Debug, Serialize)]
struct AnalysisWorkflowTiming {
    schema: String,
    schema_version: u32,
    workflow: String,
    preset: String,
    path_count: usize,
    language_count: usize,
    row_count: usize,
    warning_count: usize,
    enabled_reports: Vec<String>,
    limits: AnalysisTimingLimits,
    total_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    io_trace: Option<IoTraceSection>,
}

#[derive(Debug, Serialize)]
struct IoTraceSection {
    schema: String,
    schema_version: u32,
    total_opens: u64,
    unique_paths: u64,
    unique_keys: u64,
    duplicate_key_opens: u64,
    max_opens_for_key: u64,
    opens_per_path: f64,
    by_mode: BTreeMap<String, IoTraceModeSection>,
}

#[derive(Debug, Serialize)]
struct IoTraceModeSection {
    opens: u64,
    unique_keys: u64,
}

impl From<&IoTraceReport> for IoTraceSection {
    fn from(report: &IoTraceReport) -> Self {
        let by_mode = report
            .by_mode
            .iter()
            .map(|(mode, stats)| {
                (
                    (*mode).to_string(),
                    IoTraceModeSection {
                        opens: stats.opens,
                        unique_keys: stats.unique_keys,
                    },
                )
            })
            .collect();
        Self {
            schema: IO_TRACE_SCHEMA.to_string(),
            schema_version: 1,
            total_opens: report.total_opens,
            unique_paths: report.unique_paths,
            unique_keys: report.unique_keys,
            duplicate_key_opens: report.duplicate_key_opens,
            max_opens_for_key: report.max_opens_for_key,
            opens_per_path: report.opens_per_path(),
            by_mode,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
struct AnalysisTimingLimits {
    max_files: usize,
    max_bytes: u64,
    max_file_bytes: u64,
    max_commits: usize,
    max_commit_files: usize,
}

pub fn run(args: PerfSmokeArgs) -> Result<()> {
    let receipt = perf_smoke_receipt(&args)?;

    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let json = serde_json::to_string_pretty(&receipt).context("serialize perf smoke receipt")?;
    fs::write(&args.output, format!("{json}\n"))
        .with_context(|| format!("write {}", args.output.display()))?;

    println!(
        "perf smoke receipt written to {} ({} workflow(s), {} analysis workflow(s))",
        args.output.display(),
        receipt.workflows.len(),
        receipt.analysis_workflows.len()
    );
    Ok(())
}

fn perf_smoke_receipt(args: &PerfSmokeArgs) -> Result<PerfSmokeReceipt> {
    let scan = ScanSettings::for_paths(vec![path_arg(&args.target_repo)]);
    let lang = timed_lang_workflow(&scan, &LangSettings::default())
        .with_context(|| format!("run lang timing for {}", args.target_repo.display()))?;
    let module = timed_module_workflow(&scan, &ModuleSettings::default())
        .with_context(|| format!("run module timing for {}", args.target_repo.display()))?;
    let export = timed_export_workflow(&scan, &ExportSettings::default())
        .with_context(|| format!("run export timing for {}", args.target_repo.display()))?;

    let workflows = vec![lang.timing, module.timing, export.timing];
    let analysis_workflows = analysis_timings(args, &scan)?;
    let workflow_count = workflows.len() + analysis_workflows.len();

    Ok(PerfSmokeReceipt {
        schema: PERF_SMOKE_SCHEMA.to_string(),
        schema_version: 2,
        generated_at_ms: now_ms(),
        repo: args.repo.clone(),
        sha: receipt_sha(args),
        target: PerfSmokeTarget {
            path_count: 1,
            paths_redacted: true,
        },
        status: PerfSmokeStatus {
            ok: true,
            workflow_count,
            core_workflow_count: workflows.len(),
            analysis_workflow_count: analysis_workflows.len(),
        },
        analysis_workflows,
        workflows,
    })
}

fn analysis_timings(
    args: &PerfSmokeArgs,
    scan: &ScanSettings,
) -> Result<Vec<AnalysisWorkflowTiming>> {
    let limits = AnalysisTimingLimits {
        max_files: args.analysis_max_files,
        max_bytes: args.analysis_max_bytes,
        max_file_bytes: args.analysis_max_file_bytes,
        max_commits: args.analysis_max_commits,
        max_commit_files: args.analysis_max_commit_files,
    };

    args.analysis_presets
        .iter()
        .map(|preset| analysis_timing(scan, preset, &limits, args.trace_io))
        .collect()
}

fn analysis_timing(
    scan: &ScanSettings,
    preset: &str,
    limits: &AnalysisTimingLimits,
    trace_io: bool,
) -> Result<AnalysisWorkflowTiming> {
    let normalized = preset.trim().to_ascii_lowercase();
    let analyze = AnalyzeSettings {
        preset: normalized.clone(),
        max_files: Some(limits.max_files),
        max_bytes: Some(limits.max_bytes),
        max_file_bytes: Some(limits.max_file_bytes),
        max_commits: Some(limits.max_commits),
        max_commit_files: Some(limits.max_commit_files),
        ..AnalyzeSettings::default()
    };

    let start = Instant::now();
    let (receipt_result, io_report) = if trace_io {
        let (result, report) = io_trace::scope(|| analyze_workflow(scan, &analyze));
        (result, Some(report))
    } else {
        (analyze_workflow(scan, &analyze), None)
    };
    let total_ms = start.elapsed().as_millis();
    let receipt =
        receipt_result.with_context(|| format!("run analyze timing for preset `{normalized}`"))?;

    let derived = receipt.derived.as_ref();
    let row_count = derived
        .map(|report| report.totals.files)
        .unwrap_or_default();
    let language_count = derived
        .map(|report| report.polyglot.lang_count)
        .unwrap_or_default();

    Ok(AnalysisWorkflowTiming {
        schema: ANALYSIS_TIMING_SCHEMA.to_string(),
        schema_version: 1,
        workflow: "analyze".to_string(),
        preset: normalized,
        path_count: scan.paths.len().max(1),
        language_count,
        row_count,
        warning_count: receipt.warnings.len(),
        enabled_reports: enabled_analysis_reports(&receipt),
        limits: limits.clone(),
        total_ms,
        io_trace: io_report.as_ref().map(IoTraceSection::from),
    })
}

fn enabled_analysis_reports(receipt: &tokmd_analysis_types::AnalysisReceipt) -> Vec<String> {
    let mut reports = Vec::new();
    push_report(&mut reports, "archetype", receipt.archetype.is_some());
    push_report(&mut reports, "topics", receipt.topics.is_some());
    push_report(&mut reports, "entropy", receipt.entropy.is_some());
    push_report(
        &mut reports,
        "predictive_churn",
        receipt.predictive_churn.is_some(),
    );
    push_report(
        &mut reports,
        "corporate_fingerprint",
        receipt.corporate_fingerprint.is_some(),
    );
    push_report(&mut reports, "license", receipt.license.is_some());
    push_report(&mut reports, "derived", receipt.derived.is_some());
    push_report(&mut reports, "assets", receipt.assets.is_some());
    push_report(&mut reports, "deps", receipt.deps.is_some());
    push_report(&mut reports, "git", receipt.git.is_some());
    push_report(&mut reports, "imports", receipt.imports.is_some());
    push_report(&mut reports, "dup", receipt.dup.is_some());
    push_report(&mut reports, "complexity", receipt.complexity.is_some());
    push_report(&mut reports, "api_surface", receipt.api_surface.is_some());
    push_report(&mut reports, "effort", receipt.effort.is_some());
    push_report(&mut reports, "fun", receipt.fun.is_some());
    reports
}

fn push_report(reports: &mut Vec<String>, name: &str, present: bool) {
    if present {
        reports.push(name.to_string());
    }
}

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn receipt_sha(args: &PerfSmokeArgs) -> String {
    args.sha
        .clone()
        .or_else(|| env_non_empty("GITHUB_SHA"))
        .unwrap_or_else(|| "HEAD".to_string())
}

fn env_non_empty(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn now_ms() -> u128 {
    1
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;

    use super::*;

    #[test]
    fn receipt_records_phase_timings_without_raw_paths() -> Result<()> {
        let temp = tempfile::tempdir()?;
        fs::write(temp.path().join("main.rs"), "fn main() {}\n")?;
        let args = PerfSmokeArgs {
            target_repo: temp.path().to_path_buf(),
            output: temp.path().join("perf.json"),
            sha: Some("abc123".to_string()),
            ..PerfSmokeArgs::default()
        };

        let receipt = perf_smoke_receipt(&args)?;

        assert_eq!(receipt.schema, PERF_SMOKE_SCHEMA);
        assert_eq!(receipt.schema_version, 2);
        assert_eq!(receipt.sha, "abc123");
        assert_eq!(receipt.target.path_count, 1);
        assert!(receipt.target.paths_redacted);
        assert!(receipt.status.ok);
        assert_eq!(receipt.status.workflow_count, 3);
        assert_eq!(receipt.status.core_workflow_count, 3);
        assert_eq!(receipt.status.analysis_workflow_count, 0);
        assert_eq!(receipt.workflows.len(), 3);
        assert!(receipt.analysis_workflows.is_empty());
        assert_eq!(receipt.workflows[0].workflow, "lang");
        assert_eq!(receipt.workflows[1].workflow, "module");
        assert_eq!(receipt.workflows[2].workflow, "export");
        assert!(!serde_json::to_string(&receipt)?.contains(temp.path().to_string_lossy().as_ref()));
        Ok(())
    }

    #[test]
    fn run_writes_pretty_json_receipt() -> Result<()> {
        let temp = tempfile::tempdir()?;
        fs::write(temp.path().join("lib.rs"), "pub fn lib() {}\n")?;
        let output = temp.path().join("out").join("perf-smoke.json");
        let args = PerfSmokeArgs {
            target_repo: temp.path().to_path_buf(),
            output: output.clone(),
            ..PerfSmokeArgs::default()
        };

        run(args)?;

        let value: serde_json::Value = serde_json::from_str(&fs::read_to_string(output)?)?;
        assert_eq!(value["schema"], PERF_SMOKE_SCHEMA);
        assert_eq!(value["schema_version"], 2);
        assert_eq!(value["status"]["workflow_count"], 3);
        Ok(())
    }

    #[test]
    fn receipt_can_include_bounded_analysis_timings_without_raw_paths() -> Result<()> {
        let temp = tempfile::tempdir()?;
        fs::write(
            temp.path().join("lib.rs"),
            "pub fn lib() { /* TODO: keep test content visible */ }\n",
        )?;
        let args = PerfSmokeArgs {
            target_repo: temp.path().to_path_buf(),
            output: temp.path().join("perf.json"),
            analysis_presets: vec!["health".to_string()],
            analysis_max_files: 42,
            analysis_max_bytes: 1024,
            analysis_max_file_bytes: 512,
            analysis_max_commits: 7,
            analysis_max_commit_files: 8,
            ..PerfSmokeArgs::default()
        };

        let receipt = perf_smoke_receipt(&args)?;

        assert_eq!(receipt.status.workflow_count, 4);
        assert_eq!(receipt.status.analysis_workflow_count, 1);
        assert_eq!(receipt.analysis_workflows.len(), 1);
        let timing = &receipt.analysis_workflows[0];
        assert_eq!(timing.schema, ANALYSIS_TIMING_SCHEMA);
        assert_eq!(timing.workflow, "analyze");
        assert_eq!(timing.preset, "health");
        assert_eq!(timing.path_count, 1);
        assert!(timing.row_count >= 1);
        assert!(timing.enabled_reports.contains(&"derived".to_string()));
        assert_eq!(timing.limits.max_files, 42);
        assert_eq!(timing.limits.max_bytes, 1024);
        assert_eq!(timing.limits.max_file_bytes, 512);
        assert_eq!(timing.limits.max_commits, 7);
        assert_eq!(timing.limits.max_commit_files, 8);
        assert!(timing.io_trace.is_none());
        assert!(!serde_json::to_string(&receipt)?.contains(temp.path().to_string_lossy().as_ref()));
        Ok(())
    }

    #[test]
    fn trace_io_records_content_open_section() -> Result<()> {
        let temp = tempfile::tempdir()?;
        fs::write(
            temp.path().join("lib.rs"),
            "pub fn lib() { /* TODO: keep content enrichers reading this file */ }\n",
        )?;
        let args = PerfSmokeArgs {
            target_repo: temp.path().to_path_buf(),
            output: temp.path().join("perf.json"),
            analysis_presets: vec!["health".to_string()],
            trace_io: true,
            ..PerfSmokeArgs::default()
        };

        let receipt = perf_smoke_receipt(&args)?;

        let timing = &receipt.analysis_workflows[0];
        let trace = timing
            .io_trace
            .as_ref()
            .expect("trace_io should populate an io_trace section");
        assert_eq!(trace.schema, IO_TRACE_SCHEMA);
        assert!(trace.total_opens >= 1, "expected at least one content open");
        assert!(trace.unique_paths >= 1);
        assert!(trace.total_opens >= trace.unique_keys);
        assert_eq!(
            trace.duplicate_key_opens,
            trace.total_opens - trace.unique_keys
        );
        assert!(!trace.by_mode.is_empty());
        assert!(!serde_json::to_string(&receipt)?.contains(temp.path().to_string_lossy().as_ref()));
        Ok(())
    }
}
