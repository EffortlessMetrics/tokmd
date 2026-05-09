//! Derived analytics receipt DTOs.
//!
//! These types remain re-exported from the crate root to preserve the public
//! `tokmd_analysis_types::...` contract while keeping the DTO family in an
//! owner module.

use serde::{Deserialize, Serialize};

use crate::effort::CocomoReport;

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
pub struct IntegrityReport {
    pub algo: String,
    pub hash: String,
    pub entries: usize,
}
