//! Deep tests for tokmd-progress: spinner/bar creation, no-op mode, and edge cases.
//!
//! All tests run with `enabled = false` to avoid TTY requirements in CI.
//! This exercises the no-op / disabled code paths which must be safe and silent.

use tokmd_progress::{Progress, ProgressBarWithEta};

// ── Progress (spinner) ─────────────────────────────────────────────────

#[test]
fn progress_disabled_does_not_panic() {
    let p = Progress::new(false);
    p.set_message("scanning");
    p.set_message("");
    p.finish_and_clear();
}

#[test]
fn progress_disabled_multiple_messages() {
    let p = Progress::new(false);
    for i in 0..100 {
        p.set_message(format!("step {i}"));
    }
    p.finish_and_clear();
}

#[test]
fn progress_disabled_finish_is_idempotent() {
    let p = Progress::new(false);
    p.finish_and_clear();
    p.finish_and_clear();
    p.finish_and_clear();
}

#[test]
fn progress_disabled_drop_is_safe() {
    {
        let p = Progress::new(false);
        p.set_message("will be dropped");
    }
    // No panic on drop
}

#[test]
fn progress_enabled_in_non_tty_acts_as_noop() {
    // In CI / test environments, stderr is not a TTY, so even `enabled = true`
    // should behave like no-op without panicking.
    let p = Progress::new(true);
    p.set_message("this is fine");
    p.finish_and_clear();
}

#[test]
fn progress_enabled_drop_in_non_tty() {
    {
        let _p = Progress::new(true);
    }
    // No panic on drop even when enabled in non-TTY
}

// ── ProgressBarWithEta ─────────────────────────────────────────────────

#[test]
fn bar_disabled_does_not_panic() {
    let bar = ProgressBarWithEta::new(false, 100, "test");
    bar.inc();
    bar.inc_by(5);
    bar.set_position(50);
    bar.set_message("halfway");
    bar.set_length(200);
    bar.finish_with_message("done");
    bar.finish_and_clear();
}

#[test]
fn bar_disabled_zero_total() {
    let bar = ProgressBarWithEta::new(false, 0, "empty");
    bar.inc();
    bar.set_position(0);
    bar.finish_and_clear();
}

#[test]
fn bar_disabled_large_total() {
    let bar = ProgressBarWithEta::new(false, u64::MAX, "large");
    bar.inc();
    bar.inc_by(1_000_000);
    bar.set_position(u64::MAX / 2);
    bar.finish_and_clear();
}

#[test]
fn bar_disabled_set_length_to_zero() {
    let bar = ProgressBarWithEta::new(false, 100, "resize");
    bar.set_length(0);
    bar.inc();
    bar.finish_and_clear();
}

#[test]
fn bar_disabled_empty_message() {
    let bar = ProgressBarWithEta::new(false, 10, "");
    bar.set_message("");
    bar.finish_with_message("");
}

#[test]
fn bar_disabled_finish_is_idempotent() {
    let bar = ProgressBarWithEta::new(false, 10, "test");
    bar.finish_and_clear();
    bar.finish_and_clear();
    bar.finish_and_clear();
}

#[test]
fn bar_disabled_finish_with_message_is_idempotent() {
    let bar = ProgressBarWithEta::new(false, 10, "test");
    bar.finish_with_message("done");
    bar.finish_with_message("really done");
}

#[test]
fn bar_disabled_drop_is_safe() {
    {
        let bar = ProgressBarWithEta::new(false, 10, "dropped");
        bar.inc();
    }
    // No panic on drop
}

#[test]
fn bar_enabled_in_non_tty_acts_as_noop() {
    let bar = ProgressBarWithEta::new(true, 100, "scan");
    bar.inc();
    bar.inc_by(10);
    bar.set_position(50);
    bar.set_message("progress");
    bar.set_length(200);
    bar.finish_with_message("done");
    bar.finish_and_clear();
}

#[test]
fn bar_enabled_drop_in_non_tty() {
    {
        let _bar = ProgressBarWithEta::new(true, 50, "scanning");
    }
    // No panic
}

// ── Rapid sequence ─────────────────────────────────────────────────────

#[test]
fn bar_rapid_increments() {
    let bar = ProgressBarWithEta::new(false, 10_000, "rapid");
    for _ in 0..10_000 {
        bar.inc();
    }
    bar.finish_and_clear();
}

#[test]
fn bar_inc_by_various_deltas() {
    let bar = ProgressBarWithEta::new(false, 1000, "deltas");
    bar.inc_by(0);
    bar.inc_by(1);
    bar.inc_by(100);
    bar.inc_by(500);
    bar.inc_by(u64::MAX);
    bar.finish_and_clear();
}

// ── Multiple concurrent instances ──────────────────────────────────────

#[test]
fn multiple_progress_instances() {
    let p1 = Progress::new(false);
    let p2 = Progress::new(false);
    p1.set_message("first");
    p2.set_message("second");
    p1.finish_and_clear();
    p2.finish_and_clear();
}

#[test]
fn multiple_bar_instances() {
    let b1 = ProgressBarWithEta::new(false, 100, "first");
    let b2 = ProgressBarWithEta::new(false, 200, "second");
    b1.inc();
    b2.inc_by(5);
    b1.finish_and_clear();
    b2.finish_and_clear();
}

// ── Mixed usage ────────────────────────────────────────────────────────

#[test]
fn spinner_and_bar_together() {
    let spinner = Progress::new(false);
    let bar = ProgressBarWithEta::new(false, 50, "files");
    spinner.set_message("scanning...");
    for i in 0..50 {
        bar.inc();
        if i % 10 == 0 {
            bar.set_message(&format!("file {i}"));
        }
    }
    bar.finish_with_message("complete");
    spinner.finish_and_clear();
}
