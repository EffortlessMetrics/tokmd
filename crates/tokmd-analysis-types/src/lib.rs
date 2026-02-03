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
/// v4: Added cognitive complexity, nesting depth, and function-level details.
pub const ANALYSIS_SCHEMA_VERSION: u32 = 4;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub bytes: u64,
    pub files: Vec<String>,
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

/// Helper to convert milliseconds timestamp to ISO 8601 string.
fn chrono_timestamp_iso8601(ms: u128) -> String {
    // Simple conversion: ms to seconds, format as basic ISO timestamp
    let secs = (ms / 1000) as i64;
    let nanos = ((ms % 1000) * 1_000_000) as u32;

    // Use a basic formatting approach without external chrono dependency
    // Format: YYYY-MM-DDTHH:MM:SS.sssZ
    // For simplicity, we output the Unix timestamp in a parseable format
    // A proper implementation would use chrono, but we keep dependencies minimal
    format!("{}:{:09}", secs, nanos)
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
// Ecosystem Envelope (v1)
// =========================

/// Schema version for ecosystem envelope format.
/// v1: Initial envelope specification for multi-sensor integration.
pub const ENVELOPE_VERSION: u32 = 1;

/// Ecosystem envelope for multi-sensor integration.
///
/// The envelope provides a standardized JSON format that allows tokmd to integrate
/// with external orchestrators ("directors") that aggregate reports from multiple
/// code quality sensors into a unified PR view.
///
/// # Design Principles
/// - **Stable top-level, rich underneath**: Minimal stable envelope; tool-specific richness in `data`
/// - **Verdict-first**: Quick pass/fail/warn determination without parsing tool-specific data
/// - **Findings are portable**: Common finding structure for cross-tool aggregation
/// - **Self-describing**: Schema version and tool metadata enable forward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// Schema version (currently 1).
    pub envelope_version: u32,
    /// Tool identification.
    pub tool: EnvelopeTool,
    /// Generation timestamp (ISO 8601 format).
    pub generated_at: String,
    /// Overall result verdict.
    pub verdict: Verdict,
    /// Human-readable one-line summary.
    pub summary: String,
    /// List of findings (may be empty).
    pub findings: Vec<Finding>,
    /// Evidence gate status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gates: Option<GatesEnvelope>,
    /// Related artifact paths.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<Artifact>>,
    /// Tool-specific payload (opaque to director).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Tool identification for envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeTool {
    /// Tool name (e.g., "tokmd").
    pub name: String,
    /// Tool version (e.g., "1.5.0").
    pub version: String,
    /// Operation mode (e.g., "cockpit", "analyze").
    pub mode: String,
}

/// Overall verdict for the envelope.
///
/// Directors aggregate verdicts: `fail` > `pending` > `warn` > `pass` > `skip`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    /// All checks passed, no significant findings.
    #[default]
    Pass,
    /// Hard failure (evidence gate failed, policy violation).
    Fail,
    /// Soft warnings present, review recommended.
    Warn,
    /// Sensor skipped (missing inputs, not applicable).
    Skip,
    /// Awaiting external data (CI artifacts, etc.).
    Pending,
}

/// A finding reported by the tool.
///
/// Finding IDs follow the convention: `<tool>.<category>.<code>`
/// (e.g., `tokmd.risk.hotspot`, `tokmd.gate.mutation_failed`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Finding identifier (e.g., "tokmd.risk.hotspot").
    pub id: String,
    /// Severity level.
    pub severity: FindingSeverity,
    /// Short title for the finding.
    pub title: String,
    /// Detailed message describing the finding.
    pub message: String,
    /// Source location (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<FindingLocation>,
    /// Additional evidence data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<serde_json::Value>,
    /// Documentation URL for this finding type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
}

/// Severity level for findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingSeverity {
    /// Blocks merge (hard gate failure).
    Error,
    /// Review recommended.
    Warn,
    /// Informational, no action required.
    Info,
}

/// Source location for a finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingLocation {
    /// File path (normalized to forward slashes).
    pub path: String,
    /// Line number (1-indexed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Column number (1-indexed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

/// Evidence gates envelope section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatesEnvelope {
    /// Overall gate status.
    pub status: Verdict,
    /// Individual gate items.
    pub items: Vec<GateItem>,
}

