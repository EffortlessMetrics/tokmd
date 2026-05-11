//! Sensor finding emission from cockpit evidence.

use tokmd_envelope::findings as envelope_findings;
use tokmd_envelope::{Finding, FindingSeverity, SensorReport};

use super::super::cockpit;

/// Maximum findings emitted per category to avoid spamming the bus.
const MAX_FINDINGS_PER_CATEGORY: usize = 10;

/// Emit risk findings from cockpit data.
pub(super) fn emit_risk_findings(report: &mut SensorReport, risk: &cockpit::Risk) {
    for hotspot in risk.hotspots_touched.iter().take(MAX_FINDINGS_PER_CATEGORY) {
        report.add_finding(
            Finding::new(
                envelope_findings::risk::CHECK_ID,
                envelope_findings::risk::HOTSPOT,
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
                envelope_findings::risk::CHECK_ID,
                envelope_findings::risk::BUS_FACTOR,
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
pub(super) fn emit_contract_findings(report: &mut SensorReport, contracts: &cockpit::Contracts) {
    if contracts.schema_changed {
        report.add_finding(
            Finding::new(
                envelope_findings::contract::CHECK_ID,
                envelope_findings::contract::SCHEMA_CHANGED,
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
                envelope_findings::contract::CHECK_ID,
                envelope_findings::contract::API_CHANGED,
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
                envelope_findings::contract::CHECK_ID,
                envelope_findings::contract::CLI_CHANGED,
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
pub(super) fn emit_complexity_findings(report: &mut SensorReport, evidence: &cockpit::Evidence) {
    let Some(ref cx) = evidence.complexity else {
        return;
    };

    for file in cx
        .high_complexity_files
        .iter()
        .take(MAX_FINDINGS_PER_CATEGORY)
    {
        report.add_finding(
            Finding::new(
                envelope_findings::risk::CHECK_ID,
                envelope_findings::risk::COMPLEXITY_HIGH,
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
pub(super) fn emit_gate_findings(report: &mut SensorReport, evidence: &cockpit::Evidence) {
    if evidence.mutation.meta.status == cockpit::GateStatus::Fail {
        report.add_finding(
            Finding::new(
                envelope_findings::gate::CHECK_ID,
                envelope_findings::gate::MUTATION_FAILED,
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

    if let Some(ref dc) = evidence.diff_coverage
        && dc.meta.status == cockpit::GateStatus::Fail
    {
        report.add_finding(
            Finding::new(
                envelope_findings::gate::CHECK_ID,
                envelope_findings::gate::COVERAGE_FAILED,
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

    if let Some(ref cx) = evidence.complexity
        && cx.meta.status == cockpit::GateStatus::Fail
    {
        report.add_finding(
            Finding::new(
                envelope_findings::gate::CHECK_ID,
                envelope_findings::gate::COMPLEXITY_FAILED,
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_envelope::{ToolMeta, Verdict};

    use super::super::super::cockpit::{
        CommitMatch, ComplexityGate, DiffCoverageGate, Evidence, EvidenceSource, GateMeta,
        GateStatus, HighComplexityFile, MutationGate, MutationSurvivor, Risk, RiskLevel,
        ScopeCoverage, UncoveredHunk,
    };

    fn sample_scope() -> ScopeCoverage {
        ScopeCoverage {
            relevant: vec![],
            tested: vec![],
            ratio: 1.0,
            lines_relevant: None,
            lines_tested: None,
        }
    }

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
            .find(|f| f.code == envelope_findings::risk::HOTSPOT)
            .expect("hotspot finding should be in report.findings");
        assert!(hotspot.location.is_some());

        let bus_factor = report
            .findings
            .iter()
            .find(|f| f.code == envelope_findings::risk::BUS_FACTOR)
            .expect("bus_factor finding should be in report.findings");
        assert!(bus_factor.location.is_some());
    }

    #[test]
    fn emit_contract_findings_emits_all_flags() {
        let mut report = SensorReport::new(
            ToolMeta::tokmd("1.0.0", "sensor"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Warn,
            "Summary".to_string(),
        );
        let contracts = cockpit::Contracts {
            api_changed: true,
            cli_changed: true,
            schema_changed: true,
            breaking_indicators: 1,
        };

        emit_contract_findings(&mut report, &contracts);

        assert_eq!(report.findings.len(), 3);
        let codes: std::collections::BTreeSet<_> =
            report.findings.iter().map(|f| f.code.as_str()).collect();
        for code in [
            envelope_findings::contract::SCHEMA_CHANGED,
            envelope_findings::contract::API_CHANGED,
            envelope_findings::contract::CLI_CHANGED,
        ] {
            assert!(codes.contains(code), "missing contract finding {code}");
        }
    }

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
            uncovered_hunks: vec![UncoveredHunk {
                file: "src/lib.rs".to_string(),
                start_line: 1,
                end_line: 3,
            }],
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

        let codes: std::collections::BTreeSet<_> =
            report.findings.iter().map(|f| f.code.as_str()).collect();
        for code in [
            envelope_findings::gate::MUTATION_FAILED,
            envelope_findings::gate::COVERAGE_FAILED,
            envelope_findings::gate::COMPLEXITY_FAILED,
        ] {
            assert!(codes.contains(code), "missing gate finding {code}");
        }
    }
}
