//! Effort estimation and legacy COCOMO receipt DTOs.
//!
//! These types remain re-exported from the crate root to preserve the public
//! `tokmd_analysis_types::...` contract while keeping the DTO family in an
//! owner module.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortEstimateReport {
    pub model: EffortModel,
    pub size_basis: EffortSizeBasis,
    pub results: EffortResults,
    pub confidence: EffortConfidence,
    pub drivers: Vec<EffortDriver>,
    pub assumptions: EffortAssumptions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<EffortDeltaReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortSizeBasis {
    pub total_lines: usize,
    pub authored_lines: usize,
    pub generated_lines: usize,
    pub vendored_lines: usize,
    pub kloc_total: f64,
    pub kloc_authored: f64,
    pub generated_pct: f64,
    pub vendored_pct: f64,
    pub classification_confidence: EffortConfidenceLevel,
    pub warnings: Vec<String>,
    pub by_tag: Vec<EffortTagSizeRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortTagSizeRow {
    pub tag: String,
    pub lines: usize,
    pub authored_lines: usize,
    pub pct_of_total: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EffortModel {
    Cocomo81Basic,
    Cocomo2Early,
    Ensemble,
}

impl fmt::Display for EffortModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cocomo81Basic => f.write_str("cocomo81-basic"),
            Self::Cocomo2Early => f.write_str("cocomo2-early"),
            Self::Ensemble => f.write_str("ensemble"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortResults {
    pub effort_pm_p50: f64,
    pub schedule_months_p50: f64,
    pub staff_p50: f64,
    pub effort_pm_low: f64,
    pub effort_pm_p80: f64,
    pub schedule_months_low: f64,
    pub schedule_months_p80: f64,
    pub staff_low: f64,
    pub staff_p80: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortConfidence {
    pub level: EffortConfidenceLevel,
    pub reasons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_coverage_pct: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffortConfidenceLevel {
    Low,
    Medium,
    High,
}

impl fmt::Display for EffortConfidenceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => f.write_str("low"),
            Self::Medium => f.write_str("medium"),
            Self::High => f.write_str("high"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortDriver {
    pub key: String,
    pub label: String,
    pub weight: f64,
    pub direction: EffortDriverDirection,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffortDriverDirection {
    Raises,
    Lowers,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortAssumptions {
    pub notes: Vec<String>,
    pub overrides: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortDeltaReport {
    pub base: String,
    pub head: String,
    pub files_changed: usize,
    pub modules_changed: usize,
    pub langs_changed: usize,
    pub hotspot_files_touched: usize,
    pub coupled_neighbors_touched: usize,
    pub blast_radius: f64,
    pub classification: EffortDeltaClassification,
    pub effort_pm_low: f64,
    pub effort_pm_est: f64,
    pub effort_pm_high: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffortDeltaClassification {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for EffortDeltaClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => f.write_str("low"),
            Self::Medium => f.write_str("medium"),
            Self::High => f.write_str("high"),
            Self::Critical => f.write_str("critical"),
        }
    }
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
