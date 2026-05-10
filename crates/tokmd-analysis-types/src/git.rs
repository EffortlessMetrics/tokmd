//! Git-derived analysis receipt DTOs.
//!
//! These contract types remain re-exported from the crate root to preserve
//! existing `tokmd_analysis_types::...` names.

use serde::{Deserialize, Serialize};

use crate::churn::TrendClass;

// ---------
// Git report
// ---------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitReport {
    pub commits_scanned: usize,
    pub files_seen: usize,
    pub hotspots: Vec<HotspotRow>,
    pub bus_factor: Vec<BusFactorRow>,
    pub freshness: FreshnessReport,
    pub coupling: Vec<CouplingRow>,
    /// Code age bucket distribution plus recent refresh trend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_distribution: Option<CodeAgeDistributionReport>,
    /// Commit intent classification (feat/fix/refactor/etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<CommitIntentReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotRow {
    pub path: String,
    pub commits: usize,
    pub lines: usize,
    pub score: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusFactorRow {
    pub module: String,
    pub authors: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshnessReport {
    pub threshold_days: usize,
    pub stale_files: usize,
    pub total_files: usize,
    pub stale_pct: f64,
    pub by_module: Vec<ModuleFreshnessRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleFreshnessRow {
    pub module: String,
    pub avg_days: f64,
    pub p90_days: f64,
    pub stale_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingRow {
    pub left: String,
    pub right: String,
    pub count: usize,
    /// Jaccard similarity: count / (n_left + n_right - count). Range (0.0, 1.0].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jaccard: Option<f64>,
    /// Lift: (count * N) / (n_left * n_right), where N = commits_considered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lift: Option<f64>,
    /// Commits touching left module (within commits_considered universe).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_left: Option<usize>,
    /// Commits touching right module (within commits_considered universe).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_right: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAgeDistributionReport {
    pub buckets: Vec<CodeAgeBucket>,
    pub recent_refreshes: usize,
    pub prior_refreshes: usize,
    pub refresh_trend: TrendClass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAgeBucket {
    pub label: String,
    pub min_days: usize,
    pub max_days: Option<usize>,
    pub files: usize,
    pub pct: f64,
}

// --------------------------
// Commit intent classification
// --------------------------

// Re-export from tokmd-types (Tier 0) so existing consumers keep working.
pub use tokmd_types::CommitIntentKind;

/// Overall commit intent classification report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitIntentReport {
    /// Aggregate counts across all scanned commits.
    pub overall: CommitIntentCounts,
    /// Per-module intent breakdown.
    pub by_module: Vec<ModuleIntentRow>,
    /// Percentage of commits classified as "other" (unrecognized).
    pub unknown_pct: f64,
    /// Corrective ratio: (fix + revert) / total. Range [0.0, 1.0].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corrective_ratio: Option<f64>,
}

/// Counts per intent kind.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommitIntentCounts {
    pub feat: usize,
    pub fix: usize,
    pub refactor: usize,
    pub docs: usize,
    pub test: usize,
    pub chore: usize,
    pub ci: usize,
    pub build: usize,
    pub perf: usize,
    pub style: usize,
    pub revert: usize,
    pub other: usize,
    pub total: usize,
}

impl CommitIntentCounts {
    /// Increment the count for a given intent kind.
    pub fn increment(&mut self, kind: CommitIntentKind) {
        match kind {
            CommitIntentKind::Feat => self.feat += 1,
            CommitIntentKind::Fix => self.fix += 1,
            CommitIntentKind::Refactor => self.refactor += 1,
            CommitIntentKind::Docs => self.docs += 1,
            CommitIntentKind::Test => self.test += 1,
            CommitIntentKind::Chore => self.chore += 1,
            CommitIntentKind::Ci => self.ci += 1,
            CommitIntentKind::Build => self.build += 1,
            CommitIntentKind::Perf => self.perf += 1,
            CommitIntentKind::Style => self.style += 1,
            CommitIntentKind::Revert => self.revert += 1,
            CommitIntentKind::Other => self.other += 1,
        }
        self.total += 1;
    }
}

/// Per-module intent breakdown row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleIntentRow {
    pub module: String,
    pub counts: CommitIntentCounts,
}
