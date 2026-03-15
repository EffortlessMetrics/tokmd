#![allow(clippy::drop_non_drop)]
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

// ── Tests from PR #332 ──────────────────────────────────────────────

// ---- Progress (spinner) creation ----

#[test]
fn progress_new_disabled() {
    let _p = Progress::new(false);
}

#[test]
fn progress_new_enabled_non_tty() {
    // In test environment, stderr is not a TTY, so spinner is not shown
    // but this should not panic.
    let _p = Progress::new(true);
}

#[test]
fn progress_set_message_when_disabled() {
    let p = Progress::new(false);
    p.set_message("scanning...");
}

#[test]
fn progress_set_message_with_string() {
    let p = Progress::new(false);
    p.set_message(String::from("scanning..."));
}

#[test]
fn progress_finish_and_clear_when_disabled() {
    let p = Progress::new(false);
    p.finish_and_clear();
}

#[test]
fn progress_set_message_then_finish() {
    let p = Progress::new(false);
    p.set_message("step 1");
    p.set_message("step 2");
    p.finish_and_clear();
}

#[test]
fn progress_double_finish() {
    let p = Progress::new(false);
    p.finish_and_clear();
    p.finish_and_clear();
}

#[test]
fn progress_drop_without_finish() {
    let p = Progress::new(false);
    p.set_message("working...");
    drop(p);
}

#[test]
fn progress_enabled_in_ci_does_not_panic() {
    // Even when enabled=true, if stderr is not a TTY the spinner won't show.
    let p = Progress::new(true);
    p.set_message("ci mode");
    p.finish_and_clear();
}

// ---- ProgressBarWithEta ----

#[test]
fn progress_bar_new_disabled() {
    let _pb = ProgressBarWithEta::new(false, 100, "test");
}

#[test]
fn progress_bar_new_enabled_non_tty() {
    let _pb = ProgressBarWithEta::new(true, 100, "test");
}

#[test]
fn progress_bar_inc_when_disabled() {
    let pb = ProgressBarWithEta::new(false, 10, "scan");
    pb.inc();
}

#[test]
fn progress_bar_inc_by_when_disabled() {
    let pb = ProgressBarWithEta::new(false, 10, "scan");
    pb.inc_by(5);
}

#[test]
fn progress_bar_set_position_when_disabled() {
    let pb = ProgressBarWithEta::new(false, 10, "scan");
    pb.set_position(5);
}

#[test]
fn progress_bar_set_message_when_disabled() {
    let pb = ProgressBarWithEta::new(false, 10, "scan");
    pb.set_message("updated");
}

#[test]
fn progress_bar_set_length_when_disabled() {
    let pb = ProgressBarWithEta::new(false, 10, "scan");
    pb.set_length(20);
}

#[test]
fn progress_bar_finish_with_message_when_disabled() {
    let pb = ProgressBarWithEta::new(false, 10, "scan");
    pb.finish_with_message("done");
}

#[test]
fn progress_bar_finish_and_clear_when_disabled() {
    let pb = ProgressBarWithEta::new(false, 10, "scan");
    pb.finish_and_clear();
}

// ---- Zero total ----

#[test]
fn progress_bar_zero_total() {
    let pb = ProgressBarWithEta::new(false, 0, "empty");
    pb.inc();
    pb.set_position(0);
    pb.finish_and_clear();
}

// ---- Large total ----

#[test]
fn progress_bar_large_total() {
    let pb = ProgressBarWithEta::new(false, u64::MAX, "huge");
    pb.inc();
    pb.inc_by(1_000_000);
    pb.set_position(u64::MAX / 2);
    pb.finish_and_clear();
}

// ---- Full lifecycle ----

#[test]
fn progress_bar_full_lifecycle() {
    let pb = ProgressBarWithEta::new(false, 5, "lifecycle");
    pb.set_message("starting");
    for _ in 0..5 {
        pb.inc();
    }
    pb.finish_with_message("complete");
}

#[test]
fn progress_bar_update_length_mid_progress() {
    let pb = ProgressBarWithEta::new(false, 10, "resize");
    pb.inc_by(3);
    pb.set_length(20);
    pb.inc_by(7);
    pb.finish_and_clear();
}

#[test]
fn progress_bar_double_finish() {
    let pb = ProgressBarWithEta::new(false, 10, "double");
    pb.finish_and_clear();
    pb.finish_and_clear();
}

#[test]
fn progress_bar_drop_without_finish() {
    let pb = ProgressBarWithEta::new(false, 10, "drop");
    pb.inc_by(3);
    drop(pb);
}

// ---- Multiple progress bars ----

#[test]
fn multiple_progress_bars_simultaneously() {
    let pb1 = ProgressBarWithEta::new(false, 10, "first");
    let pb2 = ProgressBarWithEta::new(false, 20, "second");
    pb1.inc();
    pb2.inc_by(5);
    pb1.finish_and_clear();
    pb2.finish_and_clear();
}

#[test]
fn multiple_spinners_simultaneously() {
    let s1 = Progress::new(false);
    let s2 = Progress::new(false);
    s1.set_message("one");
    s2.set_message("two");
    s1.finish_and_clear();
    s2.finish_and_clear();
}

// ---- Empty message ----

#[test]
fn progress_bar_empty_message() {
    let pb = ProgressBarWithEta::new(false, 10, "");
    pb.set_message("");
    pb.finish_with_message("");
}

#[test]
fn progress_spinner_empty_message() {
    let p = Progress::new(false);
    p.set_message("");
    p.finish_and_clear();
}

// ---- Unicode messages ----

#[test]
fn progress_bar_unicode_message() {
    let pb = ProgressBarWithEta::new(false, 10, "スキャン");
    pb.set_message("完了");
    pb.finish_with_message("✅ 成功");
}

#[test]
fn progress_spinner_unicode_message() {
    let p = Progress::new(false);
    p.set_message("分析中...");
    p.finish_and_clear();
}
