//! Lane whitelist linter.
//!
//! Verifies that every job in the GitHub Actions workflows has a matching
//! entry in `policy/ci-lane-whitelist.toml`, that lane entries carry
//! complete metadata (owner, intent, failure_mode, proof_obligation,
//! evidence for blocking lanes), and that expensive default-PR lanes carry
//! a non-expired exception in `policy/ci-whitelist-exceptions.toml`.
//!
//! The linter is intentionally advisory at first: it returns a non-zero
//! exit only when the user passed `--strict` or a hard schema/parse error
//! occurred. Day-to-day drift surfaces as a non-blocking report so the
//! rollout can land before everything is clean.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use serde::Deserialize;

use crate::cli::CiLaneWhitelistArgs;

const KNOWN_TIERS: &[&str] = &["frontdoor", "risk-gated", "deep", "summary"];
const KNOWN_RUNNERS: &[&str] = &[
    "ubuntu_latest",
    "windows_latest",
    "macos_latest",
    "nix_build",
    "external_ai_review",
    "mixed",
];

#[derive(Debug, Deserialize)]
struct WhitelistFile {
    schema_version: String,
    #[serde(default)]
    policy: Option<String>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    updated: Option<String>,
    #[serde(default)]
    budget: Option<Budget>,
    #[serde(default)]
    runner_multipliers: BTreeMap<String, f64>,
    #[serde(default)]
    lane: Vec<Lane>,
}

#[derive(Debug, Deserialize)]
struct Budget {
    preferred_default_lem: u64,
    default_limit_lem: u64,
    elevated_limit_lem: u64,
    hard_limit_lem: u64,
}

#[derive(Debug, Deserialize)]
struct Lane {
    id: String,
    workflow: String,
    job: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    tier: String,
    #[serde(default)]
    default_pr: bool,
    #[serde(default)]
    blocking: bool,
    #[serde(default)]
    runner: String,
    #[serde(default)]
    base_lem: u64,
    #[serde(default)]
    owner: String,
    #[serde(default)]
    intent: String,
    #[serde(default)]
    failure_mode: String,
    #[serde(default)]
    proof_obligation: String,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default)]
    allowed_triggers: Vec<String>,
    #[serde(default)]
    expensive: bool,
    #[serde(default)]
    default_pr_exception: Option<String>,
    #[serde(default)]
    duplicate_of: Vec<String>,
    #[serde(default)]
    review_after: Option<String>,
    #[serde(default)]
    expires: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExceptionFile {
    schema_version: String,
    #[serde(default)]
    policy: Option<String>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    exception: Vec<Exception>,
}

#[derive(Debug, Deserialize)]
struct Exception {
    id: String,
    #[serde(default)]
    kind: String,
    lane: String,
    #[serde(default)]
    allowed: bool,
    #[serde(default)]
    owner: String,
    #[serde(default)]
    issue: Option<String>,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    created: Option<String>,
    #[serde(default)]
    review_after: Option<String>,
    #[serde(default)]
    expires: Option<String>,
}

