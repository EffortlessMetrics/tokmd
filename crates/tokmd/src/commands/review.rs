use crate::analysis_utils;
use crate::cli;
use crate::export_bundle;
use anyhow::{Context, Result, bail};
use serde_json::json;

pub(crate) fn handle(args: cli::ReviewArgs, global: &cli::GlobalArgs) -> Result<()> {
    #[cfg(not(feature = "git"))]
    {
        let _ = (&args, global);
        bail!("The review command requires the 'git' feature. Rebuild with --features git");
    }

    #[cfg(feature = "git")]
    {
        let _ = &args.config;
        if !tokmd_git::git_available() {
            bail!("git is not available on PATH");
        }
        let cwd = std::env::current_dir().context("Failed to resolve current directory")?;
        let repo_root = tokmd_git::repo_root(&cwd)
            .ok_or_else(|| anyhow::anyhow!("not inside a git repository"))?;
        let base = tokmd_git::resolve_base_ref(&repo_root, &args.base).unwrap_or(args.base.clone());

        let cockpit = tokmd_cockpit::compute_cockpit(
            &repo_root,
            &base,
            &args.head,
            tokmd_git::GitRangeMode::TwoDot,
            None,
        )?;

        let bundle = export_bundle::load_export_from_inputs(&[repo_root.clone()], global)?;
        let request = tokmd_analysis::AnalysisRequest {
            preset: analysis_utils::map_preset(args.analysis_preset),
            args: tokmd_analysis_types::AnalysisArgsMeta {
                preset: analysis_utils::preset_to_string(args.analysis_preset),
                format: "json".to_string(),
                window_tokens: None,
                git: Some(true),
                max_files: None,
                max_bytes: None,
                max_file_bytes: None,
                max_commits: None,
                max_commit_files: None,
                import_granularity: "module".to_string(),
            },
            limits: tokmd_analysis::AnalysisLimits::default(),
            window_tokens: None,
            git: Some(true),
            import_granularity: tokmd_analysis::ImportGranularity::Module,
            detail_functions: args.detail_functions,
            near_dup: args.near_dup,
            near_dup_threshold: 0.80,
            near_dup_max_files: 2000,
            near_dup_scope: tokmd_analysis::NearDupScope::Module,
            near_dup_max_pairs: Some(10000),
            near_dup_exclude: vec![],
            effort: None,
        };
        let analysis = tokmd_analysis::analyze(
            tokmd_analysis::AnalysisContext {
                export: bundle.export,
                root: bundle.root,
                source: tokmd_analysis_types::AnalysisSource {
                    inputs: vec![repo_root.display().to_string()],
                    export_path: None,
                    base_receipt_path: None,
                    export_schema_version: None,
                    export_generated_at_ms: None,
                    base_signature: None,
                    module_roots: vec!["crates".to_string(), "packages".to_string()],
                    module_depth: 2,
                    children: "separate".to_string(),
                },
            },
            request,
        )?;

        std::fs::create_dir_all(&args.out_dir).context("Failed to create review out-dir")?;

        let mut review_rows: Vec<_> = cockpit
            .review_plan
            .iter()
            .map(|item| {
                let score = (item.priority.max(1) as i64 * -20)
                    + item.lines_changed.unwrap_or_default() as i64;
                (score, item)
            })
            .collect();
        review_rows.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.path.cmp(&b.1.path)));

        let review_path: Vec<_> = review_rows
            .into_iter()
            .take(args.max_review_items)
            .map(|(score, item)| {
                json!({
                    "priority": item.priority,
                    "path": item.path,
                    "score": score,
                    "reasons": [item.reason.clone()],
                    "complexity": item.complexity,
                })
            })
            .collect();

        let review_map = json!({
            "schema_version": "tokmd.review_map.v1",
            "base_ref": base,
            "head_ref": args.head,
            "overall": {
                "risk_level": cockpit.risk.level.to_string(),
                "risk_score": cockpit.risk.score,
                "health_score": cockpit.code_health.score,
                "health_grade": cockpit.code_health.grade,
                "priority_items": review_path.len(),
                "complexity_findings": analysis.complexity.as_ref().map(|c| c.files.len()).unwrap_or(0),
                "duplication_findings": analysis.dup.as_ref().and_then(|d| d.near.as_ref()).map(|n| n.pairs.len()).unwrap_or(0),
                "evidence_gaps": if matches!(cockpit.evidence.overall_status, tokmd_types::cockpit::GateStatus::Pass) { 0 } else { 1 },
            },
            "review_path": review_path,
        });

        let evidence_json = serde_json::to_value(&cockpit.evidence)?;
        let manifest = json!({
            "mode":"review",
            "gate_mode": format!("{:?}", args.gate).to_lowercase(),
            "artifacts": {
                "comment":"comment.md",
                "review_map":"review-map.json",
                "cockpit":"cockpit.json",
                "analysis":"analysis-risk.json",
                "evidence":"evidence.json"
            }
        });

        std::fs::write(
            args.out_dir.join("cockpit.json"),
            serde_json::to_vec_pretty(&cockpit)?,
        )?;
        std::fs::write(
            args.out_dir.join("analysis-risk.json"),
            serde_json::to_vec_pretty(&analysis)?,
        )?;
        std::fs::write(
            args.out_dir.join("review-map.json"),
            serde_json::to_vec_pretty(&review_map)?,
        )?;
        std::fs::write(
            args.out_dir.join("evidence.json"),
            serde_json::to_vec_pretty(&evidence_json)?,
        )?;
        std::fs::write(
            args.out_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest)?,
        )?;

        let comment = tokmd_cockpit::render::render_markdown(&cockpit);
        let review_md = render_review_map_md(&review_map);
        std::fs::write(args.out_dir.join("comment.md"), comment)?;
        std::fs::write(args.out_dir.join("review-map.md"), review_md)?;

        if matches!(args.gate, cli::ReviewGateMode::Blocking)
            && matches!(
                cockpit.evidence.overall_status,
                tokmd_types::cockpit::GateStatus::Fail
            )
        {
            bail!("blocking gate failed: required evidence gates are not satisfied");
        }
        Ok(())
    }
}

fn render_review_map_md(review_map: &serde_json::Value) -> String {
    let mut out = String::from("## tokmd Review Map\n\n");
    if let Some(overall) = review_map.get("overall") {
        out.push_str(&format!(
            "Risk: {} ({})\nHealth: {} ({})\n\n",
            overall
                .get("risk_level")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown"),
            overall
                .get("risk_score")
                .and_then(|v| v.as_f64())
                .unwrap_or_default(),
            overall
                .get("health_score")
                .and_then(|v| v.as_f64())
                .unwrap_or_default(),
            overall
                .get("health_grade")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
        ));
    }
    out.push_str("### Review path\n\n| Priority | File | Why |\n|---:|---|---|\n");
    if let Some(rows) = review_map.get("review_path").and_then(|v| v.as_array()) {
        for row in rows {
            let p = row.get("priority").and_then(|v| v.as_u64()).unwrap_or(3);
            let path = row.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let why = row
                .get("reasons")
                .and_then(|v| v.as_array())
                .map(|r| {
                    r.iter()
                        .filter_map(|x| x.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            out.push_str(&format!("| P{} | {} | {} |\n", p, path, why));
        }
    }
    out
}
