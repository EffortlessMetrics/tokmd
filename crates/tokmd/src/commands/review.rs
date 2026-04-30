use crate::cli;
use anyhow::{Result, bail};
use serde_json::json;

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
            .ok_or_else(|| anyhow::anyhow!("base ref '{}' not found", args.base))?;

        let receipt = tokmd_cockpit::compute_cockpit(
            &repo_root,
            &resolved_base,
            &args.head,
            tokmd_git::GitRangeMode::TwoDot,
            None,
        )?;

        std::fs::create_dir_all(&args.out_dir)?;
        let cockpit_json = serde_json::to_string_pretty(&receipt)?;
        std::fs::write(args.out_dir.join("cockpit.json"), cockpit_json)?;

        let max_items = args.max_review_items.unwrap_or(12);
        let review_items = receipt
            .review_plan
            .iter()
            .take(max_items)
            .map(|it| {
                json!({"priority": it.priority, "path": it.path, "reason": it.reason, "complexity": it.complexity, "lines_changed": it.lines_changed})
            })
            .collect::<Vec<_>>();

        let review_map = json!({
            "schema_version": "tokmd.review_map.v1",
            "base_ref": resolved_base,
            "head_ref": args.head,
            "overall": {
                "risk_level": receipt.risk.level.to_string(),
                "risk_score": receipt.risk.score,
                "health_score": receipt.code_health.score,
                "health_grade": receipt.code_health.grade,
                "priority_items": review_items.len(),
                "complexity_findings": receipt.evidence.complexity.as_ref().map(|c| c.high_complexity_files.len()).unwrap_or(0),
                "duplication_findings": 0,
                "evidence_gaps": (receipt.evidence.overall_status != tokmd_types::cockpit::GateStatus::Pass) as usize,
            },
            "review_path": review_items,
        });

        std::fs::write(
            args.out_dir.join("review-map.json"),
            serde_json::to_string_pretty(&review_map)?,
        )?;

        let md = tokmd_cockpit::render::render_markdown(&receipt);
        std::fs::write(args.out_dir.join("comment.md"), md)?;
        std::fs::write(
            args.out_dir.join("review-map.md"),
            "# Review Map\n\nSee `review-map.json` for machine-readable details.\n",
        )?;
        std::fs::write(args.out_dir.join("analysis.json"), "{}\n")?;
        std::fs::write(
            args.out_dir.join("evidence.json"),
            serde_json::to_string_pretty(&receipt.evidence)?,
        )?;
        let manifest = json!({
            "mode": "review",
            "out_dir": args.out_dir,
            "files": ["comment.md","review-map.md","review-map.json","cockpit.json","analysis.json","evidence.json"],
            "gate": format!("{:?}", args.gate).to_lowercase(),
            "detail_functions": args.detail_functions,
            "near_dup": args.near_dup,
            "config": args.config,
        });
        std::fs::write(
            args.out_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest)?,
        )?;

        if matches!(args.gate, cli::ReviewGateMode::Blocking)
            && receipt.evidence.overall_status == tokmd_types::cockpit::GateStatus::Fail
        {
            bail!("review gate blocking mode failed due to cockpit evidence status=fail");
        }
        Ok(())
    }
}