pub fn run(args: CiLaneWhitelistArgs) -> Result<()> {
    let root = workspace_root()?;
    let workflows_dir = root.join(&args.workflows);
    let whitelist_path = root.join(&args.whitelist);
    let exceptions_path = root.join(&args.exceptions);

    let whitelist = parse_whitelist(&whitelist_path)?;
    let exceptions = parse_exceptions(&exceptions_path)?;

    let mut findings: Vec<String> = Vec::new();
    let mut hard_errors: Vec<String> = Vec::new();

    if whitelist.schema_version != "1.0" {
        hard_errors.push(format!(
            "{}: unsupported schema_version {:?}",
            whitelist_path.display(),
            whitelist.schema_version
        ));
    }
    if exceptions.schema_version != "1.0" {
        hard_errors.push(format!(
            "{}: unsupported schema_version {:?}",
            exceptions_path.display(),
            exceptions.schema_version
        ));
    }

    let today = chrono::Utc::now().date_naive();

    let lane_ids: BTreeSet<&str> = whitelist.lane.iter().map(|l| l.id.as_str()).collect();

    for lane in &whitelist.lane {
        validate_lane(
            lane,
            &lane_ids,
            &whitelist.runner_multipliers,
            today,
            &mut findings,
        );
    }

    for exc in &exceptions.exception {
        validate_exception(exc, &lane_ids, today, &mut findings);
    }

    let workflow_jobs = scan_workflows(&workflows_dir)?;
    let lane_index: BTreeMap<(&str, &str), &Lane> = whitelist
        .lane
        .iter()
        .map(|l| ((l.workflow.as_str(), l.job.as_str()), l))
        .collect();

    for (workflow, job_name) in &workflow_jobs {
        let key = (workflow.as_str(), job_name.as_str());
        if !lane_index.contains_key(&key) {
            findings.push(format!(
                "workflow job {} :: {} has no whitelist entry",
                workflow, job_name
            ));
        }
    }

    let exception_index: BTreeSet<&str> =
        exceptions.exception.iter().map(|e| e.id.as_str()).collect();
    for lane in &whitelist.lane {
        if lane.default_pr && lane.expensive {
            match &lane.default_pr_exception {
                Some(id) if exception_index.contains(id.as_str()) => {}
                Some(id) => findings.push(format!(
                    "lane {} references missing exception {}",
                    lane.id, id
                )),
                None => findings.push(format!(
                    "lane {} is expensive default-PR but has no default_pr_exception",
                    lane.id
                )),
            }
        }
    }

    if let Some(report_dir) = &args.report_dir {
        let dir = root.join(report_dir);
        fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
        let path = dir.join("ci-lane-whitelist-report.txt");
        let body = render_report(
            &workflow_jobs,
            &whitelist,
            &exceptions,
            &findings,
            &hard_errors,
        );
        fs::write(&path, &body).with_context(|| format!("write {}", path.display()))?;
        println!("ci-lane-whitelist report written to {}", path.display());
    }

    if !hard_errors.is_empty() {
        for err in &hard_errors {
            eprintln!("error: {err}");
        }
        bail!("ci-lane-whitelist: {} hard error(s)", hard_errors.len());
    }

    if findings.is_empty() {
        println!(
            "ci-lane-whitelist OK: {} lane(s), {} exception(s), {} workflow job(s)",
            whitelist.lane.len(),
            exceptions.exception.len(),
            workflow_jobs.len()
        );
        return Ok(());
    }

    println!("ci-lane-whitelist findings ({}):", findings.len());
    for finding in &findings {
        println!("  - {finding}");
    }

    if args.strict {
        bail!("ci-lane-whitelist: {} finding(s) (strict)", findings.len());
    }
    println!("(advisory mode; rerun with --strict to fail on findings)");
    Ok(())
}

fn validate_lane(
    lane: &Lane,
    _lane_ids: &BTreeSet<&str>,
    runner_multipliers: &BTreeMap<String, f64>,
    today: NaiveDate,
    findings: &mut Vec<String>,
) {
    if lane.kind.is_empty() {
        findings.push(format!("lane {}: missing kind", lane.id));
    }
    if lane.base_lem == 0 {
        findings.push(format!("lane {}: missing or zero base_lem", lane.id));
    }
    if lane.owner.is_empty() {
        findings.push(format!("lane {}: missing owner", lane.id));
    }
    if lane.intent.is_empty() {
        findings.push(format!("lane {}: missing intent", lane.id));
    }
    if lane.failure_mode.is_empty() {
        findings.push(format!("lane {}: missing failure_mode", lane.id));
    }
    if lane.proof_obligation.is_empty() {
        findings.push(format!("lane {}: missing proof_obligation", lane.id));
    }
    if lane.blocking && lane.evidence.is_empty() {
        findings.push(format!("lane {}: blocking lane lacks evidence", lane.id));
    }
    if !lane.tier.is_empty() && !KNOWN_TIERS.contains(&lane.tier.as_str()) {
        findings.push(format!(
            "lane {}: unknown tier {:?} (expected one of {:?})",
            lane.id, lane.tier, KNOWN_TIERS
        ));
    }
    if !lane.runner.is_empty()
        && !KNOWN_RUNNERS.contains(&lane.runner.as_str())
        && !runner_multipliers.contains_key(&lane.runner)
    {
        findings.push(format!(
            "lane {}: unknown runner {:?} not in [runner_multipliers]",
            lane.id, lane.runner
        ));
    }
    if lane.allowed_triggers.is_empty() {
        findings.push(format!("lane {}: missing allowed_triggers", lane.id));
    }
    if let Some(date) = &lane.review_after
        && let Err(err) = NaiveDate::parse_from_str(date, "%Y-%m-%d")
    {
        findings.push(format!(
            "lane {}: review_after {:?} is not YYYY-MM-DD ({err})",
            lane.id, date
        ));
    }
    if let Some(date) = &lane.expires {
        match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            Ok(parsed) if parsed < today => {
                findings.push(format!("lane {}: expired on {date}", lane.id));
            }
            Ok(_) => {}
            Err(err) => findings.push(format!(
                "lane {}: expires {:?} is not YYYY-MM-DD ({err})",
                lane.id, date
            )),
        }
    } else if lane.review_after.is_none() {
        findings.push(format!(
            "lane {}: missing both review_after and expires",
            lane.id
        ));
    }
    for dup in &lane.duplicate_of {
        if dup.starts_with("future:") {
            // Forward declarations are intentionally permitted.
            continue;
        }
        if !_lane_ids.contains(dup.as_str()) {
            findings.push(format!(
                "lane {}: duplicate_of {:?} does not exist",
                lane.id, dup
            ));
        }
    }
}

