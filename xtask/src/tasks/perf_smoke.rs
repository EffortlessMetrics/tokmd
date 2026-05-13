use crate::cli::PerfSmokeArgs;
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::time::{SystemTime, UNIX_EPOCH};
use tokmd_core::settings::{ExportSettings, LangSettings, ModuleSettings, ScanSettings};
use tokmd_core::{
    WorkflowTiming, timed_export_workflow, timed_lang_workflow, timed_module_workflow,
};

const PERF_SMOKE_SCHEMA: &str = "tokmd.perf_smoke.v1";

#[derive(Debug, Serialize)]
struct PerfSmokeReceipt {
    schema: String,
    schema_version: u32,
    generated_at_ms: u128,
    repo: String,
    sha: String,
    target: PerfSmokeTarget,
    workflows: Vec<WorkflowTiming>,
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
        "perf smoke receipt written to {} ({} workflow(s))",
        args.output.display(),
        receipt.workflows.len()
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

    Ok(PerfSmokeReceipt {
        schema: PERF_SMOKE_SCHEMA.to_string(),
        schema_version: 1,
        generated_at_ms: now_ms(),
        repo: args.repo.clone(),
        sha: receipt_sha(args),
        target: PerfSmokeTarget {
            path_count: 1,
            paths_redacted: true,
        },
        status: PerfSmokeStatus {
            ok: true,
            workflow_count: workflows.len(),
        },
        workflows,
    })
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
        assert_eq!(receipt.schema_version, 1);
        assert_eq!(receipt.sha, "abc123");
        assert_eq!(receipt.target.path_count, 1);
        assert!(receipt.target.paths_redacted);
        assert!(receipt.status.ok);
        assert_eq!(receipt.status.workflow_count, 3);
        assert_eq!(receipt.workflows.len(), 3);
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
        assert_eq!(value["status"]["workflow_count"], 3);
        Ok(())
    }
}
