use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use tokmd_types::cockpit::*;

mod complexity;
mod mutation;

use complexity::compute_complexity_gate;
use mutation::compute_mutation_gate;

use crate::determinism;
use crate::supply_chain::compute_supply_chain_gate;
use crate::{FileStat, round_pct};

// =============================================================================
// Evidence computation
// =============================================================================

/// Compute evidence section with all gates.
#[cfg(feature = "git")]
pub(crate) fn compute_evidence(
    repo_root: &PathBuf,
    base: &str,
    head: &str,
    changed_files: &[FileStat],
    contracts_info: &Contracts,
    range_mode: tokmd_git::GitRangeMode,
    baseline_path: Option<&Path>,
) -> Result<Evidence> {
    let mutation = compute_mutation_gate(repo_root, base, head, changed_files, range_mode)?;
    let diff_coverage = compute_diff_coverage_gate(repo_root, base, head, range_mode)?;
    let contracts = compute_contract_gate(repo_root, base, head, changed_files, contracts_info)?;
    let supply_chain = compute_supply_chain_gate(repo_root, changed_files)?;
    let determinism = compute_determinism_gate(repo_root, baseline_path)?;
    let complexity = compute_complexity_gate(repo_root, changed_files)?;

    // Compute overall status: any Fail -> Fail, all Pass -> Pass, otherwise Pending/Skipped
    let overall_status = compute_overall_status(
        &mutation,
        &diff_coverage,
        &contracts,
        &supply_chain,
        &determinism,
        &complexity,
    );

    Ok(Evidence {
        overall_status,
        mutation,
        diff_coverage,
        contracts,
        supply_chain,
        determinism,
        complexity,
    })
}

/// Compute overall status from all gates.
#[cfg(feature = "git")]
fn compute_overall_status(
    mutation: &MutationGate,
    diff_coverage: &Option<DiffCoverageGate>,
    contracts: &Option<ContractDiffGate>,
    supply_chain: &Option<SupplyChainGate>,
    determinism: &Option<DeterminismGate>,
    complexity: &Option<ComplexityGate>,
) -> GateStatus {
    let statuses: Vec<GateStatus> = [
        Some(mutation.meta.status),
        diff_coverage.as_ref().map(|g| g.meta.status),
        contracts.as_ref().map(|g| g.meta.status),
        supply_chain.as_ref().map(|g| g.meta.status),
        determinism.as_ref().map(|g| g.meta.status),
        complexity.as_ref().map(|g| g.meta.status),
    ]
    .into_iter()
    .flatten()
    .collect();

    if statuses.is_empty() {
        return GateStatus::Skipped;
    }

    // Any Fail -> overall Fail
    if statuses.contains(&GateStatus::Fail) {
        return GateStatus::Fail;
    }

    // All Pass -> overall Pass
    if statuses.iter().all(|s| *s == GateStatus::Pass) {
        return GateStatus::Pass;
    }

    // Any Pending (and no Fail) -> overall Pending
    if statuses.contains(&GateStatus::Pending) {
        return GateStatus::Pending;
    }

    // Any Warn (and no Fail/Pending) -> overall Warn
    if statuses.contains(&GateStatus::Warn) {
        return GateStatus::Warn;
    }

    // All Skipped -> Skipped; mix of Pass and Skipped -> Pass
    if statuses.iter().all(|s| *s == GateStatus::Skipped) {
        GateStatus::Skipped
    } else {
        GateStatus::Pass
    }
}

// =============================================================================
// Diff coverage gate
// =============================================================================

#[cfg(feature = "git")]
fn merge_lcov_record(
    lcov_data: &mut BTreeMap<String, BTreeMap<usize, usize>>,
    file: String,
    lines: BTreeMap<usize, usize>,
) {
    match lcov_data.entry(file) {
        std::collections::btree_map::Entry::Occupied(mut entry) => {
            entry.get_mut().extend(lines);
        }
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(lines);
        }
    }
}

