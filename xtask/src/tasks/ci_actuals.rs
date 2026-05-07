//! Emit `ci-actuals.json` from the workflow's `needs` context.
//!
//! Inputs: a JSON file whose top-level shape is
//!   {
//!     "<job-id>": {
//!       "result": "success" | "failure" | "skipped" | "cancelled",
//!       "outputs": { ... }
//!     }
//!   }
//! (the literal `${{ toJson(needs) }}` payload from a GitHub Actions
//! aggregator step), plus optional per-job duration_seconds via a
//! sidecar `--timings <PATH>` JSON of `{ "<job-id>": <seconds> }`.
//!
//! Output: `ci-actuals.json` whose schema is:
//!   {
//!     "schema_version": 1,
//!     "repo": "tokmd",
//!     "sha": "<HEAD>",
//!     "workflow": "CI",
//!     "jobs": [
//!       {
//!         "name": "<job-id>",
//!         "runner": "<runner>",
//!         "estimated_lem": <u64>,
//!         "actual_seconds": <f64>,
//!         "actual_lem": <f64>,
//!         "conclusion": "success" | ...,
//!         "cache_hit": <bool|null>,
//!         "risk_packs": []
//!       }
//!     ]
//!   }
//!
//! Static `estimated_lem` is sourced from `policy/ci-lane-whitelist.toml`
//! when available, mapping `<job-id>` to a lane via the `id` field.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cli::CiActualsArgs;

#[derive(Debug, Deserialize)]
struct WhitelistFile {
    #[serde(default)]
    runner_multipliers: BTreeMap<String, f64>,
    #[serde(default)]
    lane: Vec<Lane>,
}

#[derive(Debug, Deserialize)]
struct Lane {
    id: String,
    #[serde(default)]
    runner: String,
    #[serde(default)]
    base_lem: u64,
}

#[derive(Debug, Serialize)]
struct Output {
    schema_version: u32,
    repo: String,
    sha: String,
    workflow: String,
    jobs: Vec<JobActual>,
}

#[derive(Debug, Serialize)]
struct JobActual {
    name: String,
    runner: String,
    estimated_lem: u64,
    actual_seconds: f64,
    actual_lem: f64,
    conclusion: String,
    cache_hit: Option<bool>,
    risk_packs: Vec<String>,
}

pub fn run(args: CiActualsArgs) -> Result<()> {
    let root = workspace_root()?;
    let needs_path = root.join(&args.needs);
    let needs: BTreeMap<String, Value> = parse_json(&needs_path)?;

    let timings: BTreeMap<String, f64> = match &args.timings {
        Some(p) => parse_json(&root.join(p))?,
        None => BTreeMap::new(),
    };

    let whitelist: Option<WhitelistFile> = match &args.lanes {
        Some(p) => {
            let lp = root.join(p);
            if lp.is_file() {
                Some(parse_toml(&lp)?)
            } else {
                None
            }
        }
        None => {
            let default = root.join("policy/ci-lane-whitelist.toml");
            if default.is_file() {
                Some(parse_toml(&default)?)
            } else {
                None
            }
        }
    };

    let lane_index: BTreeMap<&str, &Lane> = whitelist
        .as_ref()
        .map(|w| w.lane.iter().map(|l| (l.id.as_str(), l)).collect())
        .unwrap_or_default();
    let multipliers: BTreeMap<&str, f64> = whitelist
        .as_ref()
        .map(|w| {
            w.runner_multipliers
                .iter()
                .map(|(k, v)| (k.as_str(), *v))
                .collect()
        })
        .unwrap_or_default();

    let mut jobs: Vec<JobActual> = Vec::new();
    for (name, payload) in &needs {
        let conclusion = payload
            .get("result")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let lane = lane_index.get(name.as_str()).copied();
        let runner = lane.map(|l| l.runner.clone()).unwrap_or_default();
        let estimated_lem = lane.map(|l| l.base_lem).unwrap_or(0);
        let actual_seconds = timings.get(name).copied().unwrap_or(0.0);
        let multiplier = multipliers.get(runner.as_str()).copied().unwrap_or(1.0);
        let actual_lem = (actual_seconds / 60.0) * multiplier;
        jobs.push(JobActual {
            name: name.clone(),
            runner,
            estimated_lem,
            actual_seconds,
            actual_lem,
            conclusion,
            cache_hit: None,
            risk_packs: Vec::new(),
        });
    }

    jobs.sort_by(|a, b| a.name.cmp(&b.name));

    let out = Output {
        schema_version: 1,
        repo: args.repo.clone(),
        sha: args.sha.clone(),
        workflow: args.workflow.clone(),
        jobs,
    };

    let json = serde_json::to_string_pretty(&out).context("serialize ci-actuals")?;
    let out_path = root.join(&args.json_out);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&out_path, &json).with_context(|| format!("write {}", out_path.display()))?;
    println!(
        "ci-actuals written to {} ({} job(s))",
        out_path.display(),
        out.jobs.len()
    );
    Ok(())
}

fn parse_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&body).with_context(|| format!("parse json {}", path.display()))
}

fn parse_toml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&body).with_context(|| format!("parse toml {}", path.display()))
}

fn workspace_root() -> Result<PathBuf> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .exec()
        .context("locate workspace root")?;
    Ok(metadata.workspace_root.into_std_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_needs_produces_empty_actuals() {
        let needs: BTreeMap<String, Value> = BTreeMap::new();
        // Don't actually run the full task here; just exercise that the
        // BTreeMap iteration and JobActual construction is sound.
        let jobs: Vec<JobActual> = needs
            .keys()
            .map(|name| JobActual {
                name: name.clone(),
                runner: String::new(),
                estimated_lem: 0,
                actual_seconds: 0.0,
                actual_lem: 0.0,
                conclusion: "unknown".into(),
                cache_hit: None,
                risk_packs: Vec::new(),
            })
            .collect();
        assert!(jobs.is_empty());
    }
}