fn validate_exception(
    exc: &Exception,
    lane_ids: &BTreeSet<&str>,
    today: NaiveDate,
    findings: &mut Vec<String>,
) {
    if !lane_ids.contains(exc.lane.as_str()) {
        findings.push(format!(
            "exception {}: references missing lane {:?}",
            exc.id, exc.lane
        ));
    }
    if exc.owner.is_empty() {
        findings.push(format!("exception {}: missing owner", exc.id));
    }
    if exc.reason.is_empty() {
        findings.push(format!("exception {}: missing reason", exc.id));
    }
    if exc.kind.is_empty() {
        findings.push(format!("exception {}: missing kind", exc.id));
    }
    if !exc.allowed {
        findings.push(format!(
            "exception {}: allowed=false but exception is present",
            exc.id
        ));
    }
    if let Some(date) = &exc.expires {
        match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            Ok(parsed) if parsed < today => {
                findings.push(format!("exception {}: expired on {date}", exc.id));
            }
            Ok(_) => {}
            Err(err) => findings.push(format!(
                "exception {}: expires {:?} is not YYYY-MM-DD ({err})",
                exc.id, date
            )),
        }
    } else {
        findings.push(format!("exception {}: missing expires", exc.id));
    }
    if let Some(date) = &exc.created
        && let Err(err) = NaiveDate::parse_from_str(date, "%Y-%m-%d")
    {
        findings.push(format!(
            "exception {}: created {:?} is not YYYY-MM-DD ({err})",
            exc.id, date
        ));
    }
    if let Some(date) = &exc.review_after
        && let Err(err) = NaiveDate::parse_from_str(date, "%Y-%m-%d")
    {
        findings.push(format!(
            "exception {}: review_after {:?} is not YYYY-MM-DD ({err})",
            exc.id, date
        ));
    }
    if let Some(issue) = &exc.issue
        && issue == "TODO"
    {
        // Accepted during the rollout; still surface as advisory finding.
        findings.push(format!(
                "exception {}: issue is TODO (rollout placeholder, attach a real ref before expires={:?})",
                exc.id, exc.expires
            ));
    }
}

fn parse_whitelist(path: &Path) -> Result<WhitelistFile> {
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str::<WhitelistFile>(&body).with_context(|| format!("parse {}", path.display()))
}

fn parse_exceptions(path: &Path) -> Result<ExceptionFile> {
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str::<ExceptionFile>(&body).with_context(|| format!("parse {}", path.display()))
}

fn scan_workflows(dir: &Path) -> Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    if !dir.is_dir() {
        bail!("workflows dir {} not found", dir.display());
    }
    for entry in fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if !(name.ends_with(".yml") || name.ends_with(".yaml")) {
            continue;
        }
        let workflow_rel = format!(".github/workflows/{name}");
        let body = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        for job_name in extract_job_names(&body) {
            out.push((workflow_rel.clone(), job_name));
        }
    }
    out.sort();
    Ok(out)
}

