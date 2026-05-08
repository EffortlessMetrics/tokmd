use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use tokmd_types::cockpit::*;

use super::is_relevant_rust_source;
use crate::FileStat;

/// Get the current HEAD commit hash.
#[cfg(feature = "git")]
fn get_head_commit(repo_root: &PathBuf) -> Result<String> {
    let output = tokmd_git::git_cmd()
        .arg("-C")
        .arg(repo_root)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .context("Failed to run git rev-parse HEAD")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git rev-parse HEAD failed: {}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// CI workflow summary format (mutants-summary.json).
#[derive(Debug, Clone, Deserialize)]
#[cfg(feature = "git")]
struct CiMutantsSummary {
    commit: String,
    status: String,
    scope: Vec<String>,
    survivors: Vec<CiSurvivor>,
    killed: usize,
    timeout: usize,
    unviable: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg(feature = "git")]
struct CiSurvivor {
    file: String,
    line: usize,
    mutation: String,
}

/// Compute the mutation gate status.
#[cfg(feature = "git")]
pub(super) fn compute_mutation_gate(
    repo_root: &PathBuf,
    _base: &str,
    _head: &str,
    changed_files: &[FileStat],
    _range_mode: tokmd_git::GitRangeMode,
) -> Result<MutationGate> {
    // Filter to relevant Rust source files
    let relevant_files: Vec<String> = changed_files
        .iter()
        .filter(|f| is_relevant_rust_source(&f.path))
        .map(|f| f.path.clone())
        .collect();

    // If no relevant files, skip
    if relevant_files.is_empty() {
        return Ok(MutationGate {
            meta: GateMeta {
                status: GateStatus::Skipped,
                source: EvidenceSource::RanLocal,
                commit_match: CommitMatch::Unknown,
                scope: ScopeCoverage {
                    relevant: Vec::new(),
                    tested: Vec::new(),
                    ratio: 1.0,
                    lines_relevant: None,
                    lines_tested: None,
                },
                evidence_commit: None,
                evidence_generated_at_ms: None,
            },
            survivors: Vec::new(),
            killed: 0,
            timeout: 0,
            unviable: 0,
        });
    }

    let head_commit = get_head_commit(repo_root)?;

    // Try to find cached results
    if let Some(gate) = try_load_ci_artifact(repo_root, &head_commit, &relevant_files)? {
        return Ok(gate);
    }

    if let Some(gate) = try_load_cached(repo_root, &head_commit, &relevant_files)? {
        return Ok(gate);
    }

    // Try to run mutations
    run_mutations(repo_root, &relevant_files)
}

/// Try to load mutation results from CI artifact.
/// Checks for mutants-summary.json (our format) first, then falls back to mutants.out/outcomes.json.
#[cfg(feature = "git")]
fn try_load_ci_artifact(
    repo_root: &Path,
    head_commit: &str,
    relevant_files: &[String],
) -> Result<Option<MutationGate>> {
    // First, check for our summary format (mutants-summary.json)
    let summary_path = repo_root.join("mutants-summary.json");
    if summary_path.exists()
        && let Ok(content) = std::fs::read_to_string(&summary_path)
        && let Ok(summary) = serde_json::from_str::<CiMutantsSummary>(&content)
    {
        // Determine commit match quality
        let commit_match = if summary.commit.starts_with(head_commit)
            || head_commit.starts_with(&summary.commit)
        {
            CommitMatch::Exact
        } else {
            CommitMatch::Stale
        };

        // Skip stale artifacts
        if commit_match == CommitMatch::Stale {
            return Ok(None);
        }

        let status = match summary.status.as_str() {
            "pass" => GateStatus::Pass,
            "fail" => GateStatus::Fail,
            "skipped" => GateStatus::Skipped,
            _ => GateStatus::Pending,
        };

        let survivors: Vec<MutationSurvivor> = summary
            .survivors
            .into_iter()
            .map(|s| MutationSurvivor {
                file: s.file,
                line: s.line,
                mutation: s.mutation,
            })
            .collect();

        let tested = summary.scope.clone();
        let scope_ratio = if relevant_files.is_empty() {
            1.0
        } else {
            tested.len() as f64 / relevant_files.len() as f64
        };

        let gate = MutationGate {
            meta: GateMeta {
                status,
                source: EvidenceSource::CiArtifact,
                commit_match,
                scope: ScopeCoverage {
                    relevant: relevant_files.to_vec(),
                    tested,
                    ratio: scope_ratio.min(1.0),
                    lines_relevant: None,
                    lines_tested: None,
                },
                evidence_commit: Some(summary.commit),
                evidence_generated_at_ms: None,
            },
            survivors,
            killed: summary.killed,
            timeout: summary.timeout,
            unviable: summary.unviable,
        };

        Ok(Some(gate))
    } else {
        Ok(None)
    }
}

/// Try to load cached mutation results.
#[cfg(feature = "git")]
fn try_load_cached(
    repo_root: &Path,
    head_commit: &str,
    relevant_files: &[String],
) -> Result<Option<MutationGate>> {
    const MUTANT_CACHE_DIR: &str = ".tokmd/cache/mutants";

    let cache_dir = repo_root.join(MUTANT_CACHE_DIR);
    if !cache_dir.exists() {
        return Ok(None);
    }

    let cache_file = cache_dir.join(cache_file_name_for_head(head_commit));
    if !cache_file.exists() {
        return Ok(None);
    }

    let gate = match std::fs::read_to_string(&cache_file)
        .ok()
        .and_then(|content| serde_json::from_str::<MutationGate>(&content).ok())
    {
        Some(gate) => gate,
        None => return Ok(None),
    };

    if cached_commit_mismatch(&gate, head_commit) {
        return Ok(None);
    }

    let tested = &gate.meta.scope.tested;
    if !relevant_files.iter().all(|file| tested.contains(file)) {
        return Ok(None);
    }

    Ok(Some(gate))
}

#[cfg(feature = "git")]
fn cache_file_name_for_head(head_commit: &str) -> String {
    format!("{head_commit}.json")
}

#[cfg(feature = "git")]
fn cached_commit_mismatch(gate: &MutationGate, head_commit: &str) -> bool {
    gate.meta
        .evidence_commit
        .as_deref()
        .is_some_and(|cached| cached != head_commit)
}

/// Run mutations locally.
#[cfg(feature = "git")]
fn run_mutations(_repo_root: &Path, relevant_files: &[String]) -> Result<MutationGate> {
    // This is expensive, so we only do it if explicitly asked or no other choice
    // For now, return Pending
    Ok(MutationGate {
        meta: GateMeta {
            status: GateStatus::Pending,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Exact,
            scope: ScopeCoverage {
                relevant: relevant_files.to_vec(),
                tested: Vec::new(),
                ratio: 0.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        survivors: Vec::new(),
        killed: 0,
        timeout: 0,
        unviable: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cached_mutation_gate(tested: Vec<String>, evidence_commit: Option<&str>) -> MutationGate {
        MutationGate {
            meta: GateMeta {
                status: GateStatus::Pass,
                source: EvidenceSource::Cached,
                commit_match: CommitMatch::Exact,
                scope: ScopeCoverage {
                    relevant: tested.clone(),
                    tested,
                    ratio: 1.0,
                    lines_relevant: None,
                    lines_tested: None,
                },
                evidence_commit: evidence_commit.map(str::to_string),
                evidence_generated_at_ms: None,
            },
            survivors: Vec::new(),
            killed: 1,
            timeout: 0,
            unviable: 0,
        }
    }

    fn write_mutant_cache(repo_root: &Path, head_commit: &str, body: &str) {
        let cache_dir = repo_root.join(".tokmd/cache/mutants");
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::write(cache_dir.join(cache_file_name_for_head(head_commit)), body).unwrap();
    }

    #[test]
    fn cache_hits_for_matching_commit_and_full_scope() {
        let dir = tempfile::tempdir().unwrap();
        let head = "abc123";
        let gate = cached_mutation_gate(vec!["src/lib.rs".into()], Some(head));
        write_mutant_cache(dir.path(), head, &serde_json::to_string(&gate).unwrap());

        let loaded = try_load_cached(dir.path(), head, &["src/lib.rs".into()])
            .unwrap()
            .expect("matching cache should load");

        assert_eq!(loaded.meta.source, EvidenceSource::Cached);
        assert_eq!(loaded.killed, 1);
    }

    #[test]
    fn cache_misses_for_partial_scope() {
        let dir = tempfile::tempdir().unwrap();
        let head = "abc123";
        let gate = cached_mutation_gate(vec!["src/lib.rs".into()], Some(head));
        write_mutant_cache(dir.path(), head, &serde_json::to_string(&gate).unwrap());

        let loaded = try_load_cached(
            dir.path(),
            head,
            &["src/lib.rs".into(), "src/new.rs".into()],
        )
        .unwrap();

        assert!(loaded.is_none());
    }

    #[test]
    fn cache_misses_for_mismatched_evidence_commit() {
        let dir = tempfile::tempdir().unwrap();
        let head = "abc123";
        let gate = cached_mutation_gate(vec!["src/lib.rs".into()], Some("def456"));
        write_mutant_cache(dir.path(), head, &serde_json::to_string(&gate).unwrap());

        let loaded = try_load_cached(dir.path(), head, &["src/lib.rs".into()]).unwrap();

        assert!(loaded.is_none());
    }

    #[test]
    fn cache_misses_for_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let head = "abc123";
        write_mutant_cache(dir.path(), head, "{");

        let loaded = try_load_cached(dir.path(), head, &["src/lib.rs".into()]).unwrap();

        assert!(loaded.is_none());
    }
}
