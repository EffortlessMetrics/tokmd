use crate::cli;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub(crate) fn handle(args: cli::ReviewArgs, _global: &cli::GlobalArgs) -> Result<()> {
    let out_dir = args
        .out_dir.clone()
        .unwrap_or_else(|| PathBuf::from(".tokmd/review"));
    fs::create_dir_all(&out_dir)?;

    let cockpit_path = out_dir.join("cockpit.json");
    run_tokmd(
        &[
            "cockpit", "--base", &args.base, "--head", &args.head, "--format", "json",
        ],
        Some(&cockpit_path),
    )?;

    let analysis_path = out_dir.join("analysis-risk.json");
    let mut analyze_args = vec!["analyze", ".", "--preset", "risk", "--format", "json"];
    if args.near_dup {
        analyze_args.push("--near-dup");
    }
    if args.detail_functions {
        analyze_args.push("--detail-functions");
    }
    run_tokmd(&analyze_args, Some(&analysis_path))?;

    let cockpit: Value = serde_json::from_str(&fs::read_to_string(&cockpit_path)?)?;
    let analysis: Value = serde_json::from_str(&fs::read_to_string(&analysis_path)?)?;

    let review_map = build_review_map(&cockpit, &analysis, &args);
    fs::write(
        out_dir.join("review-map.json"),
        serde_json::to_string_pretty(&review_map)?,
    )?;
    fs::write(
        out_dir.join("review-map.md"),
        render_review_map_md(&review_map),
    )?;

    let evidence = cockpit
        .get("evidence")
        .cloned()
        .unwrap_or_else(|| json!({}));
    fs::write(
        out_dir.join("evidence.json"),
        serde_json::to_string_pretty(&evidence)?,
    )?;

    let comment = render_comment(&cockpit, &review_map);
    fs::write(out_dir.join("comment.md"), comment)?;

    let manifest = json!({
        "base": args.base,
        "head": args.head,
        "artifacts": {
            "cockpit": "cockpit.json",
            "analysis": "analysis-risk.json",
            "review_map": "review-map.json",
            "review_map_md": "review-map.md",
            "comment": "comment.md",
            "evidence": "evidence.json"
        }
    });
    fs::write(
        out_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;

    if args.gate != cli::ReviewGateMode::Off {
        let packet_path = out_dir.join("review-packet.json");
        let packet = json!({"review_map": review_map, "cockpit": cockpit, "analysis": analysis});
        fs::write(&packet_path, serde_json::to_string_pretty(&packet)?)?;
        let gate_out = out_dir.join("gate-verdict.json");
        let status = run_tokmd(
            &["gate", packet_path.to_str().unwrap(), "--format", "json"],
            Some(&gate_out),
        );
        if args.gate == cli::ReviewGateMode::Blocking {
            status?;
        }
    }

    Ok(())
}

fn run_tokmd(args: &[&str], output: Option<&PathBuf>) -> Result<()> {
    let exe = std::env::current_exe().context("resolve current tokmd binary")?;
    let mut cmd = Command::new(exe);
    cmd.args(args);
    if let Some(path) = output {
        let out = cmd.output().context("failed to run tokmd subcommand")?;
        if !out.status.success() {
            bail!("tokmd {:?} failed", args);
        }
        fs::write(path, out.stdout)?;
    } else {
        let status = cmd.status()?;
        if !status.success() {
            bail!("tokmd {:?} failed", args);
        }
    }
    Ok(())
}

fn build_review_map(cockpit: &Value, analysis: &Value, args: &cli::ReviewArgs) -> Value {
    let review_plan = cockpit
        .get("review_plan")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut path = Vec::new();
    for (idx, item) in review_plan
        .into_iter()
        .take(args.max_review_items)
        .enumerate()
    {
        let p = item
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let mut reasons = Vec::new();
        if let Some(r) = item.get("reason").and_then(Value::as_str) {
            reasons.push(r.to_string());
        }
        path.push(
            json!({"priority": idx+1, "path": p, "score": 100-(idx as i32*10), "reasons": reasons}),
        );
    }
    let dup_findings = analysis
        .pointer("/dup/pairs")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);
    let cx_findings = analysis
        .pointer("/complexity/high_risk_files")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);
    let evidence_gaps = cockpit
        .pointer("/evidence/mutation/survivors")
        .and_then(Value::as_array)
        .map(|a| if a.is_empty() { 0 } else { 1 })
        .unwrap_or(0);
    json!({
      "schema_version":"tokmd.review_map.v1",
      "base_ref": args.base,
      "head_ref": args.head,
      "overall": {
        "risk_level": cockpit.pointer("/risk/level").and_then(Value::as_str).unwrap_or("unknown"),
        "risk_score": cockpit.pointer("/risk/score").and_then(Value::as_u64).unwrap_or(0),
        "health_score": cockpit.pointer("/code_health/score").and_then(Value::as_u64).unwrap_or(0),
        "health_grade": cockpit.pointer("/code_health/grade").and_then(Value::as_str).unwrap_or("N/A"),
        "priority_items": path.len(),
        "complexity_findings": cx_findings,
        "duplication_findings": dup_findings,
        "evidence_gaps": evidence_gaps
      },
      "review_path": path
    })
}

fn render_review_map_md(review_map: &Value) -> String {
    let mut out = String::from("## Review map\n\n| Priority | File | Why |\n|---:|---|---|\n");
    if let Some(items) = review_map.get("review_path").and_then(Value::as_array) {
        for item in items {
            let reasons = item
                .get("reasons")
                .and_then(Value::as_array)
                .map(|r| {
                    r.iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            out.push_str(&format!(
                "| P{} | {} | {} |\n",
                item.get("priority").and_then(Value::as_u64).unwrap_or(0),
                item.get("path")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                reasons
            ));
        }
    }
    out
}

fn render_comment(_cockpit: &Value, review_map: &Value) -> String {
    format!(
        "## tokmd Review Cockpit\n\nRisk: {}\nHealth: {} ({})\nComplexity: {} high-complexity files touched\nDuplication: {} near-duplicate pairs\n\n{}",
        review_map
            .pointer("/overall/risk_level")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        review_map
            .pointer("/overall/health_score")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        review_map
            .pointer("/overall/health_grade")
            .and_then(Value::as_str)
            .unwrap_or("N/A"),
        review_map
            .pointer("/overall/complexity_findings")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        review_map
            .pointer("/overall/duplication_findings")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        render_review_map_md(review_map)
    )
}
