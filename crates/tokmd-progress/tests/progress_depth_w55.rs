#![allow(clippy::drop_non_drop)]
//! Comprehensive depth tests for tokmd-progress – wave 55.
//!
//! All tests exercise the no-op (non-ui) code paths since the `ui`
//! feature is not enabled for this test crate.

use tokmd_progress::{Progress, ProgressBarWithEta};

// ── Progress (spinner) ──────────────────────────────────────────────

#[test]
fn progress_new_disabled_does_not_panic() {
    let _p = Progress::new(false);
}

#[test]
fn progress_new_enabled_without_tty_does_not_panic() {
    // Without the `ui` feature the struct is always no-op.
    let _p = Progress::new(true);
}

#[test]
fn progress_set_message_noop() {
    let p = Progress::new(false);
    p.set_message("scanning");
    p.set_message(String::from("owned string"));
}

#[test]
fn progress_finish_and_clear_noop() {
    let p = Progress::new(false);
    p.finish_and_clear();
}

#[test]
fn progress_double_finish_is_safe() {
    let p = Progress::new(false);
    p.finish_and_clear();
    p.finish_and_clear();
}

#[test]
fn progress_message_then_finish() {
    let p = Progress::new(true);
    p.set_message("step 1");
    p.set_message("step 2");
    p.finish_and_clear();
}

#[test]
fn progress_empty_message() {
    let p = Progress::new(false);
    p.set_message("");
}

// ── ProgressBarWithEta ──────────────────────────────────────────────

#[test]
fn bar_new_disabled_does_not_panic() {
    let _b = ProgressBarWithEta::new(false, 100, "work");
}

#[test]
fn bar_new_enabled_without_ui_does_not_panic() {
    let _b = ProgressBarWithEta::new(true, 100, "work");
}

#[test]
fn bar_inc_noop() {
    let b = ProgressBarWithEta::new(false, 10, "t");
    b.inc();
    b.inc();
    b.inc();
}

#[test]
fn bar_inc_by_noop() {
    let b = ProgressBarWithEta::new(false, 10, "t");
    b.inc_by(5);
}

#[test]
fn bar_set_position_noop() {
    let b = ProgressBarWithEta::new(false, 10, "t");
    b.set_position(7);
}

#[test]
fn bar_set_message_noop() {
    let b = ProgressBarWithEta::new(false, 10, "t");
    b.set_message("updated");
}

#[test]
fn bar_set_length_noop() {
    let b = ProgressBarWithEta::new(false, 10, "t");
    b.set_length(20);
}

#[test]
fn bar_finish_with_message_noop() {
    let b = ProgressBarWithEta::new(false, 10, "t");
    b.finish_with_message("done");
}

#[test]
fn bar_finish_and_clear_noop() {
    let b = ProgressBarWithEta::new(false, 10, "t");
    b.finish_and_clear();
}

#[test]
fn bar_zero_total() {
    let b = ProgressBarWithEta::new(false, 0, "empty");
    b.inc();
    b.finish_and_clear();
}

#[test]
fn bar_full_lifecycle() {
    let b = ProgressBarWithEta::new(true, 5, "lifecycle");
    b.set_message("start");
    for _ in 0..5 {
        b.inc();
    }
    b.finish_with_message("complete");
}

#[test]
fn bar_double_finish_is_safe() {
    let b = ProgressBarWithEta::new(false, 10, "t");
    b.finish_and_clear();
    b.finish_and_clear();
}

#[test]
fn bar_set_position_beyond_length() {
    let b = ProgressBarWithEta::new(false, 5, "t");
    b.set_position(100);
    b.finish_and_clear();
}
