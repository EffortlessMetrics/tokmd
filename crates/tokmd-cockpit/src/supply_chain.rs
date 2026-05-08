//! Supply-chain evidence gate for cockpit receipts.

use std::path::Path;
use std::process::Command;

use anyhow::Result;
use serde::Deserialize;
use tokmd_types::cockpit::{
    CommitMatch, EvidenceSource, GateMeta, GateStatus, ScopeCoverage, SupplyChainGate,
    Vulnerability,
};

use crate::FileStat;

/// Compute supply-chain gate evidence.
///
/// The gate is scoped to `Cargo.lock` changes. It runs `cargo audit --json`
/// when the tool is available; otherwise it records pending local evidence.
#[cfg(feature = "git")]
pub(crate) fn compute_supply_chain_gate(
    repo_root: &Path,
    changed_files: &[FileStat],
) -> Result<Option<SupplyChainGate>> {
    let lock_changed = changed_files.iter().any(|f| f.path.ends_with("Cargo.lock"));
    if !lock_changed {
        return Ok(None);
    }

    let check = Command::new("cargo").arg("audit").arg("--version").output();
    let audit_available = check.as_ref().map(|o| o.status.success()).unwrap_or(false);

    if !audit_available {
        return Ok(Some(pending_supply_chain_gate()));
    }

    let audit_output = Command::new("cargo")
        .args(["audit", "--json"])
        .current_dir(repo_root)
        .output();

    let output = match audit_output {
        Ok(o) => o,
        Err(_) => return Ok(Some(pending_supply_chain_gate())),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let (vulnerabilities, advisory_db_version, status) = parse_audit_output(&stdout);

    Ok(Some(SupplyChainGate {
        meta: GateMeta {
            status,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: vec!["Cargo.lock".to_string()],
                tested: vec!["Cargo.lock".to_string()],
                ratio: 1.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        vulnerabilities,
        denied: Vec::new(),
        advisory_db_version,
    }))
}

fn pending_supply_chain_gate() -> SupplyChainGate {
    SupplyChainGate {
        meta: GateMeta {
            status: GateStatus::Pending,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: vec!["Cargo.lock".to_string()],
                tested: Vec::new(),
                ratio: 0.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        vulnerabilities: Vec::new(),
        denied: Vec::new(),
        advisory_db_version: None,
    }
}

fn parse_audit_output(stdout: &str) -> (Vec<Vulnerability>, Option<String>, GateStatus) {
    let parsed: Result<AuditOutput, _> = serde_json::from_str(stdout);

    match parsed {
        Ok(audit) => {
            let db_version = audit.database.and_then(|db| db.version);

            let vulns: Vec<Vulnerability> = audit
                .vulnerabilities
                .and_then(|v| v.list)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|entry| {
                    let advisory = entry.advisory?;
                    Some(Vulnerability {
                        id: advisory.id.unwrap_or_default(),
                        package: entry.package.and_then(|p| p.name).unwrap_or_default(),
                        severity: advisory
                            .severity
                            .clone()
                            .unwrap_or_else(|| "unknown".to_string()),
                        title: advisory.title.unwrap_or_default(),
                    })
                })
                .collect();

            let has_critical_or_high = vulns.iter().any(|v| {
                let sev = v.severity.to_lowercase();
                sev == "critical" || sev == "high"
            });
            let has_medium = vulns.iter().any(|v| v.severity.to_lowercase() == "medium");

            let status = if has_critical_or_high {
                GateStatus::Fail
            } else if has_medium {
                GateStatus::Warn
            } else {
                GateStatus::Pass
            };

            (vulns, db_version, status)
        }
        Err(_) => (Vec::new(), None, GateStatus::Pending),
    }
}

#[derive(Deserialize)]
struct AuditOutput {
    database: Option<AuditDatabase>,
    vulnerabilities: Option<AuditVulnerabilities>,
}

#[derive(Deserialize)]
struct AuditDatabase {
    version: Option<String>,
}

#[derive(Deserialize)]
struct AuditVulnerabilities {
    list: Option<Vec<AuditVulnEntry>>,
}

#[derive(Deserialize)]
struct AuditVulnEntry {
    advisory: Option<AuditAdvisory>,
    package: Option<AuditPackage>,
}

#[derive(Deserialize)]
struct AuditAdvisory {
    id: Option<String>,
    severity: Option<String>,
    title: Option<String>,
}

#[derive(Deserialize)]
struct AuditPackage {
    name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{parse_audit_output, pending_supply_chain_gate};
    use tokmd_types::cockpit::GateStatus;

    #[test]
    fn pending_gate_records_cargo_lock_as_untested() {
        let gate = pending_supply_chain_gate();

        assert_eq!(gate.meta.status, GateStatus::Pending);
        assert_eq!(gate.meta.scope.relevant, vec!["Cargo.lock"]);
        assert!(gate.meta.scope.tested.is_empty());
        assert_eq!(gate.meta.scope.ratio, 0.0);
    }

    #[test]
    fn parse_audit_output_fails_for_high_or_critical_vulnerabilities() {
        let (vulns, db_version, status) = parse_audit_output(
            r#"{
  "database": { "advisory-count": 1, "version": "2026-05-08" },
  "vulnerabilities": {
    "found": true,
    "count": 1,
    "list": [
      {
        "advisory": { "id": "RUSTSEC-0000-0000", "severity": "high", "title": "bad" },
        "package": { "name": "example" }
      }
    ]
  }
}"#,
        );

        assert_eq!(status, GateStatus::Fail);
        assert_eq!(db_version.as_deref(), Some("2026-05-08"));
        assert_eq!(vulns.len(), 1);
        assert_eq!(vulns[0].package, "example");
    }

    #[test]
    fn parse_audit_output_warns_for_medium_vulnerabilities() {
        let (_vulns, _db_version, status) = parse_audit_output(
            r#"{
  "vulnerabilities": {
    "list": [
      {
        "advisory": { "id": "RUSTSEC-0000-0001", "severity": "medium", "title": "watch" },
        "package": { "name": "example" }
      }
    ]
  }
}"#,
        );

        assert_eq!(status, GateStatus::Warn);
    }

    #[test]
    fn parse_audit_output_passes_when_no_vulnerabilities_are_listed() {
        let (vulns, db_version, status) = parse_audit_output(
            r#"{
  "database": { "version": "2026-05-08" },
  "vulnerabilities": { "found": false, "count": 0, "list": [] }
}"#,
        );

        assert_eq!(status, GateStatus::Pass);
        assert_eq!(db_version.as_deref(), Some("2026-05-08"));
        assert!(vulns.is_empty());
    }

    #[test]
    fn parse_audit_output_marks_malformed_json_pending() {
        let (vulns, db_version, status) = parse_audit_output("not json");

        assert_eq!(status, GateStatus::Pending);
        assert!(db_version.is_none());
        assert!(vulns.is_empty());
    }
}
