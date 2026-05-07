use crate::cli::CiActualsArgs;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const CI_ACTUALS_SCHEMA: &str = "tokmd.ci_actuals.v1";

#[derive(Debug, Deserialize)]
struct NeedEntry {
    #[serde(default)]
    result: Option<String>,
    #[serde(default)]
    outputs: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TimingInput {
    Seconds(f64),
    Object(TimingObject),
}

#[derive(Debug, Deserialize)]
struct TimingObject {
    duration_seconds: Option<f64>,
    seconds: Option<f64>,
    runner: Option<String>,
    cache_hit: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
struct TimingRecord {
    duration_seconds: Option<f64>,
    runner: Option<String>,
    cache_hit: Option<bool>,
}

#[derive(Debug, Serialize)]
struct CiActualsReceipt {
    schema: String,
    schema_version: u32,
    repo: String,
    workflow: String,
    sha: String,
    github: GithubContext,
    jobs: Vec<CiJobActual>,
    status: CiActualsStatus,
}

#[derive(Debug, Serialize)]
struct GithubContext {
    run_id: Option<String>,
    run_attempt: Option<String>,
    event_name: Option<String>,
    ref_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct CiJobActual {
    name: String,
    result: String,
    output_keys: Vec<String>,
    runner: Option<String>,
    duration_seconds: Option<f64>,
    duration_minutes: Option<f64>,
    timing_status: String,
    cache_hit: Option<bool>,
}

#[derive(Debug, Serialize)]
struct CiActualsStatus {
    ok: bool,
    job_count: usize,
    timed_job_count: usize,
    missing_timing: Vec<String>,
    unused_timing: Vec<String>,
}

pub fn run(args: CiActualsArgs) -> Result<()> {
    let root = workspace_root()?;
    let receipt = ci_actuals_receipt(&root, &args)?;

    if let Some(parent) = args.output.parent() {
        let output_parent = resolve_path(&root, parent);
        fs::create_dir_all(&output_parent)
            .with_context(|| format!("create {}", output_parent.display()))?;
    }

    let output = resolve_path(&root, &args.output);
    let json = serde_json::to_string_pretty(&receipt).context("serialize CI actuals receipt")?;
    fs::write(&output, format!("{json}\n"))
        .with_context(|| format!("write {}", output.display()))?;
    println!(
        "CI actuals receipt written to {} ({} job(s), {} timed)",
        output.display(),
        receipt.status.job_count,
        receipt.status.timed_job_count
    );
    Ok(())
}

fn ci_actuals_receipt(root: &Path, args: &CiActualsArgs) -> Result<CiActualsReceipt> {
    let needs_path = resolve_path(root, &args.needs);
    let needs: BTreeMap<String, NeedEntry> = read_json(&needs_path)?;
    let timings = match &args.timings {
        Some(path) => read_timings(&resolve_path(root, path))?,
        None => BTreeMap::new(),
    };

    let mut used_timings = BTreeSet::new();
    let mut missing_timing = Vec::new();
    let mut timed_job_count = 0;
    let mut jobs = Vec::new();

    for (name, need) in needs {
        let timing = timings.get(&name);
        if timing.is_some() {
            used_timings.insert(name.clone());
        }

        let duration_seconds = timing.and_then(|record| record.duration_seconds);
        let duration_minutes = duration_seconds.map(|seconds| seconds / 60.0);
        if duration_seconds.is_none() {
            missing_timing.push(name.clone());
        } else {
            timed_job_count += 1;
        }

        let output_keys = need.outputs.keys().cloned().collect::<Vec<_>>();
        jobs.push(CiJobActual {
            name,
            result: need.result.unwrap_or_else(|| "unknown".to_string()),
            output_keys,
            runner: timing.and_then(|record| record.runner.clone()),
            duration_seconds,
            duration_minutes,
            timing_status: if duration_seconds.is_some() {
                "measured".to_string()
            } else {
                "missing".to_string()
            },
            cache_hit: timing.and_then(|record| record.cache_hit),
        });
    }

    jobs.sort_by(|left, right| left.name.cmp(&right.name));
    missing_timing.sort();

    let mut unused_timing = timings
        .keys()
        .filter(|name| !used_timings.contains(*name))
        .cloned()
        .collect::<Vec<_>>();
    unused_timing.sort();
    let job_count = jobs.len();

    Ok(CiActualsReceipt {
        schema: CI_ACTUALS_SCHEMA.to_string(),
        schema_version: 1,
        repo: args.repo.clone(),
        workflow: args.workflow.clone(),
        sha: receipt_sha(args),
        github: GithubContext {
            run_id: env_non_empty("GITHUB_RUN_ID"),
            run_attempt: env_non_empty("GITHUB_RUN_ATTEMPT"),
            event_name: env_non_empty("GITHUB_EVENT_NAME"),
            ref_name: env_non_empty("GITHUB_REF_NAME").or_else(|| env_non_empty("GITHUB_REF")),
        },
        jobs,
        status: CiActualsStatus {
            ok: true,
            job_count,
            timed_job_count,
            missing_timing,
            unused_timing,
        },
    })
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&body).with_context(|| format!("parse json {}", path.display()))
}

fn read_timings(path: &Path) -> Result<BTreeMap<String, TimingRecord>> {
    let inputs: BTreeMap<String, TimingInput> = read_json(path)?;
    inputs
        .into_iter()
        .map(|(name, input)| {
            let record = timing_record(input)
                .with_context(|| format!("invalid timing entry `{name}` in {}", path.display()))?;
            Ok((name, record))
        })
        .collect()
}

fn timing_record(input: TimingInput) -> Result<TimingRecord> {
    let record = match input {
        TimingInput::Seconds(seconds) => TimingRecord {
            duration_seconds: Some(seconds),
            runner: None,
            cache_hit: None,
        },
        TimingInput::Object(object) => TimingRecord {
            duration_seconds: object.duration_seconds.or(object.seconds),
            runner: object.runner,
            cache_hit: object.cache_hit,
        },
    };

    if let Some(seconds) = record.duration_seconds
        && (!seconds.is_finite() || seconds < 0.0)
    {
        bail!("duration_seconds must be a finite non-negative number");
    }

    Ok(record)
}

fn receipt_sha(args: &CiActualsArgs) -> String {
    args.sha
        .clone()
        .or_else(|| env_non_empty("GITHUB_SHA"))
        .unwrap_or_else(|| "HEAD".to_string())
}

fn env_non_empty(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

fn workspace_root() -> Result<PathBuf> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .exec()
        .context("locate workspace root")?;
    Ok(metadata.workspace_root.into_std_path_buf())
}

fn resolve_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_json(path: &Path, value: &serde_json::Value) {
        let body = serde_json::to_string_pretty(value).expect("serialize json fixture");
        fs::write(path, body).expect("write json fixture");
    }

