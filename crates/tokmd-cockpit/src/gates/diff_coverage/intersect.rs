//! Intersection of git-added lines with parsed LCOV coverage.
//!
//! Given the set of lines a diff introduced and a parsed LCOV map, produces
//! the totals plus the contiguous runs of *uncovered* lines (hunks) we want
//! to report.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use tokmd_types::cockpit::UncoveredHunk;

use super::lcov::LcovData;

/// Per-file totals and uncovered hunks for the diff under inspection.
pub(super) struct Intersection {
    pub total_added: usize,
    pub total_covered: usize,
    pub tested_files: BTreeSet<String>,
    pub uncovered_hunks: Vec<UncoveredHunk>,
}

/// Intersect each file's added lines with its LCOV record.
///
/// Files missing from LCOV contribute all their added lines as uncovered.
pub(super) fn intersect(
    added_lines: &BTreeMap<PathBuf, BTreeSet<usize>>,
    lcov_data: &LcovData,
) -> Intersection {
    let mut total_added = 0usize;
    let mut total_covered = 0usize;
    let mut uncovered_hunks: Vec<UncoveredHunk> = Vec::new();
    let mut tested_files: BTreeSet<String> = BTreeSet::new();

    for (file_path, lines) in added_lines {
        let file_path_str = file_path.to_string_lossy().replace('\\', "/");
        total_added += lines.len();

        let uncovered_in_file =
            partition_file(lines, lcov_data.get(&file_path_str), &mut total_covered);
        if lcov_data.contains_key(&file_path_str) {
            tested_files.insert(file_path_str.clone());
        }

        flush_uncovered_hunks(&file_path_str, &uncovered_in_file, &mut uncovered_hunks);
    }

    Intersection {
        total_added,
        total_covered,
        tested_files,
        uncovered_hunks,
    }
}

/// Split a file's added lines into covered vs. uncovered.
///
/// Increments `total_covered` for each line with a hit > 0; returns the
/// uncovered set in ascending order.
fn partition_file(
    added: &BTreeSet<usize>,
    file_lcov: Option<&BTreeMap<usize, usize>>,
    total_covered: &mut usize,
) -> Vec<usize> {
    let Some(file_lcov) = file_lcov else {
        // Whole file is absent from LCOV → every added line is uncovered.
        return added.iter().copied().collect();
    };

    let mut uncovered = Vec::new();
    for &line in added {
        match file_lcov.get(&line) {
            Some(&count) if count > 0 => *total_covered += 1,
            _ => uncovered.push(line),
        }
    }
    uncovered
}

/// Coalesce consecutive uncovered line numbers into hunks and append them.
pub(super) fn flush_uncovered_hunks(
    file: &str,
    uncovered: &[usize],
    hunks: &mut Vec<UncoveredHunk>,
) {
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
    fn intersect_marks_missing_files_as_uncovered() {
        let mut added = BTreeMap::new();
        added.insert(PathBuf::from("src/lib.rs"), BTreeSet::from([1, 2, 3]));
        let lcov: LcovData = BTreeMap::new();

        let result = intersect(&added, &lcov);
        assert_eq!(result.total_added, 3);
        assert_eq!(result.total_covered, 0);
        assert!(result.tested_files.is_empty());
        assert_eq!(result.uncovered_hunks.len(), 1);
        assert_eq!(result.uncovered_hunks[0].file, "src/lib.rs");
        assert_eq!(result.uncovered_hunks[0].start_line, 1);
        assert_eq!(result.uncovered_hunks[0].end_line, 3);
    }

    #[test]
    fn intersect_splits_covered_and_uncovered() {
        let mut added = BTreeMap::new();
        added.insert(PathBuf::from("src/lib.rs"), BTreeSet::from([1, 2, 3, 4]));
        let mut file_cov: BTreeMap<usize, usize> = BTreeMap::new();
        file_cov.insert(1, 1);
        file_cov.insert(2, 0); // present but unhit → uncovered
        file_cov.insert(3, 5);
        // line 4 absent → uncovered, non-adjacent to line 2 → separate hunk
        let mut lcov: LcovData = BTreeMap::new();
        lcov.insert("src/lib.rs".to_string(), file_cov);

        let result = intersect(&added, &lcov);
        assert_eq!(result.total_added, 4);
        assert_eq!(result.total_covered, 2);
        assert_eq!(result.tested_files.len(), 1);
        assert_eq!(result.uncovered_hunks.len(), 2);
        assert_eq!(result.uncovered_hunks[0].start_line, 2);
        assert_eq!(result.uncovered_hunks[0].end_line, 2);
        assert_eq!(result.uncovered_hunks[1].start_line, 4);
        assert_eq!(result.uncovered_hunks[1].end_line, 4);
    }
}
