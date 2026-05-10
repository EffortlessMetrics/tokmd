//! Baseline and ratchet receipt DTOs.
//!
//! This module owns serde-stable baseline structures. Public consumers should
//! keep using the root-level re-exports from `tokmd_analysis_types`.

mod complexity_baseline;
mod complexity_section;
mod determinism;
mod file_entry;
mod metrics;

pub use complexity_baseline::ComplexityBaseline;
pub use complexity_section::BaselineComplexitySection;
pub use determinism::DeterminismBaseline;
pub use file_entry::FileBaselineEntry;
pub use metrics::BaselineMetrics;

/// Schema version for baseline files.
/// v1: Initial baseline format with complexity and determinism tracking.
pub const BASELINE_VERSION: u32 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_version_constant() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(BASELINE_VERSION, 1);
        Ok(())
    }

    #[test]
    fn baseline_metrics_default_is_zeroed() -> Result<(), Box<dyn std::error::Error>> {
        let m = BaselineMetrics::default();
        assert_eq!(m.total_code_lines, 0);
        assert_eq!(m.total_files, 0);
        assert_eq!(m.avg_cyclomatic, 0.0);
        assert_eq!(m.max_cyclomatic, 0);
        assert_eq!(m.avg_cognitive, 0.0);
        assert_eq!(m.function_count, 0);
        Ok(())
    }
}
