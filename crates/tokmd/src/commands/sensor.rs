//! Handler for the `tokmd sensor` command.
//!
//! Runs tokmd as a conforming sensor, producing a `SensorReport` envelope
//! backed by cockpit computation. Implements a 3-layer output topology:
//!
//! 1. **report.json** — Thin envelope with findings, gates, summary metrics
//! 2. **extras/cockpit_receipt.json** — Full cockpit receipt sidecar
//! 3. **comment.md** — Markdown summary for PR comments

#[cfg(feature = "git")]
use std::io::Write;

#[cfg(feature = "git")]
use anyhow::Context;
use anyhow::{Result, bail};
use tokmd_config as cli;

#[cfg(feature = "git")]
use tokmd_envelope::findings;
#[cfg(feature = "git")]
use tokmd_envelope::{
    Artifact, Finding, FindingSeverity, GateItem, GateResults, SensorReport, ToolMeta, Verdict,
};

/// Maximum findings emitted per category to avoid spamming the bus.
#[cfg(feature = "git")]
const MAX_FINDINGS_PER_CATEGORY: usize = 10;

pub(crate) fn handle(args: cli::SensorArgs, global: &cli::GlobalArgs) -> Result<()> {
    #[cfg(not(feature = "git"))]
    {
        let _ = (&args, global);
        bail!("The sensor command requires the 'git' feature. Rebuild with --features git");
    }

    #[cfg(feature = "git")]
    {
        let _ = global; // scan opts not needed for cockpit path

        if !tokmd_git::git_available() {
            bail!("git is not available on PATH");
        }

        let cwd = std::env::current_dir().context("Failed to resolve current directory")?;
        let repo_root = tokmd_git::repo_root(&cwd)
            .ok_or_else(|| anyhow::anyhow!("not inside a git repository"))?;

        // Use two-dot range for sensor (same convention as cockpit)
        let range_mode = tokmd_git::GitRangeMode::TwoDot;

        // Run cockpit computation (sensor mode has no baseline path)
        let cockpit_receipt =
            super::cockpit::compute_cockpit(&repo_root, &args.base, &args.head, range_mode, None)?;

        // Build the sensor report envelope
        let generated_at = now_iso8601();
        let verdict = map_verdict(cockpit_receipt.evidence.overall_status);

        let mut report = SensorReport::new(
            ToolMeta::tokmd(env!("CARGO_PKG_VERSION"), "sensor"),
            generated_at,
            verdict,
            build_summary(&cockpit_receipt, &args.base, &args.head),
        );

        // Emit findings from cockpit data (all with fingerprints)
        emit_risk_findings(&mut report, &cockpit_receipt.risk);
        emit_contract_findings(&mut report, &cockpit_receipt.contracts);
        emit_complexity_findings(&mut report, &cockpit_receipt.evidence);
        emit_gate_findings(&mut report, &cockpit_receipt.evidence);

        // --- 3-layer output topology ---
        let output_path = &args.output;
        let artifact_dir = output_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let extras_dir = artifact_dir.join("extras");
        let comment_path = artifact_dir.join("comment.md");

        // Ensure directories exist
        if !artifact_dir.as_os_str().is_empty() {
            std::fs::create_dir_all(artifact_dir)?;
        }
        std::fs::create_dir_all(&extras_dir)?;

        // Write extras/cockpit_receipt.json (full sidecar)
        let cockpit_sidecar_path = extras_dir.join("cockpit_receipt.json");
        let cockpit_json_str = serde_json::to_string_pretty(&cockpit_receipt)?;
        std::fs::write(&cockpit_sidecar_path, cockpit_json_str.as_bytes())?;

        // Slim data: only gates + summary metrics (no embedded cockpit receipt)
        let gates = map_gates(&cockpit_receipt.evidence);
        let data = serde_json::json!({
            "gates": serde_json::to_value(&gates)?,
            "summary_metrics": {
                "files_changed": cockpit_receipt.change_surface.files_changed,
                "insertions": cockpit_receipt.change_surface.insertions,
                "deletions": cockpit_receipt.change_surface.deletions,
                "health_score": cockpit_receipt.code_health.score,
                "risk_level": cockpit_receipt.risk.level.to_string(),
                "risk_score": cockpit_receipt.risk.score,
            },
        });
        report = report.with_data(data);

        // Build enriched artifacts array with id/mime
        let path_str = |p: &std::path::Path| p.display().to_string().replace('\\', "/");
        report = report.with_artifacts(vec![
            Artifact::receipt(path_str(output_path))
                .with_id("receipt")
                .with_mime("application/json"),
            Artifact::new("evidence", path_str(&cockpit_sidecar_path))
                .with_id("cockpit")
                .with_mime("application/json"),
            Artifact::comment(path_str(&comment_path))
                .with_id("comment")
                .with_mime("text/markdown"),
        ]);

        // Render markdown AFTER data + artifacts are populated so gates section is included
        let comment_md = render_sensor_md(&report);
        std::fs::write(&comment_path, comment_md.as_bytes())?;

        // Write canonical JSON report to output path
        let json_str = serde_json::to_string_pretty(&report)?;
        let mut file = std::fs::File::create(output_path)
            .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
        file.write_all(json_str.as_bytes())?;

        // Print to stdout based on format flag
        match args.format {
            cli::SensorFormat::Json => {
                print!("{}", json_str);
            }
            cli::SensorFormat::Md => {
                print!("{}", comment_md);
            }
        }

        Ok(())
    }
}