/// Compute diff coverage gate.
/// Looks for coverage artifacts (lcov.info, coverage.json, cobertura.xml) and parses them.
#[cfg(feature = "git")]
fn compute_diff_coverage_gate(
    repo_root: &Path,
    base: &str,
    head: &str,
    range_mode: tokmd_git::GitRangeMode,
) -> Result<Option<DiffCoverageGate>> {
    // 1. Get added lines from git
    let added_lines = match tokmd_git::get_added_lines(repo_root, base, head, range_mode) {
        Ok(lines) => lines,
        Err(_) => return Ok(None),
    };

    if added_lines.is_empty() {
        return Ok(None);
    }

    // 2. Search for coverage artifacts in common locations
    let search_paths = [
        "coverage/lcov.info",
        "target/coverage/lcov.info",
        "lcov.info",
        "coverage/cobertura.xml",
        "target/coverage/cobertura.xml",
        "cobertura.xml",
        "coverage/coverage.json",
        "target/coverage/coverage.json",
        "coverage.json",
    ];

    let mut lcov_path: Option<PathBuf> = None;
    for candidate in &search_paths {
        let path = repo_root.join(candidate);
        if path.exists() {
            lcov_path = Some(path);
            break;
        }
    }

    let lcov_path = match lcov_path {
        Some(p) => p,
        None => return Ok(None), // No coverage artifact found
    };

    // Only parse lcov.info format for now (most common in Rust via cargo-llvm-cov)
    let path_str = lcov_path.to_string_lossy();
    if !path_str.ends_with("lcov.info") {
        // We found a coverage file but can't parse non-lcov yet
        return Ok(None);
    }

    let content = match std::fs::read_to_string(&lcov_path) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };

    // 3. Parse LCOV into a lookup map: file -> line -> hit_count
    let mut lcov_data: BTreeMap<String, BTreeMap<usize, usize>> = BTreeMap::new();
    let mut current_file: Option<String> = None;
    let mut current_lines = BTreeMap::new();

    for line in content.lines() {
        if let Some(sf) = line.strip_prefix("SF:") {
            // Normalize path to repo-relative
            let path = sf.replace('\\', "/");
            // If it's absolute, try to make it relative to repo root
            let normalized = if let Ok(abs) = Path::new(&path).canonicalize() {
                if let Ok(rel) = abs.strip_prefix(repo_root.canonicalize().unwrap_or_default()) {
                    rel.to_string_lossy().replace('\\', "/")
                } else {
                    path
                }
            } else {
                path
            };
            current_file = Some(normalized);
            current_lines.clear();
        } else if let Some(da) = line.strip_prefix("DA:") {
            if current_file.is_some() {
                let parts: Vec<&str> = da.splitn(2, ',').collect();
                if parts.len() == 2
                    && let (Ok(line_no), Ok(count)) =
                        (parts[0].parse::<usize>(), parts[1].parse::<usize>())
                {
                    current_lines.insert(line_no, count);
                }
            }
        } else if line == "end_of_record"
            && let Some(file) = current_file.take()
        {
            let lines = std::mem::take(&mut current_lines);
            merge_lcov_record(&mut lcov_data, file, lines);
        }
    }

    if let Some(file) = current_file.take() {
        let lines = std::mem::take(&mut current_lines);
        merge_lcov_record(&mut lcov_data, file, lines);
    }

    // 4. Intersect added lines with LCOV hits
    let mut total_added = 0usize;
    let mut total_covered = 0usize;
    let mut uncovered_hunks: Vec<UncoveredHunk> = Vec::new();
    let mut tested_files: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for (file_path, lines) in added_lines {
        let file_path_str = file_path.to_string_lossy().replace('\\', "/");
        total_added += lines.len();

        let mut uncovered_in_file = Vec::new();

        if let Some(file_lcov) = lcov_data.get(&file_path_str) {
            tested_files.insert(file_path_str.clone());
            for line in lines {
                match file_lcov.get(&line) {
                    Some(&count) if count > 0 => {
                        total_covered += 1;
                    }
                    _ => {
                        uncovered_in_file.push(line);
                    }
                }
            }
        } else {
            // File not in LCOV - treat all added lines as uncovered
            uncovered_in_file.extend(lines);
        }

        flush_uncovered_hunks(&file_path_str, &uncovered_in_file, &mut uncovered_hunks);
    }

    if total_added == 0 {
        return Ok(None);
    }

    let coverage_pct = round_pct(total_covered as f64 / total_added as f64);
    let status = if coverage_pct >= 0.80 {
        GateStatus::Pass
    } else if coverage_pct >= 0.50 {
        GateStatus::Warn
    } else {
        GateStatus::Fail
    };

    // Limit uncovered hunks to avoid huge output
    uncovered_hunks.truncate(20);

    Ok(Some(DiffCoverageGate {
        meta: GateMeta {
            status,
            source: EvidenceSource::CiArtifact,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: lcov_data.keys().cloned().collect(),
                tested: tested_files.into_iter().collect(),
                ratio: coverage_pct,
                lines_relevant: Some(total_added),
                lines_tested: Some(total_covered),
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        lines_added: total_added,
        lines_covered: total_covered,
        coverage_pct,
        uncovered_hunks,
    }))
}

