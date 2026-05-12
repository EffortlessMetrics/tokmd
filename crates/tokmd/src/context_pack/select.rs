//! File selection algorithms for LLM context packing.

use crate::cli::{ContextStrategy, ValueMetric};
use tokmd_core::context_git::GitScores;
use tokmd_core::context_policy::{
    assign_policy as assign_context_policy, classify_file as classify_context_file,
    compute_file_cap as compute_context_file_cap, is_spine_file as matches_spine_file,
    smart_exclude_reason,
};
use tokmd_scan::normalize_slashes as normalize_path;
use tokmd_types::{
    ContextFileRow, FileClassification, FileKind, FileRow, InclusionPolicy, PolicyExcludedFile,
    SmartExcludedFile,
};

mod pack;
mod policy;

use pack::to_context_row_with_reason;
#[cfg(test)]
use pack::{get_value, to_context_row};
pub use pack::{pack_greedy, pack_spread};

/// Check if a path should be smart-excluded. Returns the reason if excluded.
pub fn is_smart_excluded(path: &str) -> Option<&'static str> {
    smart_exclude_reason(path)
}

// ---------------------------------------------------------------------------
// Spine reservation: must-include files get a reserved budget fraction.
// ---------------------------------------------------------------------------

/// Fraction of total budget reserved for spine files.
const SPINE_BUDGET_FRACTION: f64 = 0.05;
/// Maximum tokens reserved for spine files.
const SPINE_BUDGET_CAP: usize = 5000;

fn is_spine_file(path: &str) -> bool {
    matches_spine_file(path)
}

/// Options for file selection with smart excludes and spine reservation.
#[allow(dead_code)]
pub struct SelectOptions {
    pub no_smart_exclude: bool,
    /// Maximum fraction of budget a single file may consume (default 0.15).
    pub max_file_pct: f64,
    /// Hard cap on tokens per file (default: None → computed as min(16_000, budget * pct)).
    pub max_file_tokens: Option<usize>,
    /// Error if git scores are unavailable and a git-based metric is requested.
    pub require_git_scores: bool,
    /// Tokens-per-line threshold above which a file is classified as DataBlob (default 50.0).
    pub dense_threshold: f64,
}

impl Default for SelectOptions {
    fn default() -> Self {
        Self {
            no_smart_exclude: false,
            max_file_pct: 0.15,
            max_file_tokens: None,
            require_git_scores: false,
            dense_threshold: 50.0,
        }
    }
}

/// Result of file selection including smart-excluded files.
pub struct SelectResult {
    pub selected: Vec<ContextFileRow>,
    pub smart_excluded: Vec<SmartExcludedFile>,
    /// Files excluded by per-file cap / classification policy.
    pub excluded_by_policy: Vec<PolicyExcludedFile>,
    /// Effective ranking metric used (may differ from requested if fallback occurred).
    pub rank_by_effective: String,
    /// Reason for fallback if the effective metric differs from the requested one.
    pub fallback_reason: Option<String>,
}

// ---------------------------------------------------------------------------
// File classification
// ---------------------------------------------------------------------------

/// Classify a file based on its path and density heuristic.
pub fn classify_file(
    path: &str,
    tokens: usize,
    lines: usize,
    dense_threshold: f64,
) -> Vec<FileClassification> {
    classify_context_file(path, tokens, lines, dense_threshold)
}

// ---------------------------------------------------------------------------
// Metric resolution with fallback tracking
// ---------------------------------------------------------------------------

/// Result of resolving a ranking metric, with fallback info.
pub struct ResolvedMetric {
    pub effective: ValueMetric,
    pub fallback_reason: Option<String>,
}

/// Resolve the requested ranking metric, falling back to Code if git scores are unavailable.
pub fn resolve_metric(requested: ValueMetric, git_scores: Option<&GitScores>) -> ResolvedMetric {
    match requested {
        ValueMetric::Hotspot if git_scores.is_none() => ResolvedMetric {
            effective: ValueMetric::Code,
            fallback_reason: Some(
                "hotspot requires git scores; falling back to code lines".to_string(),
            ),
        },
        ValueMetric::Churn if git_scores.is_none() => ResolvedMetric {
            effective: ValueMetric::Code,
            fallback_reason: Some(
                "churn requires git scores; falling back to code lines".to_string(),
            ),
        },
        _ => ResolvedMetric {
            effective: requested,
            fallback_reason: None,
        },
    }
}

// ---------------------------------------------------------------------------
// Per-file cap and policy assignment
// ---------------------------------------------------------------------------

/// Compute the maximum tokens a single file may consume.
pub fn compute_file_cap(budget: usize, options: &SelectOptions) -> usize {
    compute_context_file_cap(budget, options.max_file_pct, options.max_file_tokens)
}