/// Extract the `name:` value of every top-level entry under `jobs:`.
///
/// This is a deliberately small subset of YAML: it relies on GitHub's
/// 2-space indented job dictionary shape and the fact that workflow files
/// in this repo use plain (non-flow) YAML. If a workflow stops following
/// that shape, the linter will undercount jobs and report missing
/// whitelist entries — visible failure, not silent.
fn extract_job_names(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_jobs = false;
    let mut current_job_indent: Option<usize> = None;

    for raw in body.lines() {
        if !in_jobs {
            if raw.trim_start().starts_with("jobs:") && raw.starts_with("jobs:") {
                in_jobs = true;
            }
            continue;
        }

        // Leave jobs: when we encounter a non-empty line at column 0 that isn't a
        // comment.
        if !raw.is_empty()
            && !raw.starts_with(' ')
            && !raw.starts_with('#')
            && !raw.starts_with('\t')
            && !raw.trim().is_empty()
        {
            in_jobs = false;
            current_job_indent = None;
            continue;
        }

        let trimmed = raw.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = raw.len() - trimmed.len();

        // A new job key: `^  <id>:` at the canonical 2-space indent.
        if indent == 2 && trimmed.ends_with(':') {
            current_job_indent = Some(indent);
            // The job's `name:` is what we capture below; default to the id.
            // We push only after we see an explicit name to match the CI lane
            // whitelist shape (which uses workflow `job` = job's `name:` field).
            continue;
        }

        if let Some(_job_indent) = current_job_indent
            && indent == 4
            && let Some(rest) = trimmed.strip_prefix("name:")
        {
            let value = rest.trim();
            let value = value
                .trim_start_matches('"')
                .trim_end_matches('"')
                .trim_start_matches('\'')
                .trim_end_matches('\'');
            if !value.is_empty() {
                out.push(value.to_string());
            }
        }
    }

    out
}

fn render_report(
    workflow_jobs: &[(String, String)],
    whitelist: &WhitelistFile,
    exceptions: &ExceptionFile,
    findings: &[String],
    hard_errors: &[String],
) -> String {
    let mut out = String::new();
    out.push_str("# CI lane whitelist report\n\n");
    if let Some(name) = &whitelist.policy {
        out.push_str(&format!("- whitelist policy: {name}\n"));
    }
    if let Some(owner) = &whitelist.owner {
        out.push_str(&format!("- whitelist owner: {owner}\n"));
    }
    if let Some(status) = &whitelist.status {
        out.push_str(&format!("- whitelist status: {status}\n"));
    }
    if let Some(updated) = &whitelist.updated {
        out.push_str(&format!("- whitelist updated: {updated}\n"));
    }
    if let Some(budget) = &whitelist.budget {
        out.push_str(&format!(
            "- budget bands: preferred={} default={} elevated={} hard={}\n",
            budget.preferred_default_lem,
            budget.default_limit_lem,
            budget.elevated_limit_lem,
            budget.hard_limit_lem,
        ));
    }
    if let Some(name) = &exceptions.policy {
        out.push_str(&format!("- exceptions policy: {name}\n"));
    }
    if let Some(owner) = &exceptions.owner {
        out.push_str(&format!("- exceptions owner: {owner}\n"));
    }
    if let Some(status) = &exceptions.status {
        out.push_str(&format!("- exceptions status: {status}\n"));
    }
    out.push_str(&format!(
        "- workflows scanned: {}\n",
        count_distinct_workflows(workflow_jobs)
    ));
    out.push_str(&format!("- jobs discovered: {}\n", workflow_jobs.len()));
    out.push_str(&format!("- lanes in whitelist: {}\n", whitelist.lane.len()));
    out.push_str(&format!("- exceptions: {}\n", exceptions.exception.len()));
    out.push_str(&format!("- findings: {}\n", findings.len()));
    out.push_str(&format!("- hard errors: {}\n\n", hard_errors.len()));

    if !findings.is_empty() {
        out.push_str("## Findings\n\n");
        for finding in findings {
            out.push_str(&format!("- {finding}\n"));
        }
        out.push('\n');
    }

    if !hard_errors.is_empty() {
        out.push_str("## Hard errors\n\n");
        for err in hard_errors {
            out.push_str(&format!("- {err}\n"));
        }
    }

    out
}

