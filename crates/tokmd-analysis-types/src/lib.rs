//! # tokmd-analysis-types
//!
//! **Tier 0 (Analysis Contract)**
//!
//! Pure data structures for analysis receipts. No I/O or business logic.
//!
//! ## What belongs here
//! * Analysis-specific receipt types and findings
//! * Schema definitions for analysis outputs
//! * Type enums for classification results
//!
//! ## What does NOT belong here
//! * Analysis computation logic (use tokmd-analysis)
//! * Formatting logic (use tokmd-analysis-format)
//! * File I/O operations

pub mod findings;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tokmd_types::{ScanStatus, ToolInfo};

/// Schema version for analysis receipts.
/// v7: Added coupling normalization (Jaccard/Lift), commit intent classification, near-duplicate detection.
pub const ANALYSIS_SCHEMA_VERSION: u32 = 7;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReceipt {
    pub schema_version: u32,
    pub generated_at_ms: u128,
    pub tool: ToolInfo,
    pub mode: String,
    pub status: ScanStatus,
    pub warnings: Vec<String>,
    pub source: AnalysisSource,
    pub args: AnalysisArgsMeta,
    pub archetype: Option<Archetype>,
    pub topics: Option<TopicClouds>,
    pub entropy: Option<EntropyReport>,
    pub predictive_churn: Option<PredictiveChurnReport>,
    pub corporate_fingerprint: Option<CorporateFingerprint>,
    pub license: Option<LicenseReport>,
    pub derived: Option<DerivedReport>,
    pub assets: Option<AssetReport>,
    pub deps: Option<DependencyReport>,
    pub git: Option<GitReport>,
    pub imports: Option<ImportReport>,
    pub dup: Option<DuplicateReport>,
    pub complexity: Option<ComplexityReport>,
    pub api_surface: Option<ApiSurfaceReport>,
    pub fun: Option<FunReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSource {
    pub inputs: Vec<String>,
    pub export_path: Option<String>,
    pub base_receipt_path: Option<String>,
    pub export_schema_version: Option<u32>,
    pub export_generated_at_ms: Option<u128>,
    pub base_signature: Option<String>,
    pub module_roots: Vec<String>,
    pub module_depth: usize,
    pub children: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisArgsMeta {
    pub preset: String,
    pub format: String,
    pub window_tokens: Option<usize>,
    pub git: Option<bool>,
    pub max_files: Option<usize>,
    pub max_bytes: Option<u64>,
    pub max_commits: Option<usize>,
    pub max_commit_files: Option<usize>,
    pub max_file_bytes: Option<u64>,
    pub import_granularity: String,
}

// ---------------
// Project context
// ---------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Archetype {
    pub kind: String,
    pub evidence: Vec<String>,
}

// -----------------
// Semantic topics
// -----------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicClouds {
    pub per_module: BTreeMap<String, Vec<TopicTerm>>,
    pub overall: Vec<TopicTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicTerm {
    pub term: String,
    pub score: f64,
    pub tf: u32,
    pub df: u32,
}

// -----------------
// Entropy profiling
// -----------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyReport {
    pub suspects: Vec<EntropyFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyFinding {
    pub path: String,
    pub module: String,
    pub entropy_bits_per_byte: f32,
    pub sample_bytes: u32,
    pub class: EntropyClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntropyClass {
    Low,
    Normal,
    Suspicious,
    High,
}

// -----------------
// Predictive churn
// -----------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveChurnReport {
    pub per_module: BTreeMap<String, ChurnTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChurnTrend {
    pub slope: f64,
    pub r2: f64,
    pub recent_change: i64,
    pub classification: TrendClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendClass {
    Rising,
    Flat,
    Falling,
}

// ---------------------
// Corporate fingerprint
// ---------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorporateFingerprint {
    pub domains: Vec<DomainStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStat {
    pub domain: String,
    pub commits: u32,
    pub pct: f32,
}

// -------------
// License radar
// -------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseReport {
    pub findings: Vec<LicenseFinding>,
    pub effective: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseFinding {
    pub spdx: String,
    pub confidence: f32,
    pub source_path: String,
    pub source_kind: LicenseSourceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseSourceKind {
    Metadata,
    Text,
}

// -----------------
// Derived analytics
// -----------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedReport {
    pub totals: DerivedTotals,
    pub doc_density: RatioReport,
    pub whitespace: RatioReport,
    pub verbosity: RateReport,
    pub max_file: MaxFileReport,
    pub lang_purity: LangPurityReport,
    pub nesting: NestingReport,
    pub test_density: TestDensityReport,
    pub boilerplate: BoilerplateReport,
    pub polyglot: PolyglotReport,
    pub distribution: DistributionReport,
    pub histogram: Vec<HistogramBucket>,
    pub top: TopOffenders,
    pub tree: Option<String>,
    pub reading_time: ReadingTimeReport,
    pub context_window: Option<ContextWindowReport>,
    pub cocomo: Option<CocomoReport>,
    pub todo: Option<TodoReport>,
    pub integrity: IntegrityReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedTotals {
    pub files: usize,
    pub code: usize,
    pub comments: usize,
    pub blanks: usize,
    pub lines: usize,
    pub bytes: usize,
    pub tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatioReport {
    pub total: RatioRow,
    pub by_lang: Vec<RatioRow>,
    pub by_module: Vec<RatioRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatioRow {
    pub key: String,
    pub numerator: usize,
    pub denominator: usize,
    pub ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateReport {
    pub total: RateRow,
    pub by_lang: Vec<RateRow>,
    pub by_module: Vec<RateRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateRow {
    pub key: String,
    pub numerator: usize,
    pub denominator: usize,
    pub rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaxFileReport {
    pub overall: FileStatRow,
    pub by_lang: Vec<MaxFileRow>,
    pub by_module: Vec<MaxFileRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaxFileRow {
    pub key: String,
    pub file: FileStatRow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatRow {
    pub path: String,
    pub module: String,
    pub lang: String,
    pub code: usize,
    pub comments: usize,
    pub blanks: usize,
    pub lines: usize,
    pub bytes: usize,
    pub tokens: usize,
    pub doc_pct: Option<f64>,
    pub bytes_per_line: Option<f64>,
    pub depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangPurityReport {
    pub rows: Vec<LangPurityRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangPurityRow {
    pub module: String,
    pub lang_count: usize,
    pub dominant_lang: String,
    pub dominant_lines: usize,
    pub dominant_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestingReport {
    pub max: usize,
    pub avg: f64,
    pub by_module: Vec<NestingRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestingRow {
    pub key: String,
    pub max: usize,
    pub avg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDensityReport {
    pub test_lines: usize,
    pub prod_lines: usize,
    pub test_files: usize,
    pub prod_files: usize,
    pub ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoilerplateReport {
    pub infra_lines: usize,
    pub logic_lines: usize,
    pub ratio: f64,
    pub infra_langs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyglotReport {
    pub lang_count: usize,
    pub entropy: f64,
    pub dominant_lang: String,
    pub dominant_lines: usize,
    pub dominant_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionReport {
    pub count: usize,
    pub min: usize,
    pub max: usize,
    pub mean: f64,
    pub median: f64,
    pub p90: f64,
    pub p99: f64,
    pub gini: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    pub label: String,
    pub min: usize,
    pub max: Option<usize>,
    pub files: usize,
    pub pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopOffenders {
    pub largest_lines: Vec<FileStatRow>,
    pub largest_tokens: Vec<FileStatRow>,
    pub largest_bytes: Vec<FileStatRow>,
    pub least_documented: Vec<FileStatRow>,
    pub most_dense: Vec<FileStatRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingTimeReport {
    pub minutes: f64,
    pub lines_per_minute: usize,
    pub basis_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoReport {
    pub total: usize,
    pub density_per_kloc: f64,
    pub tags: Vec<TodoTagRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoTagRow {
    pub tag: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindowReport {
    pub window_tokens: usize,
    pub total_tokens: usize,
    pub pct: f64,
    pub fits: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CocomoReport {
    pub mode: String,
    pub kloc: f64,
    pub effort_pm: f64,
    pub duration_months: f64,
    pub staff: f64,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub algo: String,
    pub hash: String,
    pub entries: usize,
}

// -------------
// Asset metrics
// -------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetReport {
    pub total_files: usize,
    pub total_bytes: u64,
    pub categories: Vec<AssetCategoryRow>,
    pub top_files: Vec<AssetFileRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCategoryRow {
    pub category: String,
    pub files: usize,
    pub bytes: u64,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetFileRow {
    pub path: String,
    pub bytes: u64,
    pub category: String,
    pub extension: String,
}

// -----------------
// Dependency metrics
// -----------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyReport {
    pub total: usize,
    pub lockfiles: Vec<LockfileReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfileReport {
    pub path: String,
    pub kind: String,
    pub dependencies: usize,
}

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

// ----------------------------
// Near-duplicate detection
// ----------------------------

/// Scope for near-duplicate comparison partitioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NearDupScope {
    /// Compare files within the same module.
    #[default]
    Module,
    /// Compare files within the same language.
    Lang,
    /// Compare all files globally.
    Global,
}

/// Parameters for near-duplicate detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearDupParams {
    pub scope: NearDupScope,
    pub threshold: f64,
    pub max_files: usize,
}

/// Report of near-duplicate file pairs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearDuplicateReport {
    pub params: NearDupParams,
    pub pairs: Vec<NearDupPairRow>,
    pub files_analyzed: usize,
    pub files_skipped: usize,
}

/// A pair of near-duplicate files with similarity score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearDupPairRow {
    pub left: String,
    pub right: String,
    pub similarity: f64,
    pub shared_fingerprints: usize,
    pub left_fingerprints: usize,
    pub right_fingerprints: usize,
}

// -----------------
// Import graph info
// -----------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportReport {
    pub granularity: String,
    pub edges: Vec<ImportEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportEdge {
    pub from: String,
    pub to: String,
    pub count: usize,
}

// -------------------
// Duplication metrics
// -------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateReport {
    pub groups: Vec<DuplicateGroup>,
    pub wasted_bytes: u64,
    pub strategy: String,
    /// Duplication density summary overall and by module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub density: Option<DuplicationDensityReport>,
    /// Near-duplicate file pairs detected by fingerprint similarity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub near: Option<NearDuplicateReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub bytes: u64,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationDensityReport {
    pub duplicate_groups: usize,
    pub duplicate_files: usize,
    pub duplicated_bytes: u64,
    pub wasted_bytes: u64,
    pub wasted_pct_of_codebase: f64,
    pub by_module: Vec<ModuleDuplicationDensityRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDuplicationDensityRow {
    pub module: String,
    pub duplicate_files: usize,
    pub wasted_files: usize,
    pub duplicated_bytes: u64,
    pub wasted_bytes: u64,
    pub module_bytes: u64,
    pub density: f64,
}

// -------------------
// Halstead metrics
// -------------------

/// Halstead software science metrics computed from operator/operand token counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalsteadMetrics {
    /// Number of distinct operators (n1).
    pub distinct_operators: usize,
    /// Number of distinct operands (n2).
    pub distinct_operands: usize,
    /// Total number of operators (N1).
    pub total_operators: usize,
    /// Total number of operands (N2).
    pub total_operands: usize,
    /// Program vocabulary: n1 + n2.
    pub vocabulary: usize,
    /// Program length: N1 + N2.
    pub length: usize,
    /// Volume: N * log2(n).
    pub volume: f64,
    /// Difficulty: (n1/2) * (N2/n2).
    pub difficulty: f64,
    /// Effort: D * V.
    pub effort: f64,
    /// Estimated programming time in seconds: E / 18.
    pub time_seconds: f64,
    /// Estimated number of bugs: V / 3000.
    pub estimated_bugs: f64,
}

// -------------------
// Maintainability Index
// -------------------

/// Composite maintainability index based on the SEI formula.
///
/// MI = 171 - 5.2 * ln(V) - 0.23 * CC - 16.2 * ln(LOC)
///
/// When Halstead volume is unavailable, a simplified formula is used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainabilityIndex {
    /// Maintainability index score (0-171 scale, higher is better).
    pub score: f64,
    /// Average cyclomatic complexity used in calculation.
    pub avg_cyclomatic: f64,
    /// Average lines of code per file used in calculation.
    pub avg_loc: f64,
    /// Average Halstead volume (if Halstead metrics were computed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_halstead_volume: Option<f64>,
    /// Letter grade: "A" (>=85), "B" (65-84), "C" (<65).
    pub grade: String,
}

/// Complexity-to-size ratio heuristic for technical debt estimation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDebtRatio {
    /// Complexity points per KLOC (higher means denser debt).
    pub ratio: f64,
    /// Aggregate complexity points used in the ratio.
    pub complexity_points: usize,
    /// KLOC basis used in the ratio denominator.
    pub code_kloc: f64,
    /// Bucketed interpretation of debt ratio.
    pub level: TechnicalDebtLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnicalDebtLevel {
    Low,
    Moderate,
    High,
    Critical,
}

// -------------------
// Complexity metrics
// -------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityReport {
    pub total_functions: usize,
    pub avg_function_length: f64,
    pub max_function_length: usize,
    pub avg_cyclomatic: f64,
    pub max_cyclomatic: usize,
    /// Average cognitive complexity across files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_cognitive: Option<f64>,
    /// Maximum cognitive complexity found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cognitive: Option<usize>,
    /// Average nesting depth across files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_nesting_depth: Option<f64>,
    /// Maximum nesting depth found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_nesting_depth: Option<usize>,
    pub high_risk_files: usize,
    /// Histogram of cyclomatic complexity distribution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub histogram: Option<ComplexityHistogram>,
    /// Halstead software science metrics (requires `halstead` feature).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub halstead: Option<HalsteadMetrics>,
    /// Composite maintainability index.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainability_index: Option<MaintainabilityIndex>,
    /// Complexity-to-size debt heuristic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_debt: Option<TechnicalDebtRatio>,
    pub files: Vec<FileComplexity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileComplexity {
    pub path: String,
    pub module: String,
    pub function_count: usize,
    pub max_function_length: usize,
    pub cyclomatic_complexity: usize,
    /// Cognitive complexity for this file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cognitive_complexity: Option<usize>,
    /// Maximum nesting depth in this file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_nesting: Option<usize>,
    pub risk_level: ComplexityRisk,
    /// Function-level complexity details (only when --detail-functions is used).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<FunctionComplexityDetail>>,
}

/// Function-level complexity details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionComplexityDetail {
    /// Function name.
    pub name: String,
    /// Start line (1-indexed).
    pub line_start: usize,
    /// End line (1-indexed).
    pub line_end: usize,
    /// Function length in lines.
    pub length: usize,
    /// Cyclomatic complexity.
    pub cyclomatic: usize,
    /// Cognitive complexity (if computed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cognitive: Option<usize>,
    /// Maximum nesting depth within the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_nesting: Option<usize>,
    /// Number of parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param_count: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplexityRisk {
    Low,
    Moderate,
    High,
    Critical,
}

/// Histogram of cyclomatic complexity distribution across files.
///
/// Used to visualize the distribution of complexity values in a codebase.
/// Default bucket boundaries are 0-4, 5-9, 10-14, 15-19, 20-24, 25-29, 30+.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityHistogram {
    /// Bucket boundaries (e.g., [0, 5, 10, 15, 20, 25, 30]).
    pub buckets: Vec<u32>,
    /// Count of files in each bucket.
    pub counts: Vec<u32>,
    /// Total files analyzed.
    pub total: u32,
}

impl ComplexityHistogram {
    /// Generate an ASCII bar chart visualization of the histogram.
    ///
    /// # Arguments
    /// * `width` - Maximum width of the bars in characters
    ///
    /// # Returns
    /// A multi-line string with labeled bars showing distribution
    pub fn to_ascii(&self, width: usize) -> String {
        let max_count = self.counts.iter().max().copied().unwrap_or(1).max(1);
        let mut output = String::new();
        for (i, count) in self.counts.iter().enumerate() {
            let label = if i < self.buckets.len() - 1 {
                format!("{:>2}-{:<2}", self.buckets[i], self.buckets[i + 1] - 1)
            } else {
                format!("{:>2}+ ", self.buckets.get(i).copied().unwrap_or(30))
            };
            let bar_len = (*count as f64 / max_count as f64 * width as f64) as usize;
            let bar = "\u{2588}".repeat(bar_len);
            output.push_str(&format!("{} |{} {}\n", label, bar, count));
        }
        output
    }
}

// -------------------
// Baseline/Ratchet types
// -------------------

/// Schema version for baseline files.
/// v1: Initial baseline format with complexity and determinism tracking.
pub const BASELINE_VERSION: u32 = 1;

/// Complexity baseline for tracking trends over time.
///
/// Used by the ratchet system to enforce that complexity metrics
/// do not regress across commits. The baseline captures a snapshot
/// of complexity at a known-good state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityBaseline {
    /// Schema version for forward compatibility.
    pub baseline_version: u32,
    /// ISO 8601 timestamp when this baseline was generated.
    pub generated_at: String,
    /// Git commit SHA at which this baseline was captured, if available.
    pub commit: Option<String>,
    /// Aggregate complexity metrics.
    pub metrics: BaselineMetrics,
    /// Per-file baseline entries for granular tracking.
    pub files: Vec<FileBaselineEntry>,
    /// Complexity section mirroring analysis receipt structure for ratchet compatibility.
    ///
    /// This allows using the same JSON pointers (e.g., `/complexity/avg_cyclomatic`)
    /// when comparing baselines against current analysis receipts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<BaselineComplexitySection>,
    /// Determinism baseline for reproducibility verification.
    ///
    /// Present when the baseline was generated with `--determinism`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub determinism: Option<DeterminismBaseline>,
}

impl ComplexityBaseline {
    /// Creates a new empty baseline with default values.
    pub fn new() -> Self {
        Self {
            baseline_version: BASELINE_VERSION,
            generated_at: String::new(),
            commit: None,
            metrics: BaselineMetrics::default(),
            files: Vec::new(),
            complexity: None,
            determinism: None,
        }
    }

    /// Creates a baseline from an analysis receipt.
    ///
    /// Extracts complexity information from the receipt's complexity report
    /// and derived totals to build a baseline snapshot.
    pub fn from_analysis(receipt: &AnalysisReceipt) -> Self {
        let generated_at = chrono_timestamp_iso8601(receipt.generated_at_ms);

        let (metrics, files, complexity) = if let Some(ref complexity_report) = receipt.complexity {
            let total_code_lines = receipt
                .derived
                .as_ref()
                .map(|d| d.totals.code as u64)
                .unwrap_or(0);
            let total_files = receipt
                .derived
                .as_ref()
                .map(|d| d.totals.files as u64)
                .unwrap_or(0);

            let metrics = BaselineMetrics {
                total_code_lines,
                total_files,
                avg_cyclomatic: complexity_report.avg_cyclomatic,
                max_cyclomatic: complexity_report.max_cyclomatic as u32,
                avg_cognitive: complexity_report.avg_cognitive.unwrap_or(0.0),
                max_cognitive: complexity_report.max_cognitive.unwrap_or(0) as u32,
                avg_nesting_depth: complexity_report.avg_nesting_depth.unwrap_or(0.0),
                max_nesting_depth: complexity_report.max_nesting_depth.unwrap_or(0) as u32,
                function_count: complexity_report.total_functions as u64,
                avg_function_length: complexity_report.avg_function_length,
            };

            let files: Vec<FileBaselineEntry> = complexity_report
                .files
                .iter()
                .map(|f| FileBaselineEntry {
                    path: f.path.clone(),
                    code_lines: 0, // Not available in FileComplexity
                    cyclomatic: f.cyclomatic_complexity as u32,
                    cognitive: f.cognitive_complexity.unwrap_or(0) as u32,
                    max_nesting: f.max_nesting.unwrap_or(0) as u32,
                    function_count: f.function_count as u32,
                    content_hash: None,
                })
                .collect();

            // Build complexity section mirroring analysis receipt structure
            let complexity_section = BaselineComplexitySection {
                total_functions: complexity_report.total_functions,
                avg_function_length: complexity_report.avg_function_length,
                max_function_length: complexity_report.max_function_length,
                avg_cyclomatic: complexity_report.avg_cyclomatic,
                max_cyclomatic: complexity_report.max_cyclomatic,
                avg_cognitive: complexity_report.avg_cognitive,
                max_cognitive: complexity_report.max_cognitive,
                avg_nesting_depth: complexity_report.avg_nesting_depth,
                max_nesting_depth: complexity_report.max_nesting_depth,
                high_risk_files: complexity_report.high_risk_files,
            };

            (metrics, files, Some(complexity_section))
        } else {
            (BaselineMetrics::default(), Vec::new(), None)
        };

        Self {
            baseline_version: BASELINE_VERSION,
            generated_at,
            commit: None,
            metrics,
            files,
            complexity,
            determinism: None,
        }
    }
}

impl Default for ComplexityBaseline {
    fn default() -> Self {
        Self::new()
    }
}

/// Complexity section mirroring analysis receipt structure for ratchet compatibility.
///
/// This provides the same field names as `ComplexityReport` so that JSON pointers
/// like `/complexity/avg_cyclomatic` work consistently across baselines and receipts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComplexitySection {
    /// Total number of functions analyzed.
    pub total_functions: usize,
    /// Average function length in lines.
    pub avg_function_length: f64,
    /// Maximum function length found.
    pub max_function_length: usize,
    /// Average cyclomatic complexity across all files.
    pub avg_cyclomatic: f64,
    /// Maximum cyclomatic complexity found in any file.
    pub max_cyclomatic: usize,
    /// Average cognitive complexity across all files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_cognitive: Option<f64>,
    /// Maximum cognitive complexity found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cognitive: Option<usize>,
    /// Average nesting depth across all files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_nesting_depth: Option<f64>,
    /// Maximum nesting depth found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_nesting_depth: Option<usize>,
    /// Number of high-risk files.
    pub high_risk_files: usize,
}

/// Aggregate baseline metrics for the entire codebase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetrics {
    /// Total lines of code across all files.
    pub total_code_lines: u64,
    /// Total number of source files.
    pub total_files: u64,
    /// Average cyclomatic complexity across all functions.
    pub avg_cyclomatic: f64,
    /// Maximum cyclomatic complexity found in any function.
    pub max_cyclomatic: u32,
    /// Average cognitive complexity across all functions.
    pub avg_cognitive: f64,
    /// Maximum cognitive complexity found in any function.
    pub max_cognitive: u32,
    /// Average nesting depth across all functions.
    pub avg_nesting_depth: f64,
    /// Maximum nesting depth found in any function.
    pub max_nesting_depth: u32,
    /// Total number of functions analyzed.
    pub function_count: u64,
    /// Average function length in lines.
    pub avg_function_length: f64,
}

impl Default for BaselineMetrics {
    fn default() -> Self {
        Self {
            total_code_lines: 0,
            total_files: 0,
            avg_cyclomatic: 0.0,
            max_cyclomatic: 0,
            avg_cognitive: 0.0,
            max_cognitive: 0,
            avg_nesting_depth: 0.0,
            max_nesting_depth: 0,
            function_count: 0,
            avg_function_length: 0.0,
        }
    }
}

/// Per-file baseline entry for granular complexity tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileBaselineEntry {
    /// Normalized file path (forward slashes).
    pub path: String,
    /// Lines of code in this file.
    pub code_lines: u64,
    /// Cyclomatic complexity for this file.
    pub cyclomatic: u32,
    /// Cognitive complexity for this file.
    pub cognitive: u32,
    /// Maximum nesting depth in this file.
    pub max_nesting: u32,
    /// Number of functions in this file.
    pub function_count: u32,
    /// BLAKE3 hash of file content for change detection.
    pub content_hash: Option<String>,
}

/// Build determinism baseline for reproducibility verification.
///
/// Tracks hashes of build artifacts and source inputs to detect
/// non-deterministic builds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismBaseline {
    /// Schema version for forward compatibility.
    pub baseline_version: u32,
    /// ISO 8601 timestamp when this baseline was generated.
    pub generated_at: String,
    /// Hash of the final build artifact.
    pub build_hash: String,
    /// Hash of all source files combined.
    pub source_hash: String,
    /// Hash of Cargo.lock if present (Rust projects).
    pub cargo_lock_hash: Option<String>,
}

/// Helper to convert milliseconds timestamp to RFC 3339 / ISO 8601 string.
fn chrono_timestamp_iso8601(ms: u128) -> String {
    // Convert milliseconds to seconds and remaining millis
    let total_secs = (ms / 1000) as i64;
    let millis = (ms % 1000) as u32;

    // Constants for date calculation
    const SECS_PER_MIN: i64 = 60;
    const SECS_PER_HOUR: i64 = 3600;
    const SECS_PER_DAY: i64 = 86400;

    // Days since Unix epoch (1970-01-01)
    let days = total_secs / SECS_PER_DAY;
    let day_secs = total_secs % SECS_PER_DAY;

    // Handle negative timestamps (before epoch)
    let (days, day_secs) = if day_secs < 0 {
        (days - 1, day_secs + SECS_PER_DAY)
    } else {
        (days, day_secs)
    };

    // Time of day
    let hour = day_secs / SECS_PER_HOUR;
    let min = (day_secs % SECS_PER_HOUR) / SECS_PER_MIN;
    let sec = day_secs % SECS_PER_MIN;

    // Convert days since epoch to year/month/day
    // Using algorithm from Howard Hinnant's date library
    let z = days + 719468; // shift to March 1, year 0
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year
    let mp = (5 * doy + 2) / 153; // month pseudo
    let d = doy - (153 * mp + 2) / 5 + 1; // day
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month
    let y = if m <= 2 { y + 1 } else { y }; // year

    // Format as RFC 3339: YYYY-MM-DDTHH:MM:SS.sssZ
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        y, m, d, hour, min, sec, millis
    )
}

// -------------------
// API Surface metrics
// -------------------

/// Public API surface analysis report.
///
/// Computes public export ratios per language and module by scanning
/// source files for exported symbols (pub fn, export function, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSurfaceReport {
    /// Total items discovered across all languages.
    pub total_items: usize,
    /// Items with public visibility.
    pub public_items: usize,
    /// Items with internal/private visibility.
    pub internal_items: usize,
    /// Ratio of public to total items (0.0-1.0).
    pub public_ratio: f64,
    /// Ratio of documented public items (0.0-1.0).
    pub documented_ratio: f64,
    /// Per-language breakdown.
    pub by_language: BTreeMap<String, LangApiSurface>,
    /// Per-module breakdown.
    pub by_module: Vec<ModuleApiRow>,
    /// Top exporters (files with most public items).
    pub top_exporters: Vec<ApiExportItem>,
}

/// Per-language API surface breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangApiSurface {
    /// Total items in this language.
    pub total_items: usize,
    /// Public items in this language.
    pub public_items: usize,
    /// Internal items in this language.
    pub internal_items: usize,
    /// Public ratio for this language.
    pub public_ratio: f64,
}

/// Per-module API surface row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleApiRow {
    /// Module path.
    pub module: String,
    /// Total items in this module.
    pub total_items: usize,
    /// Public items in this module.
    pub public_items: usize,
    /// Public ratio for this module.
    pub public_ratio: f64,
}

/// A file that exports many public items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiExportItem {
    /// File path.
    pub path: String,
    /// Language of the file.
    pub lang: String,
    /// Number of public items exported.
    pub public_items: usize,
    /// Total items in the file.
    pub total_items: usize,
}

// ---------
// Fun stuff
// ---------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunReport {
    pub eco_label: Option<EcoLabel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcoLabel {
    pub score: f64,
    pub label: String,
    pub bytes: u64,
    pub notes: String,
}

// =========================
// Ecosystem Envelope (v1) — re-exported from tokmd-envelope
// =========================

/// Schema identifier for ecosystem envelope format.
/// v1: Initial envelope specification for multi-sensor integration.
pub const ENVELOPE_SCHEMA: &str = tokmd_envelope::SENSOR_REPORT_SCHEMA;

// Re-export all envelope types with backwards-compatible aliases
pub use tokmd_envelope::Artifact;
pub use tokmd_envelope::Finding;
pub use tokmd_envelope::FindingLocation;
pub use tokmd_envelope::FindingSeverity;
pub use tokmd_envelope::GateItem;
pub use tokmd_envelope::GateResults as GatesEnvelope;
pub use tokmd_envelope::SensorReport as Envelope;
pub use tokmd_envelope::ToolMeta as EnvelopeTool;
pub use tokmd_envelope::Verdict;

// Also re-export the canonical names for new code
pub use tokmd_envelope::GateResults;
pub use tokmd_envelope::SensorReport;
pub use tokmd_envelope::ToolMeta;
