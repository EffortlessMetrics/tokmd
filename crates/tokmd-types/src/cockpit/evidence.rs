//! Cockpit evidence gate receipt DTOs.
//!
//! These types describe the gate evidence embedded in cockpit receipts. They
//! stay serde-stable because review packets, hosted comments, and downstream
//! evidence consumers read these fields directly.

use serde::{Deserialize, Serialize};

/// Evidence section containing hard gates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Aggregate status of all gates.
    pub overall_status: GateStatus,
    /// Mutation testing gate (always present).
    pub mutation: MutationGate,
    /// Diff coverage gate (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_coverage: Option<DiffCoverageGate>,
    /// Contract diff gate (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts: Option<ContractDiffGate>,
    /// Supply chain gate (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supply_chain: Option<SupplyChainGate>,
    /// Determinism gate (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub determinism: Option<DeterminismGate>,
    /// Complexity gate (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<ComplexityGate>,
}

/// Status of a gate check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GateStatus {
    Pass,
    Warn,
    Fail,
    Skipped,
    Pending,
}

/// Source of evidence/gate results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    CiArtifact,
    Cached,
    RanLocal,
}

/// Commit match quality for evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommitMatch {
    Exact,
    Partial,
    Stale,
    Unknown,
}

/// Common metadata for all gates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateMeta {
    pub status: GateStatus,
    pub source: EvidenceSource,
    pub commit_match: CommitMatch,
    pub scope: ScopeCoverage,
    /// SHA this evidence was generated for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_commit: Option<String>,
    /// Timestamp when evidence was generated (ms since epoch).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_generated_at_ms: Option<u64>,
}

/// Scope coverage for a gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeCoverage {
    /// Files in scope for the gate.
    pub relevant: Vec<String>,
    /// Files actually tested.
    pub tested: Vec<String>,
    /// Coverage ratio (tested/relevant, 0.0-1.0).
    pub ratio: f64,
    /// Lines in scope (optional, for line-level gates).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines_relevant: Option<usize>,
    /// Lines actually tested (optional, for line-level gates).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines_tested: Option<usize>,
}

/// Mutation testing gate results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationGate {
    #[serde(flatten)]
    pub meta: GateMeta,
    pub survivors: Vec<MutationSurvivor>,
    pub killed: usize,
    pub timeout: usize,
    pub unviable: usize,
}

/// A mutation that survived testing (escaped detection).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationSurvivor {
    pub file: String,
    pub line: usize,
    pub mutation: String,
}

/// Diff coverage gate results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffCoverageGate {
    #[serde(flatten)]
    pub meta: GateMeta,
    pub lines_added: usize,
    pub lines_covered: usize,
    pub coverage_pct: f64,
    pub uncovered_hunks: Vec<UncoveredHunk>,
}

/// Uncovered hunk in diff coverage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncoveredHunk {
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Contract diff gate results (compound gate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDiffGate {
    #[serde(flatten)]
    pub meta: GateMeta,
    /// Semver sub-gate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semver: Option<SemverSubGate>,
    /// CLI sub-gate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli: Option<CliSubGate>,
    /// Schema sub-gate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<SchemaSubGate>,
    /// Count of failed sub-gates.
    pub failures: usize,
}

/// Semver sub-gate for contract diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemverSubGate {
    pub status: GateStatus,
    pub breaking_changes: Vec<BreakingChange>,
}

/// Breaking change detected by semver check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    pub kind: String,
    pub path: String,
    pub message: String,
}

/// CLI sub-gate for contract diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliSubGate {
    pub status: GateStatus,
    pub diff_summary: Option<String>,
}

/// Schema sub-gate for contract diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaSubGate {
    pub status: GateStatus,
    pub diff_summary: Option<String>,
}

/// Supply chain gate results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyChainGate {
    #[serde(flatten)]
    pub meta: GateMeta,
    pub vulnerabilities: Vec<Vulnerability>,
    pub denied: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisory_db_version: Option<String>,
}

/// Vulnerability from cargo-audit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub package: String,
    pub severity: String,
    pub title: String,
}

/// Determinism gate results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismGate {
    #[serde(flatten)]
    pub meta: GateMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_hash: Option<String>,
    pub algo: String,
    pub differences: Vec<String>,
}

/// Complexity gate results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityGate {
    #[serde(flatten)]
    pub meta: GateMeta,
    /// Number of files analyzed for complexity.
    pub files_analyzed: usize,
    /// Files with high complexity (CC > threshold).
    pub high_complexity_files: Vec<HighComplexityFile>,
    /// Average cyclomatic complexity across all analyzed files.
    pub avg_cyclomatic: f64,
    /// Maximum cyclomatic complexity found.
    pub max_cyclomatic: u32,
    /// Whether the threshold was exceeded.
    pub threshold_exceeded: bool,
}

