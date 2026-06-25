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
#[cfg(test)]
pub(crate) fn generate_complexity_histogram(
    files: &[FileComplexity],
    bucket_size: u32,
) -> ComplexityHistogram {
    generate_complexity_histogram_with_max(files, None, bucket_size)
}

/// Generate a histogram using per-function max cyclomatic per file.
pub(crate) fn generate_complexity_histogram_for_files(
    files: &[FileComplexity],
    per_file_max_cyclomatic: &[usize],
    bucket_size: u32,
) -> ComplexityHistogram {
    generate_complexity_histogram_with_max(files, Some(per_file_max_cyclomatic), bucket_size)
}

/// Generate a histogram using per-function max cyclomatic per file when provided.
fn generate_complexity_histogram_with_max(
    files: &[FileComplexity],
    per_file_max_cyclomatic: Option<&[usize]>,
    bucket_size: u32,
) -> ComplexityHistogram {
    if let Some(maxes) = per_file_max_cyclomatic {
        debug_assert_eq!(files.len(), maxes.len());
    }

    // 7 buckets: 0-4, 5-9, 10-14, 15-19, 20-24, 25-29, 30+
    let num_buckets = 7;
    let mut counts = vec![0u32; num_buckets];

    for (idx, file) in files.iter().enumerate() {
        let complexity = per_file_max_cyclomatic
            .and_then(|maxes| maxes.get(idx).copied())
            .unwrap_or(file.cyclomatic_complexity) as u32;
        let bucket = (complexity / bucket_size).min((num_buckets - 1) as u32) as usize;
        if let Some(count) = counts.get_mut(bucket) {
            *count += 1;
        }
    }

    ComplexityHistogram {
        buckets: (0..num_buckets).map(|i| (i as u32) * bucket_size).collect(),
        counts,
        total: files.len() as u32,
    }
}
