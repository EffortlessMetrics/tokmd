//! Diff-coverage gate.
//!
//! Orchestrates the three single-responsibility submodules:
//!
//! * [`artifact`] — locate the coverage report on disk.
//! * [`lcov`]     — parse its contents into a line-level lookup.
//! * [`intersect`] — combine the lookup with the diff's added lines and
//!   roll up totals + uncovered hunks.
//!
//! The orchestrator is the only place that talks to git, the filesystem, and
//! the rolled-up [`DiffCoverageGate`] result type.

use std::path::Path;

use anyhow::Result;
use tokmd_types::cockpit::*;

use crate::round_pct;

mod artifact;
mod intersect;
mod lcov;

const MAX_UNCOVERED_HUNKS: usize = 20;
const COVERAGE_PASS_THRESHOLD: f64 = 0.80;
const COVERAGE_WARN_THRESHOLD: f64 = 0.50;

/// Compute diff coverage gate.
///
/// Looks for coverage artifacts (lcov.info, coverage.json, cobertura.xml) and
/// parses them. Only LCOV is currently parsed; other formats short-circuit
/// to `Ok(None)`.
pub(in crate::gates) fn compute_diff_coverage_gate(
    repo_root: &Path,
    base: &str,
    head: &str,
    range_mode: tokmd_git::GitRangeMode,
) -> Result<Option<DiffCoverageGate>> {
    let added_lines = match tokmd_git::get_added_lines(repo_root, base, head, range_mode) {
        Ok(lines) if !lines.is_empty() => lines,
        Ok(_) => return Ok(None),
        Err(_) => return Ok(None),
    };

    let Some(lcov_path) = artifact::find_lcov_artifact(repo_root) else {
        return Ok(None);
    };

    let content = match std::fs::read_to_string(&lcov_path) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };

    let lcov_data = lcov::parse_lcov(repo_root, &content);

    let mut summary = intersect::intersect(&added_lines, &lcov_data);

    if summary.total_added == 0 {
        return Ok(None);
    }

    let coverage_pct = round_pct(summary.total_covered as f64 / summary.total_added as f64);
    let status = coverage_status(coverage_pct);

    // Cap the report size to keep the gate output manageable.
    summary.uncovered_hunks.truncate(MAX_UNCOVERED_HUNKS);

    Ok(Some(DiffCoverageGate {
        meta: GateMeta {
            status,
            source: EvidenceSource::CiArtifact,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: lcov_data.keys().cloned().collect(),
                tested: summary.tested_files.into_iter().collect(),
                ratio: coverage_pct,
                lines_relevant: Some(summary.total_added),
                lines_tested: Some(summary.total_covered),
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        lines_added: summary.total_added,
        lines_covered: summary.total_covered,
        coverage_pct,
        uncovered_hunks: summary.uncovered_hunks,
    }))
}

fn coverage_status(pct: f64) -> GateStatus {
    if pct >= COVERAGE_PASS_THRESHOLD {
        GateStatus::Pass
    } else if pct >= COVERAGE_WARN_THRESHOLD {
        GateStatus::Warn
    } else {
        GateStatus::Fail
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_status_thresholds() {
        assert_eq!(coverage_status(1.0), GateStatus::Pass);
        assert_eq!(coverage_status(0.80), GateStatus::Pass);
        assert_eq!(coverage_status(0.79), GateStatus::Warn);
        assert_eq!(coverage_status(0.50), GateStatus::Warn);
        assert_eq!(coverage_status(0.49), GateStatus::Fail);
        assert_eq!(coverage_status(0.0), GateStatus::Fail);
    }

    #[test]
    fn diff_coverage_gate_flushes_unterminated_final_lcov_record() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/lib.rs"), "fn a() {}\n").unwrap();

        let git = |args: &[&str]| {
            let status = tokmd_git::git_cmd()
                .args(args)
                .current_dir(dir.path())
                .status()
                .unwrap();
            assert!(status.success(), "git {:?} failed", args);
        };

        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "tokmd@example.com"]);
        git(&["config", "user.name", "tokmd"]);
        // Keep the test self-contained: a global gpgsign=true on the host
        // would otherwise refuse our commits.
        git(&["config", "commit.gpgsign", "false"]);
        git(&["add", "."]);
        git(&["commit", "-m", "base"]);

        std::fs::write(dir.path().join("src/lib.rs"), "fn a() {}\nfn b() {}\n").unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "head"]);

        std::fs::write(dir.path().join("lcov.info"), "SF:src/lib.rs\nDA:2,1\n").unwrap();

        let gate = compute_diff_coverage_gate(
            dir.path(),
            "HEAD~1",
            "HEAD",
            tokmd_git::GitRangeMode::TwoDot,
        )
        .unwrap()
        .expect("diff coverage gate should exist");

        assert_eq!(gate.coverage_pct, 1.0);
        assert_eq!(gate.meta.scope.lines_relevant, Some(1));
        assert_eq!(gate.meta.scope.lines_tested, Some(1));
    }
}
