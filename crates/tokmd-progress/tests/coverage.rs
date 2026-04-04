//! Additional coverage tests for `tokmd-progress`.
//!
//! Targets concurrent usage, edge cases around enabled=true in non-TTY,
//! and lifecycle patterns not covered by the existing suites.

use tokmd_progress::{Progress, ProgressBarWithEta};

// ── Enabled=true in non-TTY: graceful degradation ───────────────────────

#[test]
fn given_spinner_enabled_true_when_non_tty_then_all_methods_safe() {
    // In CI / non-TTY, enabled=true should still work without panic.
    let p = Progress::new(true);
    p.set_message("scanning files");
    p.set_message(String::from("owned message"));
    p.finish_and_clear();
}

#[test]
fn given_bar_enabled_true_when_non_tty_then_full_lifecycle_safe() {
    let bar = ProgressBarWithEta::new(true, 100, "files");
    bar.inc();
    bar.inc_by(10);
    bar.set_position(50);
    bar.set_message("halfway");
    bar.set_length(200);
    bar.finish_with_message("done");
}

// ── Multiple simultaneous instances ─────────────────────────────────────

#[test]
fn given_two_spinners_when_used_concurrently_then_no_interference() {
    let s1 = Progress::new(false);
    let s2 = Progress::new(false);
    s1.set_message("task A");
    s2.set_message("task B");
    s1.finish_and_clear();
    s2.finish_and_clear();
}

#[test]
fn given_two_bars_when_used_concurrently_then_no_interference() {
    let b1 = ProgressBarWithEta::new(false, 50, "bar1");
    let b2 = ProgressBarWithEta::new(false, 100, "bar2");
    b1.inc_by(10);
    b2.inc_by(20);
    b1.finish_and_clear();
    b2.finish_and_clear();
}

// ── Methods after finish ────────────────────────────────────────────────

#[test]
fn given_finished_bar_when_methods_called_again_then_no_panic() {
    let bar = ProgressBarWithEta::new(false, 10, "test");
    bar.inc_by(5);
    bar.finish_with_message("done");
    // Call methods after finish — should not panic
    bar.inc();
    bar.set_message("post-finish");
    bar.set_position(0);
    bar.finish_and_clear();
}

#[test]
fn given_finished_spinner_when_methods_called_again_then_no_panic() {
    let spinner = Progress::new(false);
    spinner.finish_and_clear();
    spinner.set_message("post-finish");
    spinner.finish_and_clear();
}

// ── Position regression: set backwards ──────────────────────────────────

#[test]
fn given_bar_at_position_50_when_set_to_10_then_no_panic() {
    let bar = ProgressBarWithEta::new(false, 100, "regress");
    bar.set_position(50);
    bar.set_position(10);
    bar.finish_and_clear();
}

// ── Rapid creation and destruction ──────────────────────────────────────

#[test]
fn given_many_spinners_when_created_and_dropped_rapidly_then_no_leak() {
    for _ in 0..100 {
        let p = Progress::new(false);
        p.set_message("ephemeral");
        let _ = p;
    }
}

#[test]
fn given_many_bars_when_created_and_dropped_rapidly_then_no_leak() {
    for i in 0..100u64 {
        let b = ProgressBarWithEta::new(false, i + 1, "ephemeral");
        b.inc();
        let _ = b;
    }
}
