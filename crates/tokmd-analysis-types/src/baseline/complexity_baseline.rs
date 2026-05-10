//! Aggregate baseline receipt DTOs.
//!
//! This submodule owns the top-level complexity baseline contract while
//! preserving the existing `ComplexityBaseline` re-export from
//! `tokmd_analysis_types`.

use serde::{Deserialize, Serialize};

use super::{
    BASELINE_VERSION, BaselineComplexitySection, BaselineMetrics, DeterminismBaseline,
    FileBaselineEntry,
};
use crate::AnalysisReceipt;

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

        let (metrics, files, complexity) = if let Some(ref complexity_report) = receipt.complexity {
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

            // Build complexity section mirroring analysis receipt structure.
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
            let fallback_metrics = BaselineMetrics {
                total_code_lines,
                total_files,
                ..Default::default()
            };
            (fallback_metrics, Vec::new(), None)
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{SecondsFormat, TimeZone, Utc};
    use proptest::prelude::*;

    #[test]
    fn complexity_baseline_default() -> Result<(), Box<dyn std::error::Error>> {
        let b = ComplexityBaseline::default();
        assert_eq!(b.baseline_version, BASELINE_VERSION);
        assert!(b.generated_at.is_empty());
        assert!(b.commit.is_none());
        assert!(b.files.is_empty());
        assert!(b.complexity.is_none());
        assert!(b.determinism.is_none());
        Ok(())
    }

    #[test]
    fn complexity_baseline_new_equals_default() -> Result<(), Box<dyn std::error::Error>> {
        let a = ComplexityBaseline::new();
        let b = ComplexityBaseline::default();
        assert_eq!(a.baseline_version, b.baseline_version);
        assert_eq!(a.generated_at, b.generated_at);
        assert_eq!(a.files.len(), b.files.len());
        Ok(())
    }

    #[test]
    fn complexity_baseline_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let b = ComplexityBaseline {
            baseline_version: BASELINE_VERSION,
            generated_at: "2025-01-01T00:00:00.000Z".into(),
            commit: Some("abc123".into()),
            metrics: BaselineMetrics::default(),
            files: vec![FileBaselineEntry {
                path: "src/lib.rs".into(),
                code_lines: 100,
                cyclomatic: 5,
                cognitive: 3,
                max_nesting: 2,
                function_count: 10,
                content_hash: Some("deadbeef".into()),
            }],
            complexity: None,
            determinism: None,
        };
        let json = serde_json::to_string(&b)?;
        let back: ComplexityBaseline = serde_json::from_str(&json)?;
        assert_eq!(back.baseline_version, BASELINE_VERSION);
        assert_eq!(back.commit.as_deref(), Some("abc123"));
        assert_eq!(back.files.len(), 1);
        assert_eq!(back.files[0].path, "src/lib.rs");
        Ok(())
    }

    #[test]
    fn timestamp_epoch() -> Result<(), Box<dyn std::error::Error>> {
        let result = chrono_timestamp_iso8601(0);
        assert_eq!(result, "1970-01-01T00:00:00.000Z");
        Ok(())
    }

    #[test]
    fn timestamp_with_millis() -> Result<(), Box<dyn std::error::Error>> {
        // 2025-01-01T00:00:00.500Z = 1735689600500 ms
        let result = chrono_timestamp_iso8601(1735689600500);
        assert!(result.ends_with(".500Z"));
        assert!(result.starts_with("2025-01-01"));
        Ok(())
    }

    proptest! {
        #[test]
        fn chrono_timestamp_matches_chrono(ms in 0u128..253_402_300_799_000u128) {
            let chrono_dt = Utc
                .timestamp_millis_opt(ms as i64)
                .single()
                .expect("timestamp within supported range");
            let expected = chrono_dt.to_rfc3339_opts(SecondsFormat::Millis, true);
            prop_assert_eq!(chrono_timestamp_iso8601(ms), expected);
        }

        #[test]
        fn chrono_timestamp_is_rfc3339(ms in 0u128..253_402_300_799_000u128) {
            let rendered = chrono_timestamp_iso8601(ms);
            prop_assert!(chrono::DateTime::parse_from_rfc3339(&rendered).is_ok());
        }
    }
}