/// Individual gate item in the gates envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateItem {
    /// Gate identifier (e.g., "mutation", "diff_coverage").
    pub id: String,
    /// Gate status.
    pub status: Verdict,
    /// Threshold value (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    /// Actual measured value (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<f64>,
    /// Reason for the status (especially for pending/fail).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Data source (e.g., "ci_artifact", "computed").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Path to the source artifact (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_path: Option<String>,
}

/// Artifact reference in the envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Artifact type (e.g., "comment", "receipt", "badge").
    #[serde(rename = "type")]
    pub artifact_type: String,
    /// Path to the artifact file.
    pub path: String,
}

// --------------------------
// Envelope helper methods
// --------------------------

impl Envelope {
    /// Create a new envelope with the current version.
    pub fn new(
        tool: EnvelopeTool,
        generated_at: String,
        verdict: Verdict,
        summary: String,
    ) -> Self {
        Self {
            envelope_version: ENVELOPE_VERSION,
            tool,
            generated_at,
            verdict,
            summary,
            findings: Vec::new(),
            gates: None,
            artifacts: None,
            data: None,
        }
    }

    /// Add a finding to the envelope.
    pub fn add_finding(&mut self, finding: Finding) {
        self.findings.push(finding);
    }

    /// Set the gates section.
    pub fn with_gates(mut self, gates: GatesEnvelope) -> Self {
        self.gates = Some(gates);
        self
    }

    /// Set the artifacts section.
    pub fn with_artifacts(mut self, artifacts: Vec<Artifact>) -> Self {
        self.artifacts = Some(artifacts);
        self
    }

    /// Set the data payload.
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

impl EnvelopeTool {
    /// Create a new tool identifier for tokmd.
    pub fn tokmd(version: &str, mode: &str) -> Self {
        Self {
            name: "tokmd".to_string(),
            version: version.to_string(),
            mode: mode.to_string(),
        }
    }
}

impl Finding {
    /// Create a new finding with required fields.
    pub fn new(
        id: impl Into<String>,
        severity: FindingSeverity,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            severity,
            title: title.into(),
            message: message.into(),
            location: None,
            evidence: None,
            docs_url: None,
        }
    }

    /// Add a location to the finding.
    pub fn with_location(mut self, location: FindingLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Add evidence to the finding.
    pub fn with_evidence(mut self, evidence: serde_json::Value) -> Self {
        self.evidence = Some(evidence);
        self
    }

    /// Add a documentation URL to the finding.
    pub fn with_docs_url(mut self, url: impl Into<String>) -> Self {
        self.docs_url = Some(url.into());
        self
    }
}

impl FindingLocation {
    /// Create a new location with just a path.
    pub fn path(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            line: None,
            column: None,
        }
    }

    /// Create a new location with path and line.
    pub fn path_line(path: impl Into<String>, line: u32) -> Self {
        Self {
            path: path.into(),
            line: Some(line),
            column: None,
        }
    }

    /// Create a new location with path, line, and column.
    pub fn path_line_column(path: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            path: path.into(),
            line: Some(line),
            column: Some(column),
        }
    }
}

impl GatesEnvelope {
    /// Create a new gates envelope.
    pub fn new(status: Verdict, items: Vec<GateItem>) -> Self {
        Self { status, items }
    }
}

impl GateItem {
    /// Create a new gate item with required fields.
    pub fn new(id: impl Into<String>, status: Verdict) -> Self {
        Self {
            id: id.into(),
            status,
            threshold: None,
            actual: None,
            reason: None,
            source: None,
            artifact_path: None,
        }
    }

    /// Create a gate item with pass/fail based on threshold comparison.
    pub fn with_threshold(mut self, threshold: f64, actual: f64) -> Self {
        self.threshold = Some(threshold);
        self.actual = Some(actual);
        self
    }

    /// Add a reason to the gate item.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Add a source to the gate item.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Add an artifact path to the gate item.
    pub fn with_artifact_path(mut self, path: impl Into<String>) -> Self {
        self.artifact_path = Some(path.into());
        self
    }
}

impl Artifact {
    /// Create a new artifact reference.
    pub fn new(artifact_type: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            artifact_type: artifact_type.into(),
            path: path.into(),
        }
    }

    /// Create a comment artifact.
    pub fn comment(path: impl Into<String>) -> Self {
        Self::new("comment", path)
    }

    /// Create a receipt artifact.
    pub fn receipt(path: impl Into<String>) -> Self {
        Self::new("receipt", path)
    }

    /// Create a badge artifact.
    pub fn badge(path: impl Into<String>) -> Self {
        Self::new("badge", path)
    }
}
