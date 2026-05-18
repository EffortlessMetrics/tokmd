//! Cyclomatic complexity histogram DTO and display helper.
//!
//! This module owns the serde-stable histogram shape used by complexity
//! receipts. The type remains re-exported from the crate root.

use serde::{Deserialize, Serialize};

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
        use std::fmt::Write;
        let max_count = self.counts.iter().max().copied().unwrap_or(1).max(1);
        let mut output = String::with_capacity(self.counts.len() * (width + 20));
        for (i, count) in self.counts.iter().enumerate() {
            match (self.buckets.get(i), self.buckets.get(i + 1)) {
                (Some(start), Some(next)) => {
                    let end = next.saturating_sub(1).max(*start);
                    let _ = write!(output, "{:>2}-{:<2} |", start, end);
                }
                (Some(start), None) => {
                    let _ = write!(output, "{:>2}+  |", start);
                }
                (None, _) => {
                    let _ = write!(output, "{:>2}+  |", 0);
                }
            }

            let bar_len = (*count as f64 / max_count as f64 * width as f64) as usize;
            for _ in 0..bar_len {
                output.push('\u{2588}');
            }
            let _ = writeln!(output, " {}", count);
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::ComplexityHistogram;

    #[test]
    fn complexity_histogram_to_ascii_basic() {
        let h = ComplexityHistogram {
            buckets: vec![0, 5, 10],
            counts: vec![10, 5, 2],
            total: 17,
        };
        let ascii = h.to_ascii(20);
        assert!(!ascii.is_empty());
        assert_eq!(ascii.lines().count(), 3);
    }

    #[test]
    fn complexity_histogram_to_ascii_empty_counts() {
        let h = ComplexityHistogram {
            buckets: vec![0, 5],
            counts: vec![0, 0],
            total: 0,
        };
        let ascii = h.to_ascii(20);
        assert!(!ascii.is_empty());
    }

    #[test]
    fn complexity_histogram_to_ascii_handles_counts_without_buckets() {
        let h = ComplexityHistogram {
            buckets: Vec::new(),
            counts: vec![3, 1],
            total: 4,
        };
        let ascii = h.to_ascii(10);
        assert_eq!(ascii.lines().count(), 2);
        assert!(ascii.lines().all(|line| line.starts_with(" 0+")));
    }

    #[test]
    fn complexity_histogram_to_ascii_handles_non_increasing_buckets() {
        let h = ComplexityHistogram {
            buckets: vec![5, 0],
            counts: vec![1, 2],
            total: 3,
        };
        let ascii = h.to_ascii(10);
        assert!(ascii.lines().next().unwrap().starts_with(" 5-5"));
    }
}