#[cfg(feature = "git")]
fn build_summary(receipt: &super::cockpit::CockpitReceipt, base: &str, head: &str) -> String {
    format!(
        "{} files changed, +{}/-{}, health {}/100, risk {} in {}..{}",
        receipt.change_surface.files_changed,
        receipt.change_surface.insertions,
        receipt.change_surface.deletions,
        receipt.code_health.score,
        receipt.risk.level,
        base,
        head,
    )
}

/// Map cockpit GateStatus → envelope Verdict.
#[cfg(feature = "git")]
fn map_verdict(status: super::cockpit::GateStatus) -> Verdict {
    match status {
        super::cockpit::GateStatus::Pass => Verdict::Pass,
        super::cockpit::GateStatus::Warn => Verdict::Warn,
        super::cockpit::GateStatus::Fail => Verdict::Fail,
        super::cockpit::GateStatus::Skipped => Verdict::Skip,
        super::cockpit::GateStatus::Pending => Verdict::Pending,
    }
}

/// Map cockpit Evidence → envelope GateResults.
#[cfg(feature = "git")]
fn map_gates(evidence: &super::cockpit::Evidence) -> GateResults {
    let mut items = Vec::new();

    // Mutation gate (always present)
    items.push(
        GateItem::new("mutation", map_verdict(evidence.mutation.meta.status))
            .with_source("computed"),
    );

    // Optional gates
    if let Some(ref dc) = evidence.diff_coverage {
        items.push(
            GateItem::new("diff_coverage", map_verdict(dc.meta.status))
                .with_threshold(0.8, dc.coverage_pct)
                .with_source("computed"),
        );
    }

    if let Some(ref c) = evidence.contracts {
        let mut gate =
            GateItem::new("contracts", map_verdict(c.meta.status)).with_source("computed");
        if c.failures > 0 {
            gate = gate.with_reason(format!("{} sub-gate(s) failed", c.failures));
        }
        items.push(gate);
    }

    if let Some(ref sc) = evidence.supply_chain {
        items.push(
            GateItem::new("supply_chain", map_verdict(sc.meta.status)).with_source("computed"),
        );
    }

    if let Some(ref det) = evidence.determinism {
        items.push(
            GateItem::new("determinism", map_verdict(det.meta.status)).with_source("computed"),
        );
    }

    if let Some(ref cx) = evidence.complexity {
        items
            .push(GateItem::new("complexity", map_verdict(cx.meta.status)).with_source("computed"));
    }

    GateResults::new(map_verdict(evidence.overall_status), items)
}

/// Emit risk findings from cockpit data.
#[cfg(feature = "git")]
fn emit_risk_findings(report: &mut SensorReport, risk: &super::cockpit::Risk) {
    for hotspot in risk.hotspots_touched.iter().take(MAX_FINDINGS_PER_CATEGORY) {
        report.add_finding(
            Finding::new(
                findings::risk::CHECK_ID,
                findings::risk::HOTSPOT,
                FindingSeverity::Warn,
                "Hotspot file touched",
                format!("{} is a high-churn file", hotspot),
            )
            .with_location(tokmd_envelope::FindingLocation::path(hotspot))
            .with_fingerprint("tokmd"),
        );
    }

    for path in risk
        .bus_factor_warnings
        .iter()
        .take(MAX_FINDINGS_PER_CATEGORY)
    {
        report.add_finding(
            Finding::new(
                findings::risk::CHECK_ID,
                findings::risk::BUS_FACTOR,
                FindingSeverity::Warn,
                "Bus factor warning",
                format!("{} has single-author ownership", path),
            )
            .with_location(tokmd_envelope::FindingLocation::path(path))
            .with_fingerprint("tokmd"),
        );
    }
}