/// Flush consecutive uncovered lines into hunk ranges.
#[cfg(feature = "git")]
fn flush_uncovered_hunks(file: &str, uncovered: &[usize], hunks: &mut Vec<UncoveredHunk>) {
    if uncovered.is_empty() || file.is_empty() {
        return;
    }
    let mut sorted = uncovered.to_vec();
    sorted.sort_unstable();
    let mut start = sorted[0];
    let mut end = sorted[0];
    for &line in &sorted[1..] {
        if line == end + 1 {
            end = line;
        } else {
            hunks.push(UncoveredHunk {
                file: file.to_string(),
                start_line: start,
                end_line: end,
            });
            start = line;
            end = line;
        }
    }
    hunks.push(UncoveredHunk {
        file: file.to_string(),
        start_line: start,
        end_line: end,
    });
}

// =============================================================================
// Contract gate
// =============================================================================

/// Compute contract diff gate (semver, CLI, schema).
#[cfg(feature = "git")]
fn compute_contract_gate(
    repo_root: &Path,
    base: &str,
    head: &str,
    changed_files: &[FileStat],
    contracts_info: &Contracts,
) -> Result<Option<ContractDiffGate>> {
    // Only compute if any contract-relevant files changed
    if !contracts_info.api_changed && !contracts_info.cli_changed && !contracts_info.schema_changed
    {
        return Ok(None);
    }

    let mut failures = 0;
    let mut semver = None;
    let mut cli = None;
    let mut schema = None;

    // Check for semver changes (API files)
    if contracts_info.api_changed {
        semver = Some(run_semver_check(repo_root));
    }

    // Check for CLI changes
    if contracts_info.cli_changed {
        // Gather CLI-related files that changed
        let cli_files: Vec<&str> = changed_files
            .iter()
            .filter(|f| {
                f.path.contains("crates/tokmd/src/commands/")
                    || f.path.contains("crates/tokmd/src/cli/")
                    || f.path == "crates/tokmd/src/config.rs"
            })
            .map(|s| s.path.as_str())
            .collect();

        let diff_summary = if cli_files.is_empty() {
            None
        } else {
            let command_files = cli_files
                .iter()
                .filter(|f| f.contains("crates/tokmd/src/commands/"))
                .count();
            let config_files = cli_files
                .iter()
                .filter(|f| {
                    f.contains("crates/tokmd/src/cli/") || **f == "crates/tokmd/src/config.rs"
                })
                .count();

            let mut parts = Vec::new();
            if command_files > 0 {
                parts.push(format!(
                    "{} command file{}",
                    command_files,
                    if command_files == 1 { "" } else { "s" }
                ));
            }
            if config_files > 0 {
                parts.push(format!(
                    "{} config file{}",
                    config_files,
                    if config_files == 1 { "" } else { "s" }
                ));
            }
            Some(parts.join(", "))
        };

        cli = Some(CliSubGate {
            status: GateStatus::Pass,
            diff_summary,
        });
    }

    // Check for schema changes
    if contracts_info.schema_changed {
        schema = Some(run_schema_diff(repo_root, base, head));
    }

    // Count failures from sub-gates
    if let Some(ref sg) = semver
        && sg.status == GateStatus::Fail
    {
        failures += 1;
    }
    if let Some(ref cg) = cli
        && cg.status == GateStatus::Fail
    {
        failures += 1;
    }
    if let Some(ref scg) = schema
        && scg.status == GateStatus::Fail
    {
        failures += 1;
    }

    // Determine overall status
    let status = if failures > 0 {
        GateStatus::Fail
    } else {
        // Check if any are pending
        let any_pending = [
            semver.as_ref().map(|g| g.status),
            cli.as_ref().map(|g| g.status),
            schema.as_ref().map(|g| g.status),
        ]
        .into_iter()
        .flatten()
        .any(|s| s == GateStatus::Pending);

        if any_pending {
            GateStatus::Pending
        } else {
            GateStatus::Pass
        }
    };

    // Collect relevant files for scope
    let relevant: Vec<String> = changed_files
        .iter()
        .filter(|f| {
            f.path.ends_with("/src/lib.rs")
                || f.path.ends_with("/mod.rs")
                || f.path.contains("crates/tokmd/src/commands/")
                || f.path.contains("crates/tokmd/src/cli/")
                || f.path == "crates/tokmd/src/config.rs"
                || f.path == "docs/schema.json"
        })
        .map(|f| f.path.clone())
        .collect();

    Ok(Some(ContractDiffGate {
        meta: GateMeta {
            status,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: relevant.clone(),
                tested: relevant,
                ratio: 1.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        semver,
        cli,
        schema,
        failures,
    }))
}

/// Run cargo-semver-checks if available.
/// Returns a SemverSubGate with the result.
#[cfg(feature = "git")]
fn run_semver_check(repo_root: &Path) -> SemverSubGate {
    // Check if cargo-semver-checks is available
    let available = Command::new("cargo")
        .args(["semver-checks", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !available {
        return SemverSubGate {
            status: GateStatus::Pending,
            breaking_changes: Vec::new(),
        };
    }

    // Run cargo semver-checks
    let output = match Command::new("cargo")
        .args(["semver-checks", "check-release"])
        .current_dir(repo_root)
        .output()
    {
        Ok(o) => o,
        Err(_) => {
            return SemverSubGate {
                status: GateStatus::Pending,
                breaking_changes: Vec::new(),
            };
        }
    };

    if output.status.success() {
        // Exit 0 = no breaking changes
        return SemverSubGate {
            status: GateStatus::Pass,
            breaking_changes: Vec::new(),
        };
    }

    // Non-zero exit = breaking changes found
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);

    // Parse breaking changes from output lines
    // cargo-semver-checks output format: "--- failure[kind]: message ---" or similar
    let mut breaking_changes: Vec<BreakingChange> = Vec::new();
    for line in combined.lines() {
        let trimmed = line.trim();
        if trimmed.contains("BREAKING") || trimmed.starts_with("---") {
            breaking_changes.push(BreakingChange {
                kind: "semver".to_string(),
                path: String::new(),
                message: trimmed.to_string(),
            });
        }
    }

    // If we couldn't parse specific changes but the tool failed, add a generic entry
    if breaking_changes.is_empty() {
        breaking_changes.push(BreakingChange {
            kind: "semver".to_string(),
            path: String::new(),
            message: "cargo-semver-checks reported breaking changes".to_string(),
        });
    }

    // Limit output
    breaking_changes.truncate(20);

    SemverSubGate {
        status: GateStatus::Fail,
        breaking_changes,
    }
}

/// Run git diff on docs/schema.json to detect schema changes.
/// Returns a SchemaSubGate with the result.
#[cfg(feature = "git")]
fn run_schema_diff(repo_root: &Path, base: &str, head: &str) -> SchemaSubGate {
    // Use two-dot syntax for comparing refs directly (per project convention)
    let range = format!("{}..{}", base, head);
    let output = match tokmd_git::git_cmd()
        .arg("-C")
        .arg(repo_root)
        .args(["diff", &range, "--", "docs/schema.json"])
        .output()
    {
        Ok(o) => o,
        Err(_) => {
            return SchemaSubGate {
                status: GateStatus::Pending,
                diff_summary: None,
            };
        }
    };

    if !output.status.success() {
        return SchemaSubGate {
            status: GateStatus::Pending,
            diff_summary: None,
        };
    }

    let diff = String::from_utf8_lossy(&output.stdout);
    if diff.trim().is_empty() {
        // No diff means schema.json didn't change between these refs
        return SchemaSubGate {
            status: GateStatus::Pass,
            diff_summary: None,
        };
    }

    // Analyze the diff for breaking vs additive changes
    let mut additions = 0usize;
    let mut removals = 0usize;
    let mut has_type_change = false;

    for line in diff.lines() {
        if line.starts_with('+') && !line.starts_with("+++") {
            additions += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            removals += 1;
            // Check for type changes (field type modifications)
            let trimmed = line.trim_start_matches('-').trim();
            if trimmed.contains("\"type\"") {
                has_type_change = true;
            }
        }
    }

    let (status, summary) = if removals == 0 {
        // Only additions = safe additive change
        (
            GateStatus::Pass,
            Some(format!(
                "schema.json: {} line{} added (additive only)",
                additions,
                if additions == 1 { "" } else { "s" }
            )),
        )
    } else if has_type_change || removals > additions {
        // Type changes or net removals = likely breaking
        (
            GateStatus::Fail,
            Some(format!(
                "schema.json: {} addition{}, {} removal{} (potential breaking change)",
                additions,
                if additions == 1 { "" } else { "s" },
                removals,
                if removals == 1 { "" } else { "s" }
            )),
        )
    } else {
        // Removals but mostly additions = warn
        (
            GateStatus::Pass,
            Some(format!(
                "schema.json: {} addition{}, {} removal{}",
                additions,
                if additions == 1 { "" } else { "s" },
                removals,
                if removals == 1 { "" } else { "s" }
            )),
        )
    };

    SchemaSubGate {
        status,
        diff_summary: summary,
    }
}

// =============================================================================
// Determinism gate
// =============================================================================

/// Compute determinism gate.
/// Compares expected source hash (from baseline) with a fresh hash of the repo.
#[cfg(feature = "git")]
pub fn compute_determinism_gate(
    repo_root: &Path,
    baseline_path: Option<&Path>,
) -> Result<Option<DeterminismGate>> {
    use tokmd_analysis_types::ComplexityBaseline;

    fn short16(s: &str) -> &str {
        s.get(..16).unwrap_or(s)
    }

    // Resolve baseline: explicit path or default location
    let resolved_path = match baseline_path {
        Some(p) => p.to_path_buf(),
        None => repo_root.join(".tokmd/baseline.json"),
    };

    // If no baseline file exists, skip the gate
    if !resolved_path.exists() {
        return Ok(None);
    }

    // Parse baseline
    let content = std::fs::read_to_string(&resolved_path)
        .with_context(|| format!("failed to read baseline at {}", resolved_path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&content).with_context(|| {
        format!(
            "failed to parse baseline JSON at {}",
            resolved_path.display()
        )
    })?;
    let baseline: ComplexityBaseline = match serde_json::from_value(json.clone()) {
        Ok(parsed) => parsed,
        Err(_) => {
            // Allow cockpit receipts for trend comparison; determinism data is unavailable there.
            let mode = json
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if mode == "cockpit" {
                return Ok(None);
            }
            bail!(
                "baseline JSON at {} is not a ComplexityBaseline (and not a cockpit receipt)",
                resolved_path.display()
            );
        }
    };

    // If baseline has no determinism section, skip the gate
    let det = match &baseline.determinism {
        Some(d) => d,
        None => return Ok(None),
    };

    // Recompute current source hash by walking the repo, excluding the baseline file itself
    let baseline_rel = resolved_path
        .strip_prefix(repo_root)
        .ok()
        .map(|p| p.to_string_lossy().replace('\\', "/"));
    let exclude: Vec<&str> = baseline_rel.as_deref().into_iter().collect();
    let actual_hash = determinism::hash_files_from_walk(repo_root, &exclude)?;
    let expected_hash = &det.source_hash;

    let mut differences = Vec::new();

    if actual_hash != *expected_hash {
        differences.push(format!(
            "source hash mismatch: expected {}, got {}",
            short16(expected_hash),
            short16(&actual_hash),
        ));
    }

    // Check Cargo.lock hash if baseline had one
    if let Some(expected_lock) = &det.cargo_lock_hash {
        let actual_lock = determinism::hash_cargo_lock(repo_root)?;
        match actual_lock {
            Some(ref actual) if actual != expected_lock => {
                differences.push(format!(
                    "Cargo.lock hash mismatch: expected {}, got {}",
                    short16(expected_lock),
                    short16(actual),
                ));
            }
            None => {
                differences.push("Cargo.lock missing (was present in baseline)".to_string());
            }
            _ => {}
        }
    }

    let status = if differences.is_empty() {
        GateStatus::Pass
    } else {
        GateStatus::Warn
    };

    Ok(Some(DeterminismGate {
        meta: GateMeta {
            status,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: vec!["source files".to_string()],
                tested: vec!["source files".to_string()],
                ratio: 1.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        expected_hash: Some(expected_hash.clone()),
        actual_hash: Some(actual_hash),
        algo: "blake3".to_string(),
        differences,
    }))
}

/// Check if a file is a relevant Rust source file for mutation testing.
/// Excludes test files, fuzz targets, etc.
#[cfg(feature = "git")]
pub(super) fn is_relevant_rust_source(path: &str) -> bool {
    let path_lower = path.to_lowercase();

    // Must be a .rs file
    if !path_lower.ends_with(".rs") {
        return false;
    }

    // Exclude test directories
    if path_lower.contains("/tests/") || path_lower.starts_with("tests/") {
        return false;
    }

    // Exclude test files
    if path_lower.ends_with("_test.rs") || path_lower.ends_with("_tests.rs") {
        return false;
    }

    // Exclude fuzz targets
    if path_lower.contains("/fuzz/") || path_lower.starts_with("fuzz/") {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flush_uncovered_hunks_consecutive() {
        let mut hunks = Vec::new();
        flush_uncovered_hunks("test.rs", &[1, 2, 3, 5, 6, 10], &mut hunks);
        assert_eq!(hunks.len(), 3);
        assert_eq!(hunks[0].start_line, 1);
        assert_eq!(hunks[0].end_line, 3);
        assert_eq!(hunks[1].start_line, 5);
        assert_eq!(hunks[1].end_line, 6);
        assert_eq!(hunks[2].start_line, 10);
        assert_eq!(hunks[2].end_line, 10);
    }

    #[test]
    fn test_flush_uncovered_hunks_empty() {
        let mut hunks = Vec::new();
        flush_uncovered_hunks("test.rs", &[], &mut hunks);
        assert!(hunks.is_empty());
    }

    #[test]
    fn test_flush_uncovered_hunks_empty_file() {
        let mut hunks = Vec::new();
        flush_uncovered_hunks("", &[1, 2], &mut hunks);
        assert!(hunks.is_empty());
    }

    #[test]
    fn test_flush_uncovered_hunks_single_line() {
        let mut hunks = Vec::new();
        flush_uncovered_hunks("test.rs", &[42], &mut hunks);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].start_line, 42);
        assert_eq!(hunks[0].end_line, 42);
    }

    #[test]
    fn test_diff_coverage_gate_flushes_unterminated_final_lcov_record() {
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
