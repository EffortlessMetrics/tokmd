//! Per-file token cap calculations for context policy.

use super::DEFAULT_MAX_FILE_TOKENS;

/// Compute the maximum tokens a single file may consume.
#[must_use]
pub fn compute_file_cap(budget: usize, max_file_pct: f64, max_file_tokens: Option<usize>) -> usize {
    if budget == usize::MAX {
        return usize::MAX;
    }

    let pct_cap = (budget as f64 * max_file_pct) as usize;
    let hard_cap = max_file_tokens.unwrap_or(DEFAULT_MAX_FILE_TOKENS);
    pct_cap.min(hard_cap)
}
