//! LEM-aware advisory PR Plan.
//!
//! Inputs:
//! - `policy/ci-lane-whitelist.toml` — lane catalogue + LEM budget.
//! - `policy/ci-risk-packs.toml` — path → lane routing.
//! - the PR diff `base..head` — selects risk packs.
//!
//! Output: `ci-plan.json` describing which risk packs were hit, which
//! lanes will run on this PR, the estimated LEM, and the LEM band.
//! It can also write `proof-pack-route.json`, a smaller changed-file to
//! proof-pack receipt for CI routing/debugging.
//!
//! Advisory only — does not change which jobs actually run. PR 12 wires
//! workflows to consume the plan; PR 14 layers in the budget warning.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};

use crate::cli::CiPlanArgs;

#[derive(Debug, Deserialize)]
struct WhitelistFile {
    #[serde(default)]
    budget: Option<Budget>,
    #[serde(default)]
    runner_multipliers: BTreeMap<String, f64>,
    #[serde(default)]
    lane: Vec<Lane>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct Budget {
    preferred_default_lem: u64,
    default_limit_lem: u64,
    elevated_limit_lem: u64,
    hard_limit_lem: u64,
}

#[derive(Debug, Deserialize, Clone)]
struct Lane {
    id: String,
    #[serde(default)]
    workflow: String,
    #[serde(default)]
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
    expensive: bool,
}

#[derive(Debug, Deserialize)]
struct RiskPacksFile {
    #[serde(default)]
    risk_pack: BTreeMap<String, RiskPack>,
}

#[derive(Debug, Deserialize)]
struct RiskPack {
    #[serde(default)]
    description: String,
    #[serde(default)]
    paths: Vec<String>,
    #[serde(default)]
    lanes: Vec<String>,
    #[serde(default)]
    deep_lanes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct PlanOutput {
    schema_version: u32,
    base: String,
    head: String,
    labels: Vec<String>,
    changed_files: Vec<String>,
    risk_packs_hit: Vec<RiskPackHit>,
    lanes_selected: Vec<LaneSelection>,
    estimated_lem: u64,
    band: String,
    budget: BudgetView,
}

#[derive(Debug, Serialize)]
struct RiskPackHit {
    name: String,
    description: String,
    matched_files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct LaneSelection {
    id: String,
    workflow: String,
    job: String,
    kind: String,
    tier: String,
    runner: String,
    blocking: bool,
    estimated_lem: u64,
    /// `static`, `learned-p50`, or `learned-p90` once a calibration window exists.
    estimate_source: String,
    /// Optional learned percentiles in LEM, when `--actuals-dir` is provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    learned_p50_lem: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    learned_p90_lem: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    learned_p95_lem: Option<f64>,
    reason: String,
}

#[derive(Debug, Serialize, Clone)]
struct ChangedFileRoute {
    path: String,
    surface: String,
    proof_packs: Vec<String>,
    reason: String,
    policy: String,
    lanes: Vec<String>,
    deep_lanes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SkippedByPolicy {
    lane: String,
    status: String,
    reason: String,
    matched_files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RouteSummary {
    changed_file_count: usize,
    routed_file_count: usize,
    unmatched_file_count: usize,
    skipped_lane_count: usize,
}

#[derive(Debug, Serialize)]
struct ProofPackRouteReceipt {
    schema: &'static str,
    schema_version: u32,
    base: String,
    head: String,
    labels: Vec<String>,
    changed_files: Vec<ChangedFileRoute>,
    unmatched_files: Vec<String>,
    skipped_by_policy: Vec<SkippedByPolicy>,
    summary: RouteSummary,
}

#[derive(Debug)]
struct RouteAnalysis {
    changed_files: Vec<ChangedFileRoute>,
    unmatched_files: Vec<String>,
    matched_by_pack: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Clone, Copy)]
struct BudgetView {
    preferred_default_lem: u64,
    default_limit_lem: u64,
    elevated_limit_lem: u64,
    hard_limit_lem: u64,
}

pub fn run(args: CiPlanArgs) -> Result<()> {
    let root = workspace_root()?;
    let whitelist: WhitelistFile = parse_toml(&root.join(&args.lanes), "ci-lane-whitelist")?;
    let risk_packs: RiskPacksFile = parse_toml(&root.join(&args.risk_packs), "ci-risk-packs")?;

    let actuals = match &args.actuals_dir {
        Some(dir) => load_actuals(&root.join(dir))?,
        None => BTreeMap::new(),
    };

    let budget = whitelist.budget.unwrap_or(Budget {
        preferred_default_lem: 25,
        default_limit_lem: 35,
        elevated_limit_lem: 75,
        hard_limit_lem: 125,
    });

    let labels = parse_labels(args.labels_json.as_deref());
    let changed = git_diff_names(&root, &args.base, &args.head)?;

    let lane_index: BTreeMap<&str, &Lane> =
        whitelist.lane.iter().map(|l| (l.id.as_str(), l)).collect();
    let route_analysis = route_changed_files(&changed, &risk_packs, &lane_index)?;

    let mut hits: Vec<RiskPackHit> = Vec::new();
    let mut selected: BTreeMap<String, LaneSelection> = BTreeMap::new();

    let labels_set: BTreeSet<&str> = labels.iter().map(|s| s.as_str()).collect();
    let want_full_ci = labels_set.contains("full-ci");

    // Always-on default-PR lanes from the whitelist (frontdoor + cheap
    // policy lanes). Risk packs add to this rather than replacing it.
    for lane in whitelist
        .lane
        .iter()
        .filter(|l| l.default_pr && !l.expensive)
    {
        selected.entry(lane.id.clone()).or_insert_with(|| {
            lane_to_selection(lane, &whitelist.runner_multipliers, &actuals, "default_pr")
        });
    }

    for (name, pack) in &risk_packs.risk_pack {
        let matched = route_analysis
            .matched_by_pack
            .get(name)
            .cloned()
            .unwrap_or_default();
        if matched.is_empty() {
            continue;
        }
        hits.push(RiskPackHit {
            name: name.clone(),
            description: pack.description.clone(),
            matched_files: matched.clone(),
        });
        for lane_id in &pack.lanes {
            if let Some(lane) = lane_index.get(lane_id.as_str()) {
                selected.entry(lane.id.clone()).or_insert_with(|| {
                    lane_to_selection(
                        lane,
                        &whitelist.runner_multipliers,
                        &actuals,
                        &format!("risk_pack:{name}"),
                    )
                });
            }
        }
        if want_full_ci || labels_set.contains(name.as_str()) {
            for lane_id in &pack.deep_lanes {
                if let Some(lane) = lane_index.get(lane_id.as_str()) {
                    selected.entry(lane.id.clone()).or_insert_with(|| {
                        lane_to_selection(
                            lane,
                            &whitelist.runner_multipliers,
                            &actuals,
                            &format!("risk_pack:{name}:deep"),
                        )
                    });
                }
            }
        }
    }

    if want_full_ci {
        for lane in &whitelist.lane {
            selected.entry(lane.id.clone()).or_insert_with(|| {
                lane_to_selection(
                    lane,
                    &whitelist.runner_multipliers,
                    &actuals,
                    "label:full-ci",
                )
            });
        }
    }

    let selected_ids: BTreeSet<String> = selected.keys().cloned().collect();
    let skipped_by_policy = skipped_by_policy(&whitelist, &selected_ids, &route_analysis);
    let route_receipt = ProofPackRouteReceipt {
        schema: "tokmd.proof_pack_route.v1",
        schema_version: 1,
        base: args.base.clone(),
        head: args.head.clone(),
        labels: labels.clone(),
        changed_files: route_analysis.changed_files.clone(),
        unmatched_files: route_analysis.unmatched_files.clone(),
        summary: RouteSummary {
            changed_file_count: changed.len(),
            routed_file_count: route_analysis.changed_files.len(),
            unmatched_file_count: route_analysis.unmatched_files.len(),
            skipped_lane_count: skipped_by_policy.len(),
        },
        skipped_by_policy,
    };

    let lanes_selected: Vec<LaneSelection> = selected.into_values().collect();
    let estimated_lem: u64 = lanes_selected.iter().map(|l| l.estimated_lem).sum();
    let band = classify_band(estimated_lem, budget);

    let plan = PlanOutput {
        schema_version: 1,
        base: args.base.clone(),
        head: args.head.clone(),
        labels,
        changed_files: changed,
        risk_packs_hit: hits,
        lanes_selected,
        estimated_lem,
        band: band.to_string(),
        budget: BudgetView {
            preferred_default_lem: budget.preferred_default_lem,
            default_limit_lem: budget.default_limit_lem,
            elevated_limit_lem: budget.elevated_limit_lem,
            hard_limit_lem: budget.hard_limit_lem,
        },
    };

    let json = serde_json::to_string_pretty(&plan).context("serialize ci-plan")?;
    if let Some(out) = &args.json_out {
        let path = root.join(out);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(&path, &json).with_context(|| format!("write {}", path.display()))?;
        println!("ci-plan written to {}", path.display());
    } else {
        println!("{json}");
    }

    if let Some(out) = &args.route_json_out {
        let route_json =
            serde_json::to_string_pretty(&route_receipt).context("serialize proof-pack route")?;
        let path = root.join(out);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(&path, route_json).with_context(|| format!("write {}", path.display()))?;
        println!("proof-pack route written to {}", path.display());
    }

    if let Some(summary_path) = &args.github_summary {
        let body = render_step_summary(&plan);
        let mut existing = fs::read_to_string(summary_path).unwrap_or_default();
        existing.push_str(&body);
        fs::write(summary_path, existing)
            .with_context(|| format!("append {}", summary_path.display()))?;
    }

    if let Some(output_path) = &args.github_output {
        let body = render_github_outputs(&plan);
        fs::write(output_path, body).with_context(|| format!("write {}", output_path.display()))?;
    }

    println!(
        "ci-plan: {} risk-packs hit, {} lane(s) selected, estimated {} LEM ({})",
        plan.risk_packs_hit.len(),
        plan.lanes_selected.len(),
        plan.estimated_lem,
        plan.band,
    );

    for message in maybe_budget_annotation_messages(&plan, !args.no_budget_annotations) {
        println!("{message}");
    }

    if args.enforce && budget_requires_override(&plan) {
        bail!(
            "ci-plan: estimated {} LEM exceeds hard ceiling {}",
            plan.estimated_lem,
            plan.budget.hard_limit_lem
        );
    }

    Ok(())
}

fn budget_label_flags(labels: &[String]) -> (bool, bool) {
    let labels_set: BTreeSet<&str> = labels.iter().map(|s| s.as_str()).collect();
    let override_present =
        labels_set.contains("full-ci") || labels_set.contains("ci-budget-override");
    let ack_present = labels_set.contains("ci-budget-ack");
    (override_present, ack_present)
}

fn budget_requires_override(plan: &PlanOutput) -> bool {
    let (override_present, _) = budget_label_flags(&plan.labels);
    plan.band == "override-required" && !override_present
}

fn budget_annotation_messages(plan: &PlanOutput) -> Vec<String> {
    let (override_present, ack_present) = budget_label_flags(&plan.labels);

    match plan.band.as_str() {
        "elevated" if !ack_present => vec![format!(
            "::warning::PR plan estimated {} LEM (elevated band; expected ≤ {}). \
            Apply `ci-budget-ack` to acknowledge.",
            plan.estimated_lem, plan.budget.default_limit_lem
        )],
        "high-cost" if !ack_present => vec![format!(
            "::warning::PR plan estimated {} LEM (high-cost band; expected ≤ {}). \
            Review the PR scope and apply `ci-budget-ack` if the spend is intentional; \
            hard override is required only above {} LEM.",
            plan.estimated_lem, plan.budget.elevated_limit_lem, plan.budget.hard_limit_lem
        )],
        "override-required" if override_present => vec![format!(
            "::warning::PR plan estimated {} LEM (>{} hard ceiling) — \
            proceeding because override label is present.",
            plan.estimated_lem, plan.budget.hard_limit_lem
        )],
        "override-required" => vec![format!(
            "::error::PR plan estimated {} LEM (>{} hard ceiling) — \
            apply `ci-budget-override` or `full-ci` to bypass, or split the PR.",
            plan.estimated_lem, plan.budget.hard_limit_lem
        )],
        _ => Vec::new(),
    }
}

fn maybe_budget_annotation_messages(plan: &PlanOutput, emit_annotations: bool) -> Vec<String> {
    if emit_annotations {
        budget_annotation_messages(plan)
    } else {
        Vec::new()
    }
}

fn parse_toml<T: for<'de> Deserialize<'de>>(path: &Path, kind: &str) -> Result<T> {
    let body =
        fs::read_to_string(path).with_context(|| format!("read {} ({kind})", path.display()))?;
    toml::from_str(&body).with_context(|| format!("parse {} ({kind})", path.display()))
}

fn parse_labels(raw: Option<&str>) -> Vec<String> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    // GITHUB_PR_LABELS_JSON shape is `[{"name":"X"},{"name":"Y"}]` or
    // `["X","Y"]` depending on how it was extracted. Accept both.
    if let Ok(values) = serde_json::from_str::<Vec<LabelEntry>>(trimmed) {
        return values.into_iter().map(|v| v.into_name()).collect();
    }
    if let Ok(values) = serde_json::from_str::<Vec<String>>(trimmed) {
        return values;
    }
    // Fall back to a comma-separated list.
    trimmed
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum LabelEntry {
    Object { name: String },
    Bare(String),
}

impl LabelEntry {
    fn into_name(self) -> String {
        match self {
            LabelEntry::Object { name } => name,
            LabelEntry::Bare(s) => s,
        }
    }
}

fn git_diff_names(root: &Path, base: &str, head: &str) -> Result<Vec<String>> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--name-only")
        .arg(format!("{base}...{head}"))
        .current_dir(root)
        .output()
        .context("git diff")?;
    if !output.status.success() {
        bail!(
            "git diff exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

fn match_paths(globs: &[String], files: &[String]) -> Result<Vec<String>> {
    let mut builder = GlobSetBuilder::new();
    for g in globs {
        builder.add(Glob::new(g).with_context(|| format!("compile glob {g:?}"))?);
    }
    let set: GlobSet = builder.build()?;
    let mut matched: Vec<String> = files.iter().filter(|f| set.is_match(f)).cloned().collect();
    matched.sort();
    matched.dedup();
    Ok(matched)
}

fn route_changed_files(
    changed: &[String],
    risk_packs: &RiskPacksFile,
    lane_index: &BTreeMap<&str, &Lane>,
) -> Result<RouteAnalysis> {
    let mut file_to_packs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut matched_by_pack: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (name, pack) in &risk_packs.risk_pack {
        let matched = match_paths(&pack.paths, changed)?;
        if matched.is_empty() {
            continue;
        }
        for file in &matched {
            file_to_packs
                .entry(file.clone())
                .or_default()
                .push(name.clone());
        }
        matched_by_pack.insert(name.clone(), matched);
    }

    let mut changed_files = Vec::new();
    let mut unmatched_files = Vec::new();
    for file in changed {
        let Some(pack_names) = file_to_packs.get(file) else {
            unmatched_files.push(file.clone());
            continue;
        };

        let mut lanes: BTreeSet<String> = BTreeSet::new();
        let mut deep_lanes: BTreeSet<String> = BTreeSet::new();
        for pack_name in pack_names {
            if let Some(pack) = risk_packs.risk_pack.get(pack_name) {
                lanes.extend(pack.lanes.iter().cloned());
                deep_lanes.extend(pack.deep_lanes.iter().cloned());
            }
        }

        changed_files.push(ChangedFileRoute {
            path: file.clone(),
            surface: if pack_names.len() == 1 {
                pack_names
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string())
            } else {
                "multiple".to_string()
            },
            proof_packs: pack_names.clone(),
            reason: "manifest_match".to_string(),
            policy: route_policy(&lanes, &deep_lanes, lane_index).to_string(),
            lanes: lanes.into_iter().collect(),
            deep_lanes: deep_lanes.into_iter().collect(),
        });
    }

    Ok(RouteAnalysis {
        changed_files,
        unmatched_files,
        matched_by_pack,
    })
}

fn route_policy(
    lanes: &BTreeSet<String>,
    deep_lanes: &BTreeSet<String>,
    lane_index: &BTreeMap<&str, &Lane>,
) -> &'static str {
    for lane_id in lanes.iter().chain(deep_lanes.iter()) {
        if lane_index
            .get(lane_id.as_str())
            .is_some_and(|lane| lane.blocking)
        {
            return "blocking";
        }
    }
    "advisory"
}

fn skipped_by_policy(
    whitelist: &WhitelistFile,
    selected_ids: &BTreeSet<String>,
    route: &RouteAnalysis,
) -> Vec<SkippedByPolicy> {
    let docs_only = is_docs_only_route(route);

    whitelist
        .lane
        .iter()
        .filter(|lane| lane.expensive && !selected_ids.contains(&lane.id))
        .map(|lane| {
            let direct_matched_files = matched_files_for_lane(route, &lane.id, false);
            let deep_matched_files = matched_files_for_lane(route, &lane.id, true);
            let (reason, matched_files) = if !deep_matched_files.is_empty() {
                ("deep_lane_requires_label", deep_matched_files)
            } else if !direct_matched_files.is_empty() {
                ("not_selected_by_policy", direct_matched_files)
            } else if docs_only {
                (
                    "docs_only_change",
                    route
                        .changed_files
                        .iter()
                        .map(|file| file.path.clone())
                        .collect(),
                )
            } else if route.changed_files.is_empty() && route.unmatched_files.is_empty() {
                ("no_changed_files", Vec::new())
            } else {
                ("not_selected_for_changed_surface", Vec::new())
            };

            SkippedByPolicy {
                lane: lane.id.clone(),
                status: "skipped_by_policy".to_string(),
                reason: reason.to_string(),
                matched_files,
            }
        })
        .collect()
}

fn matched_files_for_lane(route: &RouteAnalysis, lane_id: &str, deep: bool) -> Vec<String> {
    route
        .changed_files
        .iter()
        .filter(|file| {
            if deep {
                file.deep_lanes.iter().any(|candidate| candidate == lane_id)
            } else {
                file.lanes.iter().any(|candidate| candidate == lane_id)
            }
        })
        .map(|file| file.path.clone())
        .collect()
}

fn is_docs_only_route(route: &RouteAnalysis) -> bool {
    !route.changed_files.is_empty()
        && route.unmatched_files.is_empty()
        && route.changed_files.iter().all(|file| {
            !file.proof_packs.is_empty() && file.proof_packs.iter().all(|pack| pack == "docs")
        })
}

fn lane_to_selection(
    lane: &Lane,
    runner_multipliers: &BTreeMap<String, f64>,
    actuals: &BTreeMap<String, Vec<f64>>,
    reason: &str,
) -> LaneSelection {
    let multiplier = runner_multipliers
        .get(lane.runner.as_str())
        .copied()
        .unwrap_or(1.0);
    let static_lem = ((lane.base_lem as f64) * multiplier).round() as u64;

    let (estimate_source, estimated, p50, p90, p95) = match actuals.get(&lane.id) {
        Some(samples) if !samples.is_empty() => {
            let p50_secs = percentile(samples, 0.50);
            let p90_secs = percentile(samples, 0.90);
            let p95_secs = percentile(samples, 0.95);
            let to_lem = |secs: f64| (secs / 60.0) * multiplier;
            let p50_lem = to_lem(p50_secs);
            let p90_lem = to_lem(p90_secs);
            let p95_lem = to_lem(p95_secs);
            // estimate = max(static_floor, p50 * 1.15)
            let learned = (p50_lem * 1.15).round() as u64;
            let estimate = static_lem.max(learned);
            (
                "learned-p50".to_string(),
                estimate,
                Some(p50_lem),
                Some(p90_lem),
                Some(p95_lem),
            )
        }
        _ => ("static".to_string(), static_lem, None, None, None),
    };

    LaneSelection {
        id: lane.id.clone(),
        workflow: lane.workflow.clone(),
        job: lane.job.clone(),
        kind: lane.kind.clone(),
        tier: lane.tier.clone(),
        runner: lane.runner.clone(),
        blocking: lane.blocking,
        estimated_lem: estimated,
        estimate_source,
        learned_p50_lem: p50,
        learned_p90_lem: p90,
        learned_p95_lem: p95,
        reason: reason.to_string(),
    }
}

fn percentile(samples: &[f64], p: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut sorted: Vec<f64> = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let rank = (p * (sorted.len() as f64 - 1.0)).round() as usize;
    sorted[rank.min(sorted.len() - 1)]
}

/// Walk a directory of past `ci-actuals.json` artifacts and collect per-job
/// `duration_seconds` samples keyed by job id. Files that fail to parse are
/// skipped — actuals are advisory.
fn load_actuals(dir: &Path) -> Result<BTreeMap<String, Vec<f64>>> {
    let mut by_job: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    if !dir.is_dir() {
        return Ok(by_job);
    }
    for entry in walkdir::WalkDir::new(dir).max_depth(3) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry
            .path()
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if !name.ends_with(".json") {
            continue;
        }
        let body = match fs::read_to_string(entry.path()) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&body) else {
            continue;
        };
        let Some(jobs) = value.get("jobs").and_then(|j| j.as_array()) else {
            continue;
        };
        for job in jobs {
            let (Some(name), Some(seconds)) = (
                job.get("name").and_then(|v| v.as_str()),
                actual_duration_seconds(job),
            ) else {
                continue;
            };
            if seconds <= 0.0 {
                continue;
            }
            by_job.entry(name.to_string()).or_default().push(seconds);
        }
    }
    Ok(by_job)
}

fn actual_duration_seconds(job: &serde_json::Value) -> Option<f64> {
    job.get("duration_seconds")
        .or_else(|| job.get("actual_seconds"))
        .and_then(|v| v.as_f64())
}

fn classify_band(lem: u64, budget: Budget) -> &'static str {
    if lem <= budget.default_limit_lem {
        "normal"
    } else if lem <= budget.elevated_limit_lem {
        "elevated"
    } else if lem <= budget.hard_limit_lem {
        "high-cost"
    } else {
        "override-required"
    }
}

fn render_step_summary(plan: &PlanOutput) -> String {
    let mut out = String::new();
    out.push_str("\n## PR Plan (advisory)\n\n");
    out.push_str(&format!("- estimated LEM: **{}**\n", plan.estimated_lem));
    out.push_str(&format!("- band: **{}**\n", plan.band));
    out.push_str(&format!(
        "- budget: preferred {}, default {}, elevated {}, hard {}\n",
        plan.budget.preferred_default_lem,
        plan.budget.default_limit_lem,
        plan.budget.elevated_limit_lem,
        plan.budget.hard_limit_lem,
    ));
    out.push_str(&format!("- changed files: {}\n", plan.changed_files.len()));
    out.push_str(&format!("- labels: {}\n\n", plan.labels.join(", ")));

    if plan.risk_packs_hit.is_empty() {
        out.push_str("No risk packs hit.\n\n");
    } else {
        out.push_str("### Risk packs hit\n\n");
        for hit in &plan.risk_packs_hit {
            out.push_str(&format!("- **{}** — {}\n", hit.name, hit.description));
        }
        out.push('\n');
    }

    out.push_str("### Lanes selected\n\n");
    out.push_str("| Lane | Tier | Runner | LEM | Reason |\n");
    out.push_str("|------|------|--------|----:|--------|\n");
    for lane in &plan.lanes_selected {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} | {} |\n",
            lane.id, lane.tier, lane.runner, lane.estimated_lem, lane.reason
        ));
    }
    out
}

