//! # tokmd-analysis-types
//!
//! **Tier 1 (Analysis Contract)**
//!
//! Pure data structures for analysis receipts. No I/O or business logic.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tokmd_types::{ScanStatus, ToolInfo};

/// Schema version for analysis receipts.
pub const ANALYSIS_SCHEMA_VERSION: u32 = 2;

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