/// A file with high cyclomatic complexity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighComplexityFile {
    /// Path to the file.
    pub path: String,
    /// Cyclomatic complexity score.
    pub cyclomatic: u32,
    /// Number of functions in the file.
    pub function_count: usize,
    /// Maximum function length in lines.
    pub max_function_length: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_scope() -> ScopeCoverage {
        ScopeCoverage {
            relevant: vec!["src/a.rs".into(), "src/b.rs".into()],
            tested: vec!["src/a.rs".into()],
            ratio: 0.5,
            lines_relevant: Some(100),
            lines_tested: Some(50),
        }
    }

    fn sample_meta() -> GateMeta {
        GateMeta {
            status: GateStatus::Pass,
            source: EvidenceSource::CiArtifact,
            commit_match: CommitMatch::Exact,
            scope: sample_scope(),
            evidence_commit: Some("abc123".into()),
            evidence_generated_at_ms: Some(1_700_000_000_000),
        }
    }

    fn sample_mutation_gate() -> MutationGate {
        MutationGate {
            meta: sample_meta(),
            survivors: vec![MutationSurvivor {
                file: "src/a.rs".into(),
                line: 42,
                mutation: "replace + with -".into(),
            }],
            killed: 10,
            timeout: 1,
            unviable: 2,
        }
    }

    // ── Enum stability + roundtrips ─────────────────────────────────
    #[test]
    fn gate_status_uses_lowercase() {
        assert_eq!(
            serde_json::to_string(&GateStatus::Pass).unwrap(),
            "\"pass\""
        );
        assert_eq!(
            serde_json::to_string(&GateStatus::Skipped).unwrap(),
            "\"skipped\""
        );
        assert_eq!(
            serde_json::to_string(&GateStatus::Pending).unwrap(),
            "\"pending\""
        );
    }

    #[test]
    fn gate_status_all_variants_roundtrip() {
        for variant in [
            GateStatus::Pass,
            GateStatus::Warn,
            GateStatus::Fail,
            GateStatus::Skipped,
            GateStatus::Pending,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: GateStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn evidence_source_uses_snake_case() {
        assert_eq!(
            serde_json::to_string(&EvidenceSource::CiArtifact).unwrap(),
            "\"ci_artifact\""
        );
        assert_eq!(
            serde_json::to_string(&EvidenceSource::RanLocal).unwrap(),
            "\"ran_local\""
        );
        assert_eq!(
            serde_json::to_string(&EvidenceSource::Cached).unwrap(),
            "\"cached\""
        );
    }

    #[test]
    fn evidence_source_all_variants_roundtrip() {
        for variant in [
            EvidenceSource::CiArtifact,
            EvidenceSource::Cached,
            EvidenceSource::RanLocal,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: EvidenceSource = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn commit_match_uses_lowercase() {
        assert_eq!(
            serde_json::to_string(&CommitMatch::Exact).unwrap(),
            "\"exact\""
        );
        assert_eq!(
            serde_json::to_string(&CommitMatch::Stale).unwrap(),
            "\"stale\""
        );
    }

    #[test]
    fn commit_match_all_variants_roundtrip() {
        for variant in [
            CommitMatch::Exact,
            CommitMatch::Partial,
            CommitMatch::Stale,
            CommitMatch::Unknown,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: CommitMatch = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    // ── ScopeCoverage ───────────────────────────────────────────────
    #[test]
    fn scope_coverage_serde_roundtrip() {
        let s = sample_scope();
        let json = serde_json::to_string(&s).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        for key in ["relevant", "tested", "ratio"] {
            assert!(value.get(key).is_some(), "missing key `{key}`");
        }
        assert_eq!(value["lines_relevant"], 100);
        assert_eq!(value["lines_tested"], 50);
    }

    #[test]
    fn scope_coverage_omits_optional_line_fields_when_none() {
        let s = ScopeCoverage {
            relevant: vec![],
            tested: vec![],
            ratio: 0.0,
            lines_relevant: None,
            lines_tested: None,
        };
        let value = serde_json::to_value(&s).unwrap();
        assert!(value.get("lines_relevant").is_none());
        assert!(value.get("lines_tested").is_none());
    }

    #[test]
    fn scope_coverage_preserves_file_order() {
        let s = ScopeCoverage {
            relevant: vec!["c".into(), "a".into(), "b".into()],
            tested: vec!["z".into(), "y".into()],
            ratio: 0.5,
            lines_relevant: None,
            lines_tested: None,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: ScopeCoverage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.relevant, vec!["c", "a", "b"]);
        assert_eq!(back.tested, vec!["z", "y"]);
    }

    // ── GateMeta ────────────────────────────────────────────────────
    #[test]
    fn gate_meta_with_evidence_metadata_roundtrip() {
        let m = sample_meta();
        let json = serde_json::to_string(&m).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        for key in [
            "status",
            "source",
            "commit_match",
            "scope",
            "evidence_commit",
            "evidence_generated_at_ms",
        ] {
            assert!(value.get(key).is_some(), "missing key `{key}` in GateMeta");
        }
        assert_eq!(value["evidence_commit"], "abc123");
    }

    #[test]
    fn gate_meta_without_optional_metadata_omits_keys() {
        let m = GateMeta {
            status: GateStatus::Warn,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Unknown,
            scope: sample_scope(),
            evidence_commit: None,
            evidence_generated_at_ms: None,
        };
        let value = serde_json::to_value(&m).unwrap();
        assert!(value.get("evidence_commit").is_none());
        assert!(value.get("evidence_generated_at_ms").is_none());
    }

    // ── MutationGate ────────────────────────────────────────────────
    #[test]
    fn mutation_gate_flattens_meta_fields() {
        let m = sample_mutation_gate();
        let value = serde_json::to_value(&m).unwrap();
        // GateMeta fields are flattened into the gate JSON
        for key in ["status", "source", "commit_match", "scope"] {
            assert!(
                value.get(key).is_some(),
                "missing flattened meta key `{key}`"
            );
        }
        // Mutation-specific fields
        for key in ["survivors", "killed", "timeout", "unviable"] {
            assert!(
                value.get(key).is_some(),
                "missing mutation gate key `{key}`"
            );
        }
        let json = value.to_string();
        let back: MutationGate = serde_json::from_str(&json).unwrap();
        assert_eq!(back.killed, 10);
        assert_eq!(back.survivors.len(), 1);
        assert_eq!(back.survivors[0].line, 42);
    }

    #[test]
    fn mutation_survivor_field_names_stable() {
        let s = MutationSurvivor {
            file: "src/x.rs".into(),
            line: 1,
            mutation: "swap".into(),
        };
        let value = serde_json::to_value(&s).unwrap();
        for key in ["file", "line", "mutation"] {
            assert!(value.get(key).is_some());
        }
        let json = serde_json::to_string(&s).unwrap();
        let back: MutationSurvivor = serde_json::from_str(&json).unwrap();
        assert_eq!(back.file, "src/x.rs");
        assert_eq!(back.line, 1);
    }

    // ── DiffCoverageGate ────────────────────────────────────────────
    #[test]
    fn diff_coverage_gate_flattens_meta() {
        let g = DiffCoverageGate {
            meta: sample_meta(),
            lines_added: 100,
            lines_covered: 70,
            coverage_pct: 0.7,
            uncovered_hunks: vec![UncoveredHunk {
                file: "src/x.rs".into(),
                start_line: 10,
                end_line: 20,
            }],
        };
        let value = serde_json::to_value(&g).unwrap();
        assert!(value.get("status").is_some());
        assert!(value.get("lines_added").is_some());
        assert!(value.get("coverage_pct").is_some());
        assert!(value.get("uncovered_hunks").is_some());
        let back: DiffCoverageGate = serde_json::from_value(value).unwrap();
        assert_eq!(back.lines_added, 100);
        assert_eq!(back.uncovered_hunks[0].start_line, 10);
        assert_eq!(back.uncovered_hunks[0].end_line, 20);
    }

    #[test]
    fn uncovered_hunk_roundtrip() {
        let h = UncoveredHunk {
            file: "f".into(),
            start_line: 5,
            end_line: 7,
        };
        let json = serde_json::to_string(&h).unwrap();
        let back: UncoveredHunk = serde_json::from_str(&json).unwrap();
        assert_eq!(back.file, "f");
        assert_eq!(back.start_line, 5);
        assert_eq!(back.end_line, 7);
    }

    // ── ContractDiffGate + sub-gates ────────────────────────────────
    #[test]
    fn contract_diff_gate_with_all_sub_gates_present() {
        let g = ContractDiffGate {
            meta: sample_meta(),
            semver: Some(SemverSubGate {
                status: GateStatus::Fail,
                breaking_changes: vec![BreakingChange {
                    kind: "removal".into(),
                    path: "foo::bar".into(),
                    message: "function removed".into(),
                }],
            }),
            cli: Some(CliSubGate {
                status: GateStatus::Pass,
                diff_summary: Some("no change".into()),
            }),
            schema: Some(SchemaSubGate {
                status: GateStatus::Pass,
                diff_summary: None,
            }),
            failures: 1,
        };
        let value = serde_json::to_value(&g).unwrap();
        assert!(value.get("semver").is_some());
        assert!(value.get("cli").is_some());
        assert!(value.get("schema").is_some());
        assert_eq!(value["failures"], 1);
        // Flattened meta
        assert!(value.get("status").is_some());
        let back: ContractDiffGate = serde_json::from_value(value).unwrap();
        assert_eq!(back.failures, 1);
        let semver = back.semver.expect("semver present");
        assert_eq!(semver.status, GateStatus::Fail);
        assert_eq!(semver.breaking_changes[0].kind, "removal");
    }

    #[test]
    fn contract_diff_gate_with_no_sub_gates_omits_them() {
        let g = ContractDiffGate {
            meta: sample_meta(),
            semver: None,
            cli: None,
            schema: None,
            failures: 0,
        };
        let value = serde_json::to_value(&g).unwrap();
        assert!(value.get("semver").is_none());
        assert!(value.get("cli").is_none());
        assert!(value.get("schema").is_none());
    }

    #[test]
    fn breaking_change_field_names_stable() {
        let b = BreakingChange {
            kind: "rename".into(),
            path: "x::y".into(),
            message: "x renamed".into(),
        };
        let value = serde_json::to_value(&b).unwrap();
        for key in ["kind", "path", "message"] {
            assert!(value.get(key).is_some());
        }
    }

    // ── SupplyChainGate ─────────────────────────────────────────────
    #[test]
    fn supply_chain_gate_roundtrip() {
        let g = SupplyChainGate {
            meta: sample_meta(),
            vulnerabilities: vec![Vulnerability {
                id: "CVE-2025-1".into(),
                package: "libfoo".into(),
                severity: "high".into(),
                title: "use after free".into(),
            }],
            denied: vec!["GPL-3.0".into()],
            advisory_db_version: Some("2025-05-01".into()),
        };
        let json = serde_json::to_string(&g).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["vulnerabilities"][0]["id"], "CVE-2025-1");
        assert_eq!(value["denied"][0], "GPL-3.0");
        assert_eq!(value["advisory_db_version"], "2025-05-01");
        let back: SupplyChainGate = serde_json::from_str(&json).unwrap();
        assert_eq!(back.vulnerabilities.len(), 1);
        assert_eq!(back.denied, vec!["GPL-3.0".to_string()]);
    }

    #[test]
    fn supply_chain_gate_omits_advisory_db_when_none() {
        let g = SupplyChainGate {
            meta: sample_meta(),
            vulnerabilities: vec![],
            denied: vec![],
            advisory_db_version: None,
        };
        let value = serde_json::to_value(&g).unwrap();
        assert!(value.get("advisory_db_version").is_none());
    }

    // ── DeterminismGate ─────────────────────────────────────────────
    #[test]
    fn determinism_gate_roundtrip_with_hashes() {
        let g = DeterminismGate {
            meta: sample_meta(),
            expected_hash: Some("aaaa".into()),
            actual_hash: Some("bbbb".into()),
            algo: "sha256".into(),
            differences: vec!["file1.txt: bytes differ".into()],
        };
        let json = serde_json::to_string(&g).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["expected_hash"], "aaaa");
        assert_eq!(value["actual_hash"], "bbbb");
        assert_eq!(value["algo"], "sha256");
        let back: DeterminismGate = serde_json::from_str(&json).unwrap();
        assert_eq!(back.expected_hash.as_deref(), Some("aaaa"));
        assert_eq!(back.differences.len(), 1);
    }

    #[test]
    fn determinism_gate_omits_hashes_when_none() {
        let g = DeterminismGate {
            meta: sample_meta(),
            expected_hash: None,
            actual_hash: None,
            algo: "blake3".into(),
            differences: vec![],
        };
        let value = serde_json::to_value(&g).unwrap();
        assert!(value.get("expected_hash").is_none());
        assert!(value.get("actual_hash").is_none());
        assert_eq!(value["algo"], "blake3");
    }

    // ── ComplexityGate ──────────────────────────────────────────────
    #[test]
    fn complexity_gate_roundtrip() {
        let g = ComplexityGate {
            meta: sample_meta(),
            files_analyzed: 25,
            high_complexity_files: vec![HighComplexityFile {
                path: "src/big.rs".into(),
                cyclomatic: 35,
                function_count: 12,
                max_function_length: 250,
            }],
            avg_cyclomatic: 6.5,
            max_cyclomatic: 35,
            threshold_exceeded: true,
        };
        let json = serde_json::to_string(&g).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        for key in [
            "files_analyzed",
            "high_complexity_files",
            "avg_cyclomatic",
            "max_cyclomatic",
            "threshold_exceeded",
        ] {
            assert!(
                value.get(key).is_some(),
                "missing key `{key}` in ComplexityGate"
            );
        }
        let back: ComplexityGate = serde_json::from_str(&json).unwrap();
        assert!(back.threshold_exceeded);
        assert_eq!(back.high_complexity_files[0].cyclomatic, 35);
        assert_eq!(back.high_complexity_files[0].max_function_length, 250);
    }

    #[test]
    fn high_complexity_file_field_names_stable() {
        let f = HighComplexityFile {
            path: "p".into(),
            cyclomatic: 1,
            function_count: 2,
            max_function_length: 3,
        };
        let value = serde_json::to_value(&f).unwrap();
        for key in [
            "path",
            "cyclomatic",
            "function_count",
            "max_function_length",
        ] {
            assert!(value.get(key).is_some());
        }
    }

    // ── Evidence (top-level) ────────────────────────────────────────
    #[test]
    fn evidence_only_mutation_required() {
        let e = Evidence {
            overall_status: GateStatus::Pass,
            mutation: sample_mutation_gate(),
            diff_coverage: None,
            contracts: None,
            supply_chain: None,
            determinism: None,
            complexity: None,
        };
        let value = serde_json::to_value(&e).unwrap();
        // mutation is always present
        assert!(value.get("mutation").is_some());
        assert!(value.get("overall_status").is_some());
        // Other optional gates are omitted when None
        assert!(value.get("diff_coverage").is_none());
        assert!(value.get("contracts").is_none());
        assert!(value.get("supply_chain").is_none());
        assert!(value.get("determinism").is_none());
        assert!(value.get("complexity").is_none());
        let back: Evidence = serde_json::from_value(value).unwrap();
        assert_eq!(back.overall_status, GateStatus::Pass);
        assert_eq!(back.mutation.killed, 10);
    }

    #[test]
    fn evidence_with_all_gates_serde_roundtrip() {
        let e = Evidence {
            overall_status: GateStatus::Fail,
            mutation: sample_mutation_gate(),
            diff_coverage: Some(DiffCoverageGate {
                meta: sample_meta(),
                lines_added: 1,
                lines_covered: 1,
                coverage_pct: 1.0,
                uncovered_hunks: vec![],
            }),
            contracts: Some(ContractDiffGate {
                meta: sample_meta(),
                semver: None,
                cli: None,
                schema: None,
                failures: 0,
            }),
            supply_chain: Some(SupplyChainGate {
                meta: sample_meta(),
                vulnerabilities: vec![],
                denied: vec![],
                advisory_db_version: None,
            }),
            determinism: Some(DeterminismGate {
                meta: sample_meta(),
                expected_hash: None,
                actual_hash: None,
                algo: "sha256".into(),
                differences: vec![],
            }),
            complexity: Some(ComplexityGate {
                meta: sample_meta(),
                files_analyzed: 0,
                high_complexity_files: vec![],
                avg_cyclomatic: 0.0,
                max_cyclomatic: 0,
                threshold_exceeded: false,
            }),
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: Evidence = serde_json::from_str(&json).unwrap();
        assert_eq!(back.overall_status, GateStatus::Fail);
        assert!(back.diff_coverage.is_some());
        assert!(back.contracts.is_some());
        assert!(back.supply_chain.is_some());
        assert!(back.determinism.is_some());
        assert!(back.complexity.is_some());
    }

    #[test]
    fn semver_sub_gate_with_status_only() {
        let s = SemverSubGate {
            status: GateStatus::Pass,
            breaking_changes: vec![],
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: SemverSubGate = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, GateStatus::Pass);
        assert!(back.breaking_changes.is_empty());
    }

    #[test]
    fn cli_sub_gate_with_diff_summary_none_still_emits_null() {
        // CliSubGate does not have skip_serializing_if on diff_summary
        let s = CliSubGate {
            status: GateStatus::Skipped,
            diff_summary: None,
        };
        let value = serde_json::to_value(&s).unwrap();
        assert!(value.get("diff_summary").is_some());
        assert!(value["diff_summary"].is_null());
    }

    #[test]
    fn schema_sub_gate_serde_roundtrip() {
        let s = SchemaSubGate {
            status: GateStatus::Warn,
            diff_summary: Some("schema changed".into()),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: SchemaSubGate = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, GateStatus::Warn);
        assert_eq!(back.diff_summary.as_deref(), Some("schema changed"));
    }
}