fn render_github_outputs(plan: &PlanOutput) -> String {
    let hit = |name: &str| {
        plan.risk_packs_hit
            .iter()
            .any(|pack| pack.name.as_str() == name)
    };
    let changed = |path: &str| plan.changed_files.iter().any(|file| file == path);
    let glob_changed = |prefix: &str, suffix: &str| {
        plan.changed_files
            .iter()
            .any(|file| file.starts_with(prefix) && file.ends_with(suffix))
    };

    // Keep these names compatible with `.github/workflows/ci.yml` job outputs
    // while moving the path classification into the Rust-owned planner.
    let mut outputs = BTreeMap::new();
    outputs.insert("analysis", hit("analysis"));
    outputs.insert("core_receipts", hit("core_receipts"));
    outputs.insert("git_io", hit("git_io"));
    outputs.insert(
        "nix",
        changed("flake.nix") || changed("flake.lock") || changed("Dockerfile"),
    );
    outputs.insert(
        "release",
        hit("release") || glob_changed("crates/", "/Cargo.toml"),
    );
    outputs.insert("wasm", hit("wasm"));
    outputs.insert("windows_path", hit("git_io"));

    let mut out = String::new();
    for (key, value) in outputs {
        out.push_str(key);
        out.push('=');
        out.push_str(if value { "true" } else { "false" });
        out.push('\n');
    }
    out
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

    fn budget() -> Budget {
        Budget {
            preferred_default_lem: 25,
            default_limit_lem: 35,
            elevated_limit_lem: 75,
            hard_limit_lem: 125,
        }
    }

    fn test_lane(id: &str, blocking: bool, expensive: bool) -> Lane {
        Lane {
            id: id.into(),
            workflow: "ci.yml".into(),
            job: id.into(),
            kind: "test".into(),
            tier: "frontdoor".into(),
            default_pr: !expensive,
            blocking,
            runner: "ubuntu_latest".into(),
            base_lem: 1,
            expensive,
        }
    }

    fn route_test_whitelist() -> WhitelistFile {
        WhitelistFile {
            budget: Some(budget()),
            runner_multipliers: BTreeMap::new(),
            lane: vec![
                test_lane("docs_check", true, false),
                test_lane("rust_fast_gate", true, false),
                test_lane("rust_coverage", false, true),
                test_lane("build_test_windows", true, true),
            ],
        }
    }

    fn route_test_risk_packs() -> RiskPacksFile {
        RiskPacksFile {
            risk_pack: BTreeMap::from([
                (
                    "core".to_string(),
                    RiskPack {
                        description: "Core".into(),
                        paths: vec!["crates/tokmd/**".into()],
                        lanes: vec!["rust_fast_gate".into()],
                        deep_lanes: vec!["build_test_windows".into()],
                    },
                ),
                (
                    "docs".to_string(),
                    RiskPack {
                        description: "Docs".into(),
                        paths: vec!["docs/**".into(), "*.md".into()],
                        lanes: vec!["docs_check".into()],
                        deep_lanes: Vec::new(),
                    },
                ),
            ]),
        }
    }

    fn route_lane_index(whitelist: &WhitelistFile) -> BTreeMap<&str, &Lane> {
        whitelist
            .lane
            .iter()
            .map(|lane| (lane.id.as_str(), lane))
            .collect()
    }

    #[test]
    fn band_classification() {
        let b = budget();
        assert_eq!(classify_band(0, b), "normal");
        assert_eq!(classify_band(35, b), "normal");
        assert_eq!(classify_band(36, b), "elevated");
        assert_eq!(classify_band(75, b), "elevated");
        assert_eq!(classify_band(76, b), "high-cost");
        assert_eq!(classify_band(125, b), "high-cost");
        assert_eq!(classify_band(126, b), "override-required");
    }

    #[test]
    fn parse_labels_object_form() {
        let labels = parse_labels(Some(r#"[{"name":"wasm"},{"name":"full-ci"}]"#));
        assert_eq!(labels, vec!["wasm".to_string(), "full-ci".to_string()]);
    }

    #[test]
    fn parse_labels_bare_array() {
        let labels = parse_labels(Some(r#"["mutation","release-check"]"#));
        assert_eq!(
            labels,
            vec!["mutation".to_string(), "release-check".to_string()]
        );
    }

    #[test]
    fn parse_labels_csv() {
        let labels = parse_labels(Some("ripr-waive, full-ci"));
        assert_eq!(
            labels,
            vec!["ripr-waive".to_string(), "full-ci".to_string()]
        );
    }

    #[test]
    fn match_paths_basic() {
        let files = vec![
            "crates/tokmd/src/main.rs".to_string(),
            "docs/README.md".to_string(),
        ];
        let globs = vec!["crates/tokmd/**".to_string()];
        let matched = match_paths(&globs, &files).expect("globs valid");
        assert_eq!(matched, vec!["crates/tokmd/src/main.rs".to_string()]);
    }

    #[test]
    fn proof_pack_route_maps_files_and_unmatched_paths() {
        let whitelist = route_test_whitelist();
        let lane_index = route_lane_index(&whitelist);
        let risk_packs = route_test_risk_packs();
        let changed = vec![
            "crates/tokmd/src/main.rs".to_string(),
            "unowned/input.txt".to_string(),
        ];

        let route = route_changed_files(&changed, &risk_packs, &lane_index).expect("route");

        assert_eq!(route.changed_files.len(), 1);
        assert_eq!(route.changed_files[0].path, "crates/tokmd/src/main.rs");
        assert_eq!(route.changed_files[0].surface, "core");
        assert_eq!(route.changed_files[0].proof_packs, vec!["core".to_string()]);
        assert_eq!(route.changed_files[0].policy, "blocking");
        assert_eq!(route.unmatched_files, vec!["unowned/input.txt".to_string()]);
        assert_eq!(
            route.matched_by_pack["core"],
            vec!["crates/tokmd/src/main.rs".to_string()]
        );

        let selected_ids = BTreeSet::from(["rust_fast_gate".to_string()]);
        let skipped = skipped_by_policy(&whitelist, &selected_ids, &route);
        assert!(skipped.iter().any(|skip| {
            skip.lane == "build_test_windows"
                && skip.reason == "deep_lane_requires_label"
                && skip.matched_files == vec!["crates/tokmd/src/main.rs".to_string()]
        }));
    }

    #[test]
    fn skipped_policy_reports_docs_only_reason() {
        let whitelist = route_test_whitelist();
        let lane_index = route_lane_index(&whitelist);
        let risk_packs = route_test_risk_packs();
        let changed = vec!["docs/ci/pr-plan.md".to_string()];
        let route = route_changed_files(&changed, &risk_packs, &lane_index).expect("route");
        let selected_ids = BTreeSet::from(["docs_check".to_string()]);

        let skipped = skipped_by_policy(&whitelist, &selected_ids, &route);

        assert!(skipped.iter().any(|skip| {
            skip.lane == "rust_coverage"
                && skip.status == "skipped_by_policy"
                && skip.reason == "docs_only_change"
                && skip.matched_files == vec!["docs/ci/pr-plan.md".to_string()]
        }));
    }

    #[test]
    fn lane_to_selection_uses_runner_multiplier() {
        let lane = Lane {
            id: "x".into(),
            workflow: "w.yml".into(),
            job: "X".into(),
            kind: "rust".into(),
            tier: "frontdoor".into(),
            default_pr: true,
            blocking: true,
            runner: "windows_latest".into(),
            base_lem: 10,
            expensive: false,
        };
        let mut multipliers = BTreeMap::new();
        multipliers.insert("windows_latest".into(), 2.0);
        let actuals = BTreeMap::new();
        let sel = lane_to_selection(&lane, &multipliers, &actuals, "test");
        assert_eq!(sel.estimated_lem, 20);
        assert_eq!(sel.estimate_source, "static");
    }

    #[test]
    fn percentile_basic_quantiles() {
        let samples = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];
        assert_eq!(percentile(&samples, 0.50), 60.0);
        assert!((percentile(&samples, 0.90) - 90.0).abs() < 1e-9);
    }

    #[test]
    fn lane_to_selection_uses_actuals_when_higher() {
        let lane = Lane {
            id: "x".into(),
            workflow: "w.yml".into(),
            job: "X".into(),
            kind: "rust".into(),
            tier: "frontdoor".into(),
            default_pr: true,
            blocking: true,
            runner: "ubuntu_latest".into(),
            base_lem: 5,
            expensive: false,
        };
        let multipliers = BTreeMap::new();
        // p50 = 600s = 10 LEM × 1.15 = 11.5 → 12, beats static 5.
        let mut actuals: BTreeMap<String, Vec<f64>> = BTreeMap::new();
        actuals.insert("x".into(), vec![300.0, 400.0, 600.0, 700.0, 900.0]);
        let sel = lane_to_selection(&lane, &multipliers, &actuals, "test");
        assert!(sel.estimated_lem >= 11);
        assert_eq!(sel.estimate_source, "learned-p50");
        assert!(sel.learned_p50_lem.is_some());
    }

    #[test]
    fn load_actuals_consumes_ci_actuals_duration_seconds() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("ci-actuals.json");
        fs::write(
            &path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema": "tokmd.ci_actuals.v1",
                "jobs": [
                    {"name": "build_test_linux", "duration_seconds": 600.0},
                    {"name": "legacy_lane", "actual_seconds": 120.0},
                    {"name": "zero_lane", "duration_seconds": 0.0},
                    {"name": "missing_lane"}
                ]
            }))
            .expect("serialize fixture"),
        )
        .expect("write fixture");

        let actuals = load_actuals(temp.path()).expect("load actuals");

        assert_eq!(actuals["build_test_linux"], vec![600.0]);
        assert_eq!(actuals["legacy_lane"], vec![120.0]);
        assert!(
            !actuals.contains_key("zero_lane"),
            "zero-duration samples should not seed learned estimates"
        );
        assert!(
            !actuals.contains_key("missing_lane"),
            "missing-duration jobs should not seed learned estimates"
        );
    }

    #[test]
    fn github_outputs_include_ci_compatibility_flags() {
        let plan = PlanOutput {
            schema_version: 1,
            base: "origin/main".into(),
            head: "HEAD".into(),
            labels: Vec::new(),
            changed_files: vec![
                "crates/tokmd-git/src/lib.rs".into(),
                "crates/tokmd-demo/Cargo.toml".into(),
                "flake.nix".into(),
            ],
            risk_packs_hit: vec![
                RiskPackHit {
                    name: "git_io".into(),
                    description: "Git and IO".into(),
                    matched_files: vec!["crates/tokmd-git/src/lib.rs".into()],
                },
                RiskPackHit {
                    name: "release".into(),
                    description: "Release".into(),
                    matched_files: vec!["flake.nix".into()],
                },
            ],
            lanes_selected: Vec::new(),
            estimated_lem: 0,
            band: "normal".into(),
            budget: BudgetView {
                preferred_default_lem: 25,
                default_limit_lem: 35,
                elevated_limit_lem: 75,
                hard_limit_lem: 125,
            },
        };

        let outputs = render_github_outputs(&plan);

        assert!(outputs.contains("analysis=false\n"), "{outputs}");
        assert!(outputs.contains("core_receipts=false\n"), "{outputs}");
        assert!(outputs.contains("git_io=true\n"), "{outputs}");
        assert!(outputs.contains("nix=true\n"), "{outputs}");
        assert!(outputs.contains("release=true\n"), "{outputs}");
        assert!(outputs.contains("wasm=false\n"), "{outputs}");
        assert!(outputs.contains("windows_path=true\n"), "{outputs}");
    }

    #[test]
    fn budget_annotation_messages_preserve_override_required_error() {
        let plan = plan_for_budget("override-required", 150, Vec::new());

        let messages = budget_annotation_messages(&plan);

        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("::error::PR plan estimated 150 LEM"));
        assert!(budget_requires_override(&plan));
    }

    #[test]
    fn budget_annotation_messages_high_cost_asks_for_ack_not_override() {
        let plan = plan_for_budget("high-cost", 113, Vec::new());

        let messages = budget_annotation_messages(&plan);

        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("::warning::PR plan estimated 113 LEM"));
        assert!(messages[0].contains("ci-budget-ack"));
        assert!(!messages[0].contains("ci-budget-override"));
        assert!(!budget_requires_override(&plan));
    }

    #[test]
    fn budget_annotation_messages_high_cost_ack_suppresses_warning() {
        let plan = plan_for_budget("high-cost", 113, vec!["ci-budget-ack".to_string()]);

        let messages = budget_annotation_messages(&plan);

        assert!(messages.is_empty(), "{messages:?}");
        assert!(!budget_requires_override(&plan));
    }

    #[test]
    fn budget_annotation_messages_ack_override_label() {
        let plan = plan_for_budget(
            "override-required",
            150,
            vec!["ci-budget-override".to_string()],
        );

        let messages = budget_annotation_messages(&plan);

        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("proceeding because override label is present"));
        assert!(!budget_requires_override(&plan));
    }

    #[test]
    fn budget_annotation_messages_allow_silent_detector_mode() {
        let plan = plan_for_budget("override-required", 150, Vec::new());

        let emitted = maybe_budget_annotation_messages(&plan, false);

        assert!(emitted.is_empty());
        assert!(budget_requires_override(&plan));
    }

    fn plan_for_budget(band: &str, estimated_lem: u64, labels: Vec<String>) -> PlanOutput {
        PlanOutput {
            schema_version: 1,
            base: "origin/main".into(),
            head: "HEAD".into(),
            labels,
            changed_files: Vec::new(),
            risk_packs_hit: Vec::new(),
            lanes_selected: Vec::new(),
            estimated_lem,
            band: band.into(),
            budget: BudgetView {
                preferred_default_lem: 25,
                default_limit_lem: 35,
                elevated_limit_lem: 75,
                hard_limit_lem: 125,
            },
        }
    }
}