/// Emit contract findings from cockpit data.
#[cfg(feature = "git")]
fn emit_contract_findings(report: &mut SensorReport, contracts: &super::cockpit::Contracts) {
    if contracts.schema_changed {
        report.add_finding(
            Finding::new(
                findings::contract::CHECK_ID,
                findings::contract::SCHEMA_CHANGED,
                FindingSeverity::Info,
                "Schema version changed",
                "Schema version files were modified in this PR",
            )
            .with_fingerprint("tokmd"),
        );
    }
    if contracts.api_changed {
        report.add_finding(
            Finding::new(
                findings::contract::CHECK_ID,
                findings::contract::API_CHANGED,
                FindingSeverity::Warn,
                "Public API changed",
                "Public API surface files were modified",
            )
            .with_fingerprint("tokmd"),
        );
    }
    if contracts.cli_changed {
        report.add_finding(
            Finding::new(
                findings::contract::CHECK_ID,
                findings::contract::CLI_CHANGED,
                FindingSeverity::Info,
                "CLI interface changed",
                "CLI definition files were modified",
            )
            .with_fingerprint("tokmd"),
        );
    }
}

/// Emit complexity findings from cockpit evidence.
///
/// Inspects the complexity gate and emits per-file findings for high cyclomatic
/// complexity. Capped at `MAX_FINDINGS_PER_CATEGORY` per category.
#[cfg(feature = "git")]
fn emit_complexity_findings(report: &mut SensorReport, evidence: &super::cockpit::Evidence) {
    let Some(ref cx) = evidence.complexity else {
        return;
    };

    // Emit COMPLEXITY_HIGH findings for high-complexity files
    for file in cx
        .high_complexity_files
        .iter()
        .take(MAX_FINDINGS_PER_CATEGORY)
    {
        report.add_finding(
            Finding::new(
                findings::risk::CHECK_ID,
                findings::risk::COMPLEXITY_HIGH,
                FindingSeverity::Warn,
                "High cyclomatic complexity",
                format!(
                    "{} has cyclomatic complexity {} ({} functions)",
                    file.path, file.cyclomatic, file.function_count
                ),
            )
            .with_location(tokmd_envelope::FindingLocation::path(&file.path))
            .with_evidence(serde_json::json!({
                "cyclomatic": file.cyclomatic,
                "function_count": file.function_count,
                "max_function_length": file.max_function_length,
            }))
            .with_fingerprint("tokmd"),
        );
    }
}

/// Emit gate failure findings from cockpit evidence.
///
/// Inspects evidence gates and emits findings for any that failed.
#[cfg(feature = "git")]
fn emit_gate_findings(report: &mut SensorReport, evidence: &super::cockpit::Evidence) {
    // Mutation gate failure
    if evidence.mutation.meta.status == super::cockpit::GateStatus::Fail {
        report.add_finding(
            Finding::new(
                findings::gate::CHECK_ID,
                findings::gate::MUTATION_FAILED,
                FindingSeverity::Error,
                "Mutation gate failed",
                format!(
                    "{} mutation(s) survived testing",
                    evidence.mutation.survivors.len()
                ),
            )
            .with_fingerprint("tokmd"),
        );
    }

    // Diff coverage gate failure
    if let Some(ref dc) = evidence.diff_coverage
        && dc.meta.status == super::cockpit::GateStatus::Fail
    {
        report.add_finding(
            Finding::new(
                findings::gate::CHECK_ID,
                findings::gate::COVERAGE_FAILED,
                FindingSeverity::Error,
                "Diff coverage gate failed",
                format!(
                    "Coverage {:.1}% below threshold ({} of {} lines covered)",
                    dc.coverage_pct * 100.0,
                    dc.lines_covered,
                    dc.lines_added
                ),
            )
            .with_fingerprint("tokmd"),
        );
    }

    // Complexity gate failure
    if let Some(ref cx) = evidence.complexity
        && cx.meta.status == super::cockpit::GateStatus::Fail
    {
        report.add_finding(
            Finding::new(
                findings::gate::CHECK_ID,
                findings::gate::COMPLEXITY_FAILED,
                FindingSeverity::Error,
                "Complexity gate failed",
                format!(
                    "Max cyclomatic {} exceeds threshold ({} files analyzed)",
                    cx.max_cyclomatic, cx.files_analyzed
                ),
            )
            .with_fingerprint("tokmd"),
        );
    }
}