    #[test]
    fn receipt_preserves_missing_timing_as_missing() {
        let temp = tempfile::tempdir().expect("tempdir");
        let needs = temp.path().join("needs.json");
        write_json(
            &needs,
            &serde_json::json!({
                "docs-check": {"result": "success", "outputs": {"docs": "ok"}},
                "mutation": {"result": "skipped", "outputs": {}}
            }),
        );

        let args = CiActualsArgs {
            needs,
            output: temp.path().join("out.json"),
            sha: Some("abc123".to_string()),
            ..CiActualsArgs::default()
        };

        let receipt = ci_actuals_receipt(Path::new("."), &args).expect("receipt");
        assert_eq!(receipt.schema, CI_ACTUALS_SCHEMA);
        assert_eq!(receipt.sha, "abc123");
        assert_eq!(receipt.status.job_count, 2);
        assert_eq!(receipt.status.timed_job_count, 0);
        assert_eq!(receipt.status.missing_timing, ["docs-check", "mutation"]);
        assert!(
            receipt
                .jobs
                .iter()
                .all(|job| job.duration_seconds.is_none())
        );
        assert!(
            receipt
                .jobs
                .iter()
                .all(|job| job.timing_status == "missing")
        );
    }

    #[test]
    fn receipt_merges_timing_sidecar_and_sorts_jobs() {
        let temp = tempfile::tempdir().expect("tempdir");
        let needs = temp.path().join("needs.json");
        let timings = temp.path().join("timings.json");
        write_json(
            &needs,
            &serde_json::json!({
                "z-build": {"result": "success", "outputs": {}},
                "a-docs": {"result": "success", "outputs": {"cache-hit": "true"}}
            }),
        );
        write_json(
            &timings,
            &serde_json::json!({
                "a-docs": {"duration_seconds": 90.0, "runner": "ubuntu-latest", "cache_hit": true},
                "unused": 12.0,
                "z-build": 120.0
            }),
        );

        let args = CiActualsArgs {
            needs,
            timings: Some(timings),
            output: temp.path().join("out.json"),
            ..CiActualsArgs::default()
        };

        let receipt = ci_actuals_receipt(Path::new("."), &args).expect("receipt");
        assert_eq!(receipt.status.job_count, 2);
        assert_eq!(receipt.status.timed_job_count, 2);
        assert_eq!(receipt.status.unused_timing, ["unused"]);
        assert_eq!(receipt.jobs[0].name, "a-docs");
        assert_eq!(receipt.jobs[0].duration_seconds, Some(90.0));
        assert_eq!(receipt.jobs[0].duration_minutes, Some(1.5));
        assert_eq!(receipt.jobs[0].runner.as_deref(), Some("ubuntu-latest"));
        assert_eq!(receipt.jobs[0].cache_hit, Some(true));
        assert_eq!(receipt.jobs[1].name, "z-build");
    }

    #[test]
    fn negative_timing_is_rejected() {
        let result = timing_record(TimingInput::Seconds(-1.0));
        assert!(result.is_err());
    }
}
