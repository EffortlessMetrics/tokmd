//! LEM-aware advisory PR Plan.
//!
//! Inputs:
//! - `policy/ci-lane-whitelist.toml` — lane catalogue + LEM budget.
//! - `policy/ci-risk-packs.toml` — path → lane routing.
//! - the PR diff `base..head` — selects risk packs.
//!
//! Output: `ci-plan.json` describing which risk packs were hit, which
//! lanes will run on this PR, the estimated LEM, and the LEM band.
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
        let matched = match_paths(&pack.paths, &changed)?;
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

    // Soft budget guard. Always reports advisory diagnostics. Only fails
    // when --enforce is set, and then only on the over-hard-limit case
    // unless an override label is present.
    let labels_set: BTreeSet<&str> = plan.labels.iter().map(|s| s.as_str()).collect();
    let override_present =
        labels_set.contains("full-ci") || labels_set.contains("ci-budget-override");
    let ack_present = labels_set.contains("ci-budget-ack");

    match plan.band.as_str() {
        "elevated" if !ack_present => {
            println!(
                "::warning::PR plan estimated {} LEM (elevated band; expected ≤ {}). \
                Apply `ci-budget-ack` to acknowledge.",
                plan.estimated_lem, plan.budget.default_limit_lem
            );
        }
        "high-cost" if !override_present => {
            println!(
                "::warning::PR plan estimated {} LEM (high-cost band; expected ≤ {}). \
                Apply `ci-budget-override` or `full-ci` to bypass.",
                plan.estimated_lem, plan.budget.elevated_limit_lem
            );
        }
        "override-required" => {
            if override_present {
                println!(
                    "::warning::PR plan estimated {} LEM (>{} hard ceiling) — \
                    proceeding because override label is present.",
                    plan.estimated_lem, plan.budget.hard_limit_lem
                );
            } else {
                println!(
                    "::error::PR plan estimated {} LEM (>{} hard ceiling) — \
                    apply `ci-budget-override` or `full-ci` to bypass, or split the PR.",
                    plan.estimated_lem, plan.budget.hard_limit_lem
                );
                if args.enforce {
                    bail!(
                        "ci-plan: estimated {} LEM exceeds hard ceiling {}",
                        plan.estimated_lem,
                        plan.budget.hard_limit_lem
                    );
                }
            }
        }
        _ => {}
    }

    Ok(())
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
/// `actual_seconds` samples keyed by job id. Files that fail to parse are
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
                job.get("actual_seconds").and_then(|v| v.as_f64()),
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
}
