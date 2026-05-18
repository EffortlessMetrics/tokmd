//! Numeric helpers for diff Markdown rendering.

pub(super) fn percent_change(old: usize, new: usize) -> f64 {
    if old > 0 {
        ((new as f64 - old as f64) / old as f64) * 100.0
    } else if new > 0 {
        100.0
    } else {
        0.0
    }
}
