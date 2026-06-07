use std::path::Path;

use anyhow::Result;
use crate::source_complexity::analyze_rust_function_complexity;
use tokmd_types::cockpit::*;

use super::rust_source::is_relevant_rust_source;
use crate::{COMPLEXITY_THRESHOLD, FileStat, round_pct};

/// Compute complexity gate.
/// Analyzes cyclomatic complexity of changed Rust source files.
#[cfg(feature = "git")]
pub(super) fn compute_complexity_gate(
    repo_root: &Path,
    changed_files: &[FileStat],
) -> Result<Option<ComplexityGate>> {
    // Filter to relevant Rust source files
    let relevant_files: Vec<String> = changed_files
        .iter()
        .filter(|f| is_relevant_rust_source(&f.path))
        .map(|f| f.path.clone())
        .collect();

    // If no relevant files, skip
    if relevant_files.is_empty() {
        return Ok(None);
    }

    let mut high_complexity_files = Vec::new();
    let mut total_complexity: u64 = 0;
    let mut max_cyclomatic: u32 = 0;
    let mut files_analyzed: usize = 0;

    for file_path in &relevant_files {
        let full_path = repo_root.join(file_path);
        if !full_path.exists() {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(&full_path) {
            let analysis = analyze_rust_function_complexity(&content);
            files_analyzed += 1;
            total_complexity += analysis.total_complexity as u64;
            max_cyclomatic = max_cyclomatic.max(analysis.max_complexity);

            if analysis.max_complexity > COMPLEXITY_THRESHOLD {
                high_complexity_files.push(HighComplexityFile {
                    path: file_path.clone(),
                    cyclomatic: analysis.max_complexity,
                    function_count: analysis.function_count,
                    max_function_length: analysis.max_function_length,
                });
            }
        }
    }

    // Sort high complexity files by cyclomatic complexity (descending), then path for determinism
    high_complexity_files.sort_by(|a, b| {
        b.cyclomatic
            .cmp(&a.cyclomatic)
            .then_with(|| a.path.cmp(&b.path))
    });

    let avg_cyclomatic = if files_analyzed > 0 {
        round_pct(total_complexity as f64 / files_analyzed as f64)
    } else {
        0.0
    };

    // Determine gate status:
    // - Pass: no high complexity files
    // - Warn (represented as Pending): 1-3 high complexity files
    // - Fail: >3 high complexity files
    let high_count = high_complexity_files.len();
    let (status, threshold_exceeded) = match high_count {
        0 => (GateStatus::Pass, false),
        1..=3 => (GateStatus::Warn, true),
        _ => (GateStatus::Fail, true),
    };

    Ok(Some(ComplexityGate {
        meta: GateMeta {
            status,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Exact,
            scope: ScopeCoverage {
                relevant: relevant_files.clone(),
                tested: relevant_files,
                ratio: 1.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            ),
        },
        files_analyzed,
        high_complexity_files,
        avg_cyclomatic,
        max_cyclomatic,
        threshold_exceeded,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stat(path: &str) -> FileStat {
        FileStat {
            path: path.to_string(),
            insertions: 1,
            deletions: 0,
        }
    }

    #[test]
    fn skips_when_no_relevant_rust_sources_changed() {
        let changed_files = vec![stat("README.md"), stat("tests/cockpit.rs")];

        let gate = compute_complexity_gate(Path::new("."), &changed_files).unwrap();

        assert!(gate.is_none());
    }

    #[test]
    fn analyzes_changed_rust_sources() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(
            dir.path().join("src/lib.rs"),
            r#"
fn maybe(flag: bool) {
    if flag {
        println!("yes");
    }
}
"#,
        )
        .unwrap();

        let changed_files = vec![stat("src/lib.rs")];

        let gate = compute_complexity_gate(dir.path(), &changed_files)
            .unwrap()
            .expect("changed Rust source should produce complexity gate");

        assert_eq!(gate.files_analyzed, 1);
        assert_eq!(gate.meta.status, GateStatus::Pass);
        assert_eq!(gate.meta.scope.relevant, vec!["src/lib.rs"]);
        assert_eq!(gate.meta.scope.tested, vec!["src/lib.rs"]);
        assert!(!gate.threshold_exceeded);
    }
}
