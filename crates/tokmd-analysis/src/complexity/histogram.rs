//! Cyclomatic complexity histogram construction.

use tokmd_analysis_types::{ComplexityHistogram, FileComplexity};

/// Generate a histogram of cyclomatic complexity distribution.
///
/// Buckets files by cyclomatic complexity: 0-4, 5-9, 10-14, 15-19, 20-24, 25-29, 30+.
///
/// # Arguments
/// * `files` - Slice of file complexity data
/// * `bucket_size` - Size of each bucket (default 5)
///
/// # Returns
/// A `ComplexityHistogram` with counts for each bucket
///
/// # Note
/// This function is planned for integration in v1.6.0.
pub(crate) fn generate_complexity_histogram(
    files: &[FileComplexity],
    bucket_size: u32,
) -> ComplexityHistogram {
    // 7 buckets: 0-4, 5-9, 10-14, 15-19, 20-24, 25-29, 30+
    let num_buckets = 7;
    let mut counts = vec![0u32; num_buckets];

    for file in files {
        let complexity = file.cyclomatic_complexity as u32;
        let bucket = (complexity / bucket_size).min((num_buckets - 1) as u32) as usize;
        counts[bucket] += 1;
    }

    ComplexityHistogram {
        buckets: (0..num_buckets).map(|i| (i as u32) * bucket_size).collect(),
        counts,
        total: files.len() as u32,
    }
}