#[cfg(feature = "git")]
fn render_sensor_md(report: &SensorReport) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    let _ = writeln!(s, "## Sensor Report: {}", report.tool.name);
    let _ = writeln!(s);
    let _ = writeln!(s, "**Verdict**: {}", report.verdict);
    let _ = writeln!(s, "**Summary**: {}", report.summary);
    let _ = writeln!(s);

    if !report.findings.is_empty() {
        let _ = writeln!(s, "### Findings");
        let _ = writeln!(s);
        for f in &report.findings {
            let _ = writeln!(
                s,
                "- **[{}]** {}.{}: {} — {}",
                f.severity, f.check_id, f.code, f.title, f.message
            );
        }
        let _ = writeln!(s);
    }

    // Extract gates from data (gates are embedded inside data, not top-level)
    if let Some(ref data) = report.data
        && let Some(gates_val) = data.get("gates")
        && let Ok(gates) = serde_json::from_value::<GateResults>(gates_val.clone())
    {
        let _ = writeln!(s, "### Gates ({})", gates.status);
        let _ = writeln!(s);
        for g in &gates.items {
            let _ = writeln!(s, "- **{}**: {}", g.id, g.status);
        }
    }

    s
}

#[cfg(feature = "git")]
fn now_iso8601() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(all(test, feature = "git"))]
mod tests {
    use super::*;

    #[cfg(feature = "git")]
    use super::super::cockpit::{
        CommitMatch, ComplexityGate, ContractDiffGate, DeterminismGate, DiffCoverageGate, Evidence,
        EvidenceSource, GateMeta, GateStatus, HighComplexityFile, MutationGate, MutationSurvivor,
        Risk, RiskLevel, ScopeCoverage, SupplyChainGate, UncoveredHunk,
    };

