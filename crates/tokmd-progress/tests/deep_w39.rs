#![allow(clippy::drop_non_drop)]
//! Wave-39 deep tests for tokmd-progress: construction, method coverage,
//! and edge cases for both Progress and ProgressBarWithEta.

use tokmd_progress::{Progress, ProgressBarWithEta};

// ── Progress spinner (disabled) ──────────────────────────────────────────

#[test]
fn progress_disabled_set_message_no_panic() {
    let p = Progress::new(false);
    p.set_message("scanning files...");
    p.set_message(String::from("owned string"));
}

#[test]
fn progress_disabled_finish_idempotent() {
    let p = Progress::new(false);
    p.finish_and_clear();
    p.finish_and_clear(); // double finish should not panic
}

#[test]
fn progress_disabled_drop_safe() {
    {
        let p = Progress::new(false);
        p.set_message("will be dropped");
    } // drop here
}

#[test]
fn progress_enabled_in_non_tty_degrades_gracefully() {
    // In CI/test, stderr is not a TTY so this should degrade to no-op.
    let p = Progress::new(true);
    p.set_message("should degrade");
    p.finish_and_clear();
}

// ── ProgressBarWithEta (disabled) ────────────────────────────────────────

#[test]
fn bar_disabled_all_methods_no_panic() {
    let bar = ProgressBarWithEta::new(false, 100, "processing");
    bar.inc();
    bar.inc_by(5);
    bar.set_position(10);
    bar.set_message("halfway");
    bar.set_length(200);
    bar.finish_with_message("complete");
    bar.finish_and_clear();
}

#[test]
fn bar_disabled_zero_total() {
    let bar = ProgressBarWithEta::new(false, 0, "zero items");
    bar.inc();
    bar.set_position(0);
    bar.finish_and_clear();
}

#[test]
fn bar_disabled_very_large_total() {
    let bar = ProgressBarWithEta::new(false, u64::MAX, "huge");
    bar.inc();
    bar.inc_by(u64::MAX / 2);
    bar.set_position(u64::MAX);
    bar.finish_and_clear();
}

#[test]
fn bar_disabled_double_finish() {
    let bar = ProgressBarWithEta::new(false, 50, "test");
    bar.finish_with_message("done");
    bar.finish_and_clear();
    bar.finish_and_clear();
}

#[test]
fn bar_disabled_drop_safe() {
    {
        let bar = ProgressBarWithEta::new(false, 10, "will drop");
        bar.inc();
    } // drop
}

#[test]
fn bar_enabled_non_tty_degrades() {
    // In test context, stderr is not a TTY, so bar degrades to no-op.
    let bar = ProgressBarWithEta::new(true, 100, "test");
    bar.inc();
    bar.inc_by(10);
    bar.set_position(50);
    bar.set_message("msg");
    bar.set_length(200);
    bar.finish_with_message("ok");
}

// ── Sequential operations ────────────────────────────────────────────────

#[test]
fn bar_sequential_inc_to_completion() {
    let bar = ProgressBarWithEta::new(false, 5, "counting");
    for _ in 0..5 {
        bar.inc();
    }
    bar.finish_and_clear();
}

#[test]
fn bar_set_length_then_position() {
    let bar = ProgressBarWithEta::new(false, 10, "resize");
    bar.set_length(20);
    bar.set_position(15);
    bar.finish_and_clear();
}

// ── Multiple progress instances ──────────────────────────────────────────

#[test]
fn multiple_progress_instances_independent() {
    let p1 = Progress::new(false);
    let p2 = Progress::new(false);
    p1.set_message("first");
    p2.set_message("second");
    p1.finish_and_clear();
    p2.finish_and_clear();
}

#[test]
fn multiple_bars_independent() {
    let b1 = ProgressBarWithEta::new(false, 10, "bar1");
    let b2 = ProgressBarWithEta::new(false, 20, "bar2");
    b1.inc();
    b2.inc_by(5);
    b1.finish_and_clear();
    b2.finish_and_clear();
}