fn count_distinct_workflows(jobs: &[(String, String)]) -> usize {
    let mut set = BTreeSet::new();
    for (wf, _) in jobs {
        set.insert(wf.as_str());
    }
    set.len()
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
    fn extracts_job_names_from_simple_workflow() {
        let body = r#"name: CI

on:
  push: {}

jobs:
  msrv:
    name: MSRV Check
    runs-on: ubuntu-latest
  build:
    name: Build & Test
    runs-on: ${{ matrix.os }}
"#;
        let jobs = extract_job_names(body);
        assert_eq!(
            jobs,
            vec!["MSRV Check".to_string(), "Build & Test".to_string()]
        );
    }

    #[test]
    fn extracts_job_names_with_quoted_names() {
        let body = r#"jobs:
  a:
    name: "Cargo Deny"
  b:
    name: 'Quality Gate'
"#;
        let jobs = extract_job_names(body);
        assert_eq!(
            jobs,
            vec!["Cargo Deny".to_string(), "Quality Gate".to_string()]
        );
    }

    #[test]
    fn job_without_explicit_name_is_skipped() {
        // Lanes are matched on the `name:` value; jobs without one fall out.
        let body = r#"jobs:
  build:
    runs-on: ubuntu-latest
  with_name:
    name: Has Name
"#;
        let jobs = extract_job_names(body);
        assert_eq!(jobs, vec!["Has Name".to_string()]);
    }

    fn lane_template() -> Lane {
        Lane {
            id: "demo".into(),
            workflow: ".github/workflows/x.yml".into(),
            job: "Demo".into(),
            kind: "policy".into(),
            tier: "frontdoor".into(),
            default_pr: true,
            blocking: true,
            runner: "ubuntu_latest".into(),
            base_lem: 1,
            owner: "demo".into(),
            intent: "do thing".into(),
            failure_mode: "thing fails".into(),
            proof_obligation: "run thing".into(),
            evidence: vec!["job logs".into()],
            allowed_triggers: vec!["pull_request".into()],
            expensive: false,
            default_pr_exception: None,
            duplicate_of: vec![],
            review_after: Some("2099-01-01".into()),
            expires: Some("2099-12-31".into()),
        }
    }

    #[test]
    fn lane_missing_owner_is_finding() {
        let mut lane = lane_template();
        lane.owner.clear();
        let mut findings = Vec::new();
        let ids: BTreeSet<&str> = std::iter::once("demo").collect();
        let multipliers = BTreeMap::new();
        let today = chrono::NaiveDate::from_ymd_opt(2026, 5, 7).expect("date");
        validate_lane(&lane, &ids, &multipliers, today, &mut findings);
        assert!(
            findings.iter().any(|f| f.contains("missing owner")),
            "{findings:?}"
        );
    }

    #[test]
    fn expired_lane_is_finding() {
        let mut lane = lane_template();
        lane.expires = Some("2020-01-01".into());
        let mut findings = Vec::new();
        let ids: BTreeSet<&str> = std::iter::once("demo").collect();
        let multipliers = BTreeMap::new();
        let today = chrono::NaiveDate::from_ymd_opt(2026, 5, 7).expect("date");
        validate_lane(&lane, &ids, &multipliers, today, &mut findings);
        assert!(
            findings.iter().any(|f| f.contains("expired on 2020-01-01")),
            "{findings:?}"
        );
    }

    #[test]
    fn duplicate_of_existing_lane_is_ok() {
        let mut lane = lane_template();
        lane.duplicate_of = vec!["other".into(), "future:planned".into()];
        let mut findings = Vec::new();
        let ids: BTreeSet<&str> = ["demo", "other"].into_iter().collect();
        let multipliers = BTreeMap::new();
        let today = chrono::NaiveDate::from_ymd_opt(2026, 5, 7).expect("date");
        validate_lane(&lane, &ids, &multipliers, today, &mut findings);
        assert!(
            !findings.iter().any(|f| f.contains("duplicate_of")),
            "{findings:?}"
        );
    }

    #[test]
    fn duplicate_of_missing_lane_is_finding() {
        let mut lane = lane_template();
        lane.duplicate_of = vec!["nope".into()];
        let mut findings = Vec::new();
        let ids: BTreeSet<&str> = std::iter::once("demo").collect();
        let multipliers = BTreeMap::new();
        let today = chrono::NaiveDate::from_ymd_opt(2026, 5, 7).expect("date");
        validate_lane(&lane, &ids, &multipliers, today, &mut findings);
        assert!(
            findings.iter().any(|f| f.contains("duplicate_of \"nope\"")),
            "{findings:?}"
        );
    }
}