    #[test]
    fn render_sensor_md_includes_findings_and_gates() {
        let mut report = SensorReport::new(
            ToolMeta::tokmd("1.0.0", "sensor"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Warn,
            "Summary text".to_string(),
        );
        report.add_finding(
            Finding::new(
                findings::risk::CHECK_ID,
                findings::risk::HOTSPOT,
                FindingSeverity::Warn,
                "Hotspot",
                "High churn detected",
            )
            .with_fingerprint("tokmd"),
        );

        let gates = GateResults::new(
            Verdict::Warn,
            vec![GateItem::new("mutation", Verdict::Warn).with_source("computed")],
        );
        report = report.with_data(serde_json::json!({
            "gates": serde_json::to_value(&gates).unwrap(),
        }));

        let md = render_sensor_md(&report);
        assert!(md.contains("## Sensor Report: tokmd"));
        assert!(md.contains("### Findings"));
        assert!(md.contains("risk.hotspot"));
        assert!(md.contains("### Gates (warn)"));
        assert!(md.contains("mutation"));
    }

    #[cfg(feature = "git")]
    #[test]
    fn map_verdict_covers_all_gate_statuses() {
        use super::super::cockpit::GateStatus;

        assert_eq!(map_verdict(GateStatus::Pass), Verdict::Pass);
        assert_eq!(map_verdict(GateStatus::Warn), Verdict::Warn);
        assert_eq!(map_verdict(GateStatus::Fail), Verdict::Fail);
        assert_eq!(map_verdict(GateStatus::Skipped), Verdict::Skip);
        assert_eq!(map_verdict(GateStatus::Pending), Verdict::Pending);
    }

    #[cfg(feature = "git")]
    fn sample_scope() -> ScopeCoverage {
        ScopeCoverage {
            relevant: vec![],
            tested: vec![],
            ratio: 1.0,
            lines_relevant: None,
            lines_tested: None,
        }
    }

    #[cfg(feature = "git")]
    fn sample_meta(status: GateStatus) -> GateMeta {
        GateMeta {
            status,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Exact,
            scope: sample_scope(),
            evidence_commit: None,
            evidence_generated_at_ms: None,
        }
    }

    #[cfg(feature = "git")]
    fn sample_mutation_gate(status: GateStatus) -> MutationGate {
        MutationGate {
            meta: sample_meta(status),
            survivors: vec![MutationSurvivor {
                file: "src/lib.rs".to_string(),
                line: 10,
                mutation: "replace".to_string(),
            }],
            killed: 0,
            timeout: 0,
            unviable: 0,
        }
    }

    #[cfg(feature = "git")]
    fn base_evidence() -> Evidence {
        Evidence {
            overall_status: GateStatus::Warn,
            mutation: sample_mutation_gate(GateStatus::Warn),
            diff_coverage: None,
            contracts: None,
            supply_chain: None,
            determinism: None,
            complexity: None,
        }
    }

    #[cfg(feature = "git")]
    #[test]
    fn build_summary_formats_expected_fields() {
        let receipt = super::super::cockpit::CockpitReceipt {
            schema_version: 3,
            generated_at_ms: 0,
            base_ref: "main".to_string(),
            head_ref: "HEAD".to_string(),
            change_surface: super::super::cockpit::ChangeSurface {
                commits: 1,
                files_changed: 2,
                insertions: 10,
                deletions: 5,
                net_lines: 5,
                churn_velocity: 15.0,
                change_concentration: 0.4,
            },
            composition: super::super::cockpit::Composition {
                code_pct: 0.8,
                test_pct: 0.1,
                docs_pct: 0.05,
                config_pct: 0.05,
                test_ratio: 0.2,
            },
            code_health: super::super::cockpit::CodeHealth {
                score: 75,
                grade: "B".to_string(),
                large_files_touched: 0,
                avg_file_size: 10,
                complexity_indicator: super::super::cockpit::ComplexityIndicator::Low,
                warnings: vec![],
            },
            risk: Risk {
                hotspots_touched: vec![],
                bus_factor_warnings: vec![],
                level: RiskLevel::High,
                score: 80,
            },
            contracts: super::super::cockpit::Contracts {
                api_changed: false,
                cli_changed: false,
                schema_changed: false,
                breaking_indicators: 0,
            },
            evidence: base_evidence(),
            review_plan: vec![],
            trend: None,
        };

        let summary = build_summary(&receipt, "main", "HEAD");
        assert!(summary.contains("2 files changed"));
        assert!(summary.contains("+10/-5"));
        assert!(summary.contains("health 75/100"));
        assert!(summary.contains("risk high"));
        assert!(summary.contains("main..HEAD"));
    }

    #[cfg(feature = "git")]
    #[test]
    fn map_gates_includes_optional_items_and_reasons() {
        let mut evidence = base_evidence();
        evidence.diff_coverage = Some(DiffCoverageGate {
            meta: sample_meta(GateStatus::Fail),
            lines_added: 10,
            lines_covered: 5,
            coverage_pct: 0.5,
            uncovered_hunks: vec![UncoveredHunk {
                file: "src/lib.rs".to_string(),
                start_line: 1,
                end_line: 3,
            }],
        });
        evidence.contracts = Some(ContractDiffGate {
            meta: sample_meta(GateStatus::Warn),
            semver: None,
            cli: None,
            schema: None,
            failures: 2,
        });
        evidence.supply_chain = Some(SupplyChainGate {
            meta: sample_meta(GateStatus::Pass),
            vulnerabilities: vec![],
            denied: vec![],
            advisory_db_version: None,
        });
        evidence.determinism = Some(DeterminismGate {
            meta: sample_meta(GateStatus::Warn),
            expected_hash: Some("abc".to_string()),
            actual_hash: Some("def".to_string()),
            algo: "blake3".to_string(),
            differences: vec!["target/app".to_string()],
        });
        evidence.complexity = Some(ComplexityGate {
            meta: sample_meta(GateStatus::Fail),
            files_analyzed: 1,
            high_complexity_files: vec![],
            avg_cyclomatic: 4.0,
            max_cyclomatic: 12,
            threshold_exceeded: true,
        });

        let gates = map_gates(&evidence);
        let ids: std::collections::HashSet<_> = gates.items.iter().map(|g| g.id.as_str()).collect();
        for id in [
            "mutation",
            "diff_coverage",
            "contracts",
            "supply_chain",
            "determinism",
            "complexity",
        ] {
            assert!(ids.contains(id), "missing gate {id}");
        }

        let diff_gate = gates
            .items
            .iter()
            .find(|g| g.id == "diff_coverage")
            .expect("diff gate");
        assert_eq!(diff_gate.threshold, Some(0.8));
        assert_eq!(diff_gate.actual, Some(0.5));

        let contracts_gate = gates
            .items
            .iter()
            .find(|g| g.id == "contracts")
            .expect("contracts gate");
        assert_eq!(
            contracts_gate.reason.as_deref(),
            Some("2 sub-gate(s) failed")
        );
    }

    #[cfg(feature = "git")]
    #[test]
    fn emit_risk_findings_emits_hotspots_and_bus_factor() {
        let mut report = SensorReport::new(
            ToolMeta::tokmd("1.0.0", "sensor"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Warn,
            "Summary".to_string(),
        );
        let risk = Risk {
            hotspots_touched: vec!["src/lib.rs".to_string(), "src/main.rs".to_string()],
            bus_factor_warnings: vec!["src/owner.rs".to_string()],
            level: RiskLevel::Medium,
            score: 50,
        };

        emit_risk_findings(&mut report, &risk);

        assert_eq!(report.findings.len(), 3);
        let hotspot = report
            .findings
            .iter()
            .find(|f| f.code == findings::risk::HOTSPOT)
            .expect("hotspot finding");
        assert!(hotspot.location.is_some());

        let bus_factor = report
            .findings
            .iter()
            .find(|f| f.code == findings::risk::BUS_FACTOR)
            .expect("bus factor finding");
        assert!(bus_factor.location.is_some());
    }

    #[cfg(feature = "git")]
    #[test]
    fn emit_contract_findings_emits_all_flags() {
        let mut report = SensorReport::new(
            ToolMeta::tokmd("1.0.0", "sensor"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Warn,
            "Summary".to_string(),
        );
        let contracts = super::super::cockpit::Contracts {
            api_changed: true,
            cli_changed: true,
            schema_changed: true,
            breaking_indicators: 1,
        };

        emit_contract_findings(&mut report, &contracts);

        assert_eq!(report.findings.len(), 3);
        let codes: std::collections::HashSet<_> =
            report.findings.iter().map(|f| f.code.as_str()).collect();
        for code in [
            findings::contract::SCHEMA_CHANGED,
            findings::contract::API_CHANGED,
            findings::contract::CLI_CHANGED,
        ] {
            assert!(codes.contains(code), "missing contract finding {code}");
        }
    }

    #[cfg(feature = "git")]
    #[test]
    fn emit_complexity_findings_is_capped() {
        let mut report = SensorReport::new(
            ToolMeta::tokmd("1.0.0", "sensor"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Warn,
            "Summary".to_string(),
        );

        let files: Vec<HighComplexityFile> = (0..(MAX_FINDINGS_PER_CATEGORY + 2))
            .map(|idx| HighComplexityFile {
                path: format!("src/file{idx}.rs"),
                cyclomatic: 12,
                function_count: 3,
                max_function_length: 10,
            })
            .collect();

        let mut evidence = base_evidence();
        evidence.complexity = Some(ComplexityGate {
            meta: sample_meta(GateStatus::Warn),
            files_analyzed: files.len(),
            high_complexity_files: files,
            avg_cyclomatic: 3.2,
            max_cyclomatic: 12,
            threshold_exceeded: true,
        });

        emit_complexity_findings(&mut report, &evidence);
        assert_eq!(report.findings.len(), MAX_FINDINGS_PER_CATEGORY);
    }

    #[cfg(feature = "git")]
    #[test]
    fn emit_gate_findings_emits_failures() {
        let mut report = SensorReport::new(
            ToolMeta::tokmd("1.0.0", "sensor"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Warn,
            "Summary".to_string(),
        );

        let mut evidence = base_evidence();
        evidence.mutation = sample_mutation_gate(GateStatus::Fail);
        evidence.diff_coverage = Some(DiffCoverageGate {
            meta: sample_meta(GateStatus::Fail),
            lines_added: 20,
            lines_covered: 5,
            coverage_pct: 0.25,
            uncovered_hunks: vec![],
        });
        evidence.complexity = Some(ComplexityGate {
            meta: sample_meta(GateStatus::Fail),
            files_analyzed: 4,
            high_complexity_files: vec![],
            avg_cyclomatic: 6.0,
            max_cyclomatic: 18,
            threshold_exceeded: true,
        });

        emit_gate_findings(&mut report, &evidence);

        let codes: std::collections::HashSet<_> =
            report.findings.iter().map(|f| f.code.as_str()).collect();
        for code in [
            findings::gate::MUTATION_FAILED,
            findings::gate::COVERAGE_FAILED,
            findings::gate::COMPLEXITY_FAILED,
        ] {
            assert!(codes.contains(code), "missing gate finding {code}");
        }
    }
}
