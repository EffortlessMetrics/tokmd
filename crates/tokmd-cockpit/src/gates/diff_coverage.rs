use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
use tokmd_types::cockpit::*;

use crate::round_pct;

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
pub(super) fn compute_diff_coverage_gate(
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
    let mut tested_files: BTreeSet<String> = BTreeSet::new();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flush_uncovered_hunks_consecutive() {
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
    fn flush_uncovered_hunks_empty() {
        let mut hunks = Vec::new();
        flush_uncovered_hunks("test.rs", &[], &mut hunks);
        assert!(hunks.is_empty());
    }

    #[test]
    fn flush_uncovered_hunks_empty_file() {
        let mut hunks = Vec::new();
        flush_uncovered_hunks("", &[1, 2], &mut hunks);
        assert!(hunks.is_empty());
    }

    #[test]
    fn flush_uncovered_hunks_single_line() {
        let mut hunks = Vec::new();
        flush_uncovered_hunks("test.rs", &[42], &mut hunks);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].start_line, 42);
        assert_eq!(hunks[0].end_line, 42);
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
        git(&["config", "commit.gpgsign", "false"]);
        git(&["config", "tag.gpgsign", "false"]);
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
