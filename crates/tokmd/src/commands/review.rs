use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Result, bail};
use serde::Serialize;

use crate::cli;

#[derive(Debug, Clone, Serialize)]
struct ReviewOverall {
    risk_level: String,
    risk_score: u32,
    health_score: u32,
    health_grade: String,
    priority_items: usize,
    complexity_findings: usize,
    duplication_findings: usize,
    evidence_gaps: usize,
}

#[derive(Debug, Clone, Serialize)]
struct ReviewPathItem {
    priority: u32,
    path: String,
    score: u32,
    reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ReviewMap {
    schema_version: String,
    base_ref: String,
    head_ref: String,
    overall: ReviewOverall,
    review_path: Vec<ReviewPathItem>,
    evidence_gaps: Vec<String>,
}

pub(crate) fn handle(args: cli::ReviewArgs, _global: &cli::GlobalArgs) -> Result<()> {
    #[cfg(not(feature = "git"))]
    {
        let _ = &args;
        bail!("The review command requires the 'git' feature. Rebuild with --features git");
    }

    #[cfg(feature = "git")]
    {
        if !tokmd_git::git_available() {
            bail!("git is not available on PATH");
        }
        let cwd = std::env::current_dir()?;
        let repo_root = tokmd_git::repo_root(&cwd)
            .ok_or_else(|| anyhow::anyhow!("not inside a git repository"))?;
        let resolved_base = tokmd_git::resolve_base_ref(&repo_root, &args.base)
            .ok_or_else(|| anyhow::anyhow!("unable to resolve base ref '{}", args.base))?;

        let cockpit = tokmd_cockpit::compute_cockpit(
            &repo_root,
            &resolved_base,
            &args.head,
            tokmd_git::GitRangeMode::TwoDot,
            None,
        )?;

        fs::create_dir_all(&args.out_dir)?;
        write_json(args.out_dir.join("cockpit.json"), &cockpit)?;
        write_json(
            args.out_dir.join("complexity.json"),
            &cockpit.evidence.complexity,
        )?;
        write_json(args.out_dir.join("evidence.json"), &cockpit.evidence)?;

        let mut dup = BTreeMap::new();
        dup.insert("enabled", args.near_dup);
        dup.insert("detail_functions", args.detail_functions);
        write_json(args.out_dir.join("duplication.json"), &dup)?;

        let evidence_gaps = collect_evidence_gaps(&cockpit);
        let complexity_findings = cockpit
            .evidence
            .complexity
            .as_ref()
            .map(|c| c.high_complexity_files.len())
            .unwrap_or(0);

        let mut review_path: Vec<ReviewPathItem> = cockpit
            .review_plan
            .iter()
            .map(|item| ReviewPathItem {
                priority: item.priority,
                path: item.path.clone(),
                score: score_item(item),
                reasons: vec![item.reason.clone()],
            })
            .collect();
        review_path.sort_by_key(|x| (x.priority, std::cmp::Reverse(x.score)));
        review_path.truncate(args.max_review_items);

        let review_map = ReviewMap {
            schema_version: "tokmd.review_map.v1".to_string(),
            base_ref: resolved_base.clone(),
            head_ref: args.head.clone(),
            overall: ReviewOverall {
                risk_level: cockpit.risk.level.to_string(),
                risk_score: cockpit.risk.score,
                health_score: cockpit.code_health.score,
                health_grade: cockpit.code_health.grade.clone(),
                priority_items: review_path.iter().filter(|i| i.priority == 1).count(),
                complexity_findings,
                duplication_findings: 0,
                evidence_gaps: evidence_gaps.len(),
            },
            review_path,
            evidence_gaps,
        };

        write_json(args.out_dir.join("review-map.json"), &review_map)?;
        fs::write(
            args.out_dir.join("review-map.md"),
            render_review_map_md(&review_map),
        )?;
        fs::write(
            args.out_dir.join("comment.md"),
            render_comment_md(&review_map),
        )?;

        let manifest = serde_json::json!({
            "mode": "review",
            "base_ref": review_map.base_ref,
            "head_ref": review_map.head_ref,
            "artifacts": {
                "comment": "comment.md",
                "review_map": "review-map.json",
                "cockpit": "cockpit.json",
                "complexity": "complexity.json",
                "duplication": "duplication.json",
                "evidence": "evidence.json"
            },
            "gate": format!("{:?}", args.gate).to_lowercase(),
        });
        write_json(args.out_dir.join("manifest.json"), &manifest)?;

        if matches!(args.gate, cli::ReviewGateMode::Blocking) && review_map.overall.risk_score >= 90
        {
            bail!(
                "blocking review gate failed: critical risk score {}",
                review_map.overall.risk_score
            );
        }

        print!("{}", render_comment_md(&review_map));
        Ok(())
    }
}

fn write_json(path: impl AsRef<Path>, value: &impl Serialize) -> Result<()> {
    let content = serde_json::to_string_pretty(value)?;
    fs::write(path, content)?;
    Ok(())
}

fn collect_evidence_gaps(cockpit: &tokmd_types::cockpit::CockpitReceipt) -> Vec<String> {
    let mut gaps = Vec::new();
    if cockpit.evidence.mutation.meta.status != tokmd_types::cockpit::GateStatus::Pass {
        gaps.push("Mutation evidence is missing, stale, or failing.".to_string());
    }
    if let Some(dc) = &cockpit.evidence.diff_coverage {
        if dc.meta.status != tokmd_types::cockpit::GateStatus::Pass {
            gaps.push("Diff coverage evidence is missing, stale, or failing.".to_string());
        }
    }
    gaps
}

fn score_item(item: &tokmd_types::cockpit::ReviewItem) -> u32 {
    let mut score = 20 + (4_u32.saturating_sub(item.priority) * 15);
    if let Some(lines) = item.lines_changed {
        score += (lines as u32 / 20).min(20);
    }
    if let Some(c) = item.complexity {
        score += (c as u32).min(20);
    }
    score.min(100)
}

fn render_comment_md(map: &ReviewMap) -> String {
    format!(
        "## tokmd Review Cockpit\n\nRisk: {} ({})\nHealth: {}/100 ({})\nComplexity: {} high-complexity files touched\nDuplication: {} near-duplicate findings\nEvidence gaps: {}\n",
        map.overall.risk_level,
        map.overall.risk_score,
        map.overall.health_score,
        map.overall.health_grade,
        map.overall.complexity_findings,
        map.overall.duplication_findings,
        map.overall.evidence_gaps
    )
}

fn render_review_map_md(map: &ReviewMap) -> String {
    let mut out = String::from("## Review path\n\n| Priority | File | Why |\n|---:|---|---|\n");
    for item in &map.review_path {
        let why = item.reasons.join("; ");
        out.push_str(&format!(
            "| P{} | {} | {} |\n",
            item.priority, item.path, why
        ));
    }
    out
}