/// Assign an inclusion policy to a file based on its size and classifications.
pub fn assign_policy(
    tokens: usize,
    file_cap: usize,
    classifications: &[FileClassification],
) -> (InclusionPolicy, Option<String>) {
    assign_context_policy(tokens, file_cap, classifications)
}

/// Select files based on strategy (no smart excludes, no spine reservation).
#[allow(dead_code)]
pub fn select_files(
    rows: &[FileRow],
    budget: usize,
    strategy: ContextStrategy,
    metric: ValueMetric,
    git_scores: Option<&GitScores>,
) -> Vec<ContextFileRow> {
    select_files_with_options(
        rows,
        budget,
        strategy,
        metric,
        git_scores,
        &SelectOptions {
            no_smart_exclude: true,
            ..Default::default()
        },
    )
    .selected
}

/// Select files with smart excludes and spine reservation.
pub fn select_files_with_options(
    rows: &[FileRow],
    budget: usize,
    strategy: ContextStrategy,
    metric: ValueMetric,
    git_scores: Option<&GitScores>,
    options: &SelectOptions,
) -> SelectResult {
    // Step 0: Resolve metric (detect fallback)
    let resolved = resolve_metric(metric, git_scores);
    let effective_metric = resolved.effective;

    // If require_git_scores is set and a fallback occurred, we still proceed
    // but the caller can check fallback_reason and decide to error.

    let metric_name = match effective_metric {
        ValueMetric::Code => "code",
        ValueMetric::Tokens => "tokens",
        ValueMetric::Churn => "churn",
        ValueMetric::Hotspot => "hotspot",
    };

    // Step 1: Partition smart-excluded files
    let mut smart_excluded = Vec::new();
    let candidates: Vec<&FileRow> = if options.no_smart_exclude {
        rows.iter().collect()
    } else {
        rows.iter()
            .filter(|row| {
                if row.kind != FileKind::Parent {
                    return true; // Don't filter children (they're filtered later)
                }
                let path = normalize_path(&row.path);
                if let Some(reason) = is_smart_excluded(&path) {
                    smart_excluded.push(SmartExcludedFile {
                        path,
                        reason: reason.to_string(),
                        tokens: row.tokens,
                    });
                    false
                } else {
                    true
                }
            })
            .collect()
    };

    // Collect candidates back into a Vec<FileRow> (needed by pack functions)
    let candidate_rows: Vec<FileRow> = candidates.into_iter().cloned().collect();

    // Step 2: Classify all candidates and compute file cap
    let policy_selection = policy::prepare_policy_selection(&candidate_rows, budget, options);

    // Step 4: Spine reservation
    let spine_budget = std::cmp::min(
        (budget as f64 * SPINE_BUDGET_FRACTION) as usize,
        SPINE_BUDGET_CAP,
    );

    let parents: Vec<&FileRow> = policy_selection
        .pack_rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .collect();

    let mut spine_files: Vec<ContextFileRow> = Vec::new();
    let mut spine_used = 0;
    let mut spine_paths: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();

    let mut spine_candidates: Vec<&FileRow> = parents
        .iter()
        .filter(|r| is_spine_file(&r.path))
        .copied()
        .collect();
    spine_candidates.sort_by(|a, b| a.tokens.cmp(&b.tokens).then_with(|| a.path.cmp(&b.path)));

    for row in spine_candidates {
        if spine_used + row.tokens <= spine_budget {
            spine_used += row.tokens;
            spine_paths.insert(row.path.as_str());
            spine_files.push(to_context_row_with_reason(
                row,
                effective_metric,
                git_scores,
                "spine",
            ));
        }
    }

    // Step 5: Normal selection with remaining budget, excluding spine files
    let remaining_budget = budget.saturating_sub(spine_used);
    let non_spine_rows: Vec<FileRow> = policy_selection
        .pack_rows
        .iter()
        .filter(|r| !spine_paths.contains(&r.path.as_str()))
        .cloned()
        .collect();

    let mut ranked: Vec<ContextFileRow> = match strategy {
        ContextStrategy::Greedy => pack_greedy(
            &non_spine_rows,
            remaining_budget,
            effective_metric,
            git_scores,
        ),
        ContextStrategy::Spread => pack_spread(
            &non_spine_rows,
            remaining_budget,
            effective_metric,
            git_scores,
        ),
    };

    // Tag ranked files with their metric reason
    for file in &mut ranked {
        if file.rank_reason.is_empty() {
            file.rank_reason = metric_name.to_string();
        }
    }

    // Step 6: Concatenate spine + ranked
    let mut selected = spine_files;
    selected.extend(ranked);

    // Step 7: Annotate each selected file with policy, classifications, effective_tokens
    policy_selection.annotate_selected(&mut selected);

    SelectResult {
        selected,
        smart_excluded,
        excluded_by_policy: policy_selection.excluded_by_policy,
        rank_by_effective: metric_name.to_string(),
        fallback_reason: resolved.fallback_reason,
    }
}

#[cfg(test)]
mod tests;
