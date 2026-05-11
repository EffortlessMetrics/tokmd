//! Handoff intelligence artifact construction.

use std::collections::BTreeMap;

use tokmd_scan::normalize_slashes as normalize_path;
use tokmd_types::{
    CapabilityState, CapabilityStatus, ExportData, FileKind, FileRow, HandoffDerived,
    HandoffHotspot, HandoffIntelligence,
};

use crate::cli;

use super::capabilities::{capability_reason, capability_state};
use super::{DEFAULT_TREE_DEPTH, round_f64};

mod complexity;

use complexity::build_simple_complexity;

/// Build intelligence data for the handoff.
pub(super) fn build_intelligence(
    export: &ExportData,
    args: &cli::HandoffArgs,
    capabilities: &[CapabilityStatus],
    git_scores: Option<&tokmd_core::context_git::GitScores>,
) -> HandoffIntelligence {
    let mut warnings = Vec::new();

    // Build tree (always included)
    let tree = Some(tokmd_format::render_handoff_tree(
        export,
        DEFAULT_TREE_DEPTH,
    ));
    let tree_depth = tree.as_ref().map(|_| DEFAULT_TREE_DEPTH);

    // Build hotspots (Risk/Deep presets)
    let wants_hotspots = matches!(
        args.preset,
        cli::HandoffPreset::Risk | cli::HandoffPreset::Deep
    );
    let hotspots = if wants_hotspots {
        match git_scores {
            Some(scores) if !scores.hotspots.is_empty() => {
                let mut hotspot_rows: Vec<HandoffHotspot> = scores
                    .hotspots
                    .iter()
                    .map(|(path, &score)| {
                        let commits = scores.commit_counts.get(path).copied().unwrap_or(0);
                        let lines = export
                            .rows
                            .iter()
                            .find(|r| normalize_path(&r.path) == *path)
                            .map(|r| r.lines)
                            .unwrap_or(0);
                        HandoffHotspot {
                            path: path.clone(),
                            commits,
                            lines,
                            score,
                        }
                    })
                    .collect();
                // Sort by score descending, then by path
                hotspot_rows
                    .sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.path.cmp(&b.path)));
                // Limit to top 20
                hotspot_rows.truncate(20);
                Some(hotspot_rows)
            }
            _ => {
                let state = capability_state(capabilities, "git_history");
                if wants_hotspots {
                    let reason = capability_reason(capabilities, "git_history");
                    match state {
                        Some(CapabilityState::Available) => {
                            warnings.push("hotspots unavailable: no git history found".to_string());
                        }
                        Some(CapabilityState::Skipped) => {
                            let msg = if let Some(r) = reason {
                                format!("hotspots unavailable: git history skipped ({})", r)
                            } else {
                                "hotspots unavailable: git history skipped".to_string()
                            };
                            warnings.push(msg);
                        }
                        Some(CapabilityState::Unavailable) => {
                            let msg = if let Some(r) = reason {
                                format!("hotspots unavailable: git history unavailable ({})", r)
                            } else {
                                "hotspots unavailable: git history unavailable".to_string()
                            };
                            warnings.push(msg);
                        }
                        None => {}
                    }
                }
                None
            }
        }
    } else {
        None
    };

    // Build complexity (Standard/Risk/Deep presets)
    let complexity = if matches!(
        args.preset,
        cli::HandoffPreset::Standard | cli::HandoffPreset::Risk | cli::HandoffPreset::Deep
    ) {
        Some(build_simple_complexity(export))
    } else {
        None
    };

    // Build derived (Standard/Risk/Deep presets)
    let derived = if matches!(
        args.preset,
        cli::HandoffPreset::Standard | cli::HandoffPreset::Risk | cli::HandoffPreset::Deep
    ) {
        Some(build_simple_derived(export))
    } else {
        None
    };

    HandoffIntelligence {
        tree,
        tree_depth,
        hotspots,
        complexity,
        derived,
        warnings,
    }
}

/// Build simple derived metrics from export data.
fn build_simple_derived(export: &ExportData) -> HandoffDerived {
    let parents: Vec<&FileRow> = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .collect();

    let total_files = parents.len();
    let total_code: usize = parents.iter().map(|r| r.code).sum();
    let total_lines: usize = parents.iter().map(|r| r.lines).sum();
    let total_tokens: usize = parents.iter().map(|r| r.tokens).sum();

    // Count languages
    let mut lang_counts: BTreeMap<String, usize> = BTreeMap::new();
    for row in &parents {
        *lang_counts.entry(row.lang.clone()).or_insert(0) += row.code;
    }
    let lang_count = lang_counts.len();

    // Find dominant language
    let (dominant_lang, dominant_code) = lang_counts
        .iter()
        .max_by_key(|(_, code)| *code)
        .map(|(lang, code)| (lang.clone(), *code))
        .unwrap_or_else(|| ("Unknown".to_string(), 0));

    let dominant_pct = if total_code > 0 {
        (dominant_code as f64 / total_code as f64) * 100.0
    } else {
        0.0
    };

    HandoffDerived {
        total_files,
        total_code,
        total_lines,
        total_tokens,
        lang_count,
        dominant_lang,
        dominant_pct: round_f64(dominant_pct, 2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_simple_derived_empty() {
        let export = ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 2,
            children: tokmd_types::ChildIncludeMode::ParentsOnly,
        };
        let derived = build_simple_derived(&export);
        assert_eq!(derived.total_files, 0);
        assert_eq!(derived.total_code, 0);
        assert_eq!(derived.lang_count, 0);
    }
}
