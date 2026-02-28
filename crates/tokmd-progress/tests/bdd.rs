//! BDD-style scenario tests for `tokmd-progress`.
//!
//! These tests exercise the public API of [`Progress`] and [`ProgressBarWithEta`]
//! using Given/When/Then style assertions.

use tokmd_progress::{Progress, ProgressBarWithEta};

// ── Progress spinner scenarios ──────────────────────────────────────────────

#[test]
fn scenario_spinner_disabled_does_not_panic() {
    // Given a spinner created with enabled=false
    let spinner = Progress::new(false);

    // When we call every public method
    spinner.set_message("scanning");
    spinner.set_message(String::from("owned string"));
    spinner.finish_and_clear();

    // Then no panic occurs (implicit pass)
}

#[test]
fn scenario_spinner_enabled_in_non_tty_acts_as_noop() {
    // Given a spinner created with enabled=true but running in CI (no TTY)
    let spinner = Progress::new(true);

    // When we drive the full lifecycle
    spinner.set_message("phase 1");
    spinner.set_message("phase 2");
    spinner.finish_and_clear();

    // Then no panic occurs – the implementation gracefully falls back to noop
}

#[test]
fn scenario_spinner_drop_cleans_up() {
    // Given a spinner
    let spinner = Progress::new(false);
    spinner.set_message("working");

    // When the spinner is dropped
    drop(spinner);

    // Then no panic occurs during cleanup
}

#[test]
fn scenario_spinner_finish_then_drop_is_safe() {
    // Given a spinner that is explicitly finished
    let spinner = Progress::new(true);
    spinner.finish_and_clear();

    // When it is also dropped
    drop(spinner);

    // Then double-finish does not panic
}

// ── ProgressBarWithEta scenarios ────────────────────────────────────────────

#[test]
fn scenario_bar_creation_with_zero_total() {
    // Given a bar with total=0
    let bar = ProgressBarWithEta::new(false, 0, "empty");

    // When we interact with it
    bar.inc();
    bar.set_position(0);
    bar.finish_and_clear();

    // Then no panic occurs
}

#[test]
fn scenario_bar_creation_with_large_total() {
    // Given a bar with a very large total
    let bar = ProgressBarWithEta::new(false, u64::MAX, "huge");

    // When we increment
    bar.inc();
    bar.inc_by(1000);
    bar.set_position(u64::MAX / 2);
    bar.finish_and_clear();

    // Then no overflow or panic
}

#[test]
fn scenario_bar_full_lifecycle_disabled() {
    // Given a disabled bar with total=100
    let bar = ProgressBarWithEta::new(false, 100, "scan");

    // When we drive it through a complete lifecycle
    for i in 0..100 {
        bar.inc();
        if i % 25 == 0 {
            bar.set_message(&format!("processing batch {}", i / 25));
        }
    }
    bar.finish_with_message("done");

    // Then all operations complete without error
}

#[test]
fn scenario_bar_full_lifecycle_enabled_non_tty() {
    // Given an enabled bar in a non-TTY environment
    let bar = ProgressBarWithEta::new(true, 50, "files");

    // When we drive it through a complete lifecycle
    for _ in 0..50 {
        bar.inc();
    }
    bar.finish_with_message("complete");

    // Then it gracefully degrades to noop
}

#[test]
fn scenario_bar_inc_by_accumulates() {
    // Given a bar with total=100
    let bar = ProgressBarWithEta::new(false, 100, "batch");

    // When we increment by chunks
    bar.inc_by(25);
    bar.inc_by(25);
    bar.inc_by(50);

    // Then finish succeeds (position tracking is internal)
    bar.finish_and_clear();
}

#[test]
fn scenario_bar_set_position_beyond_total() {
    // Given a bar with total=10
    let bar = ProgressBarWithEta::new(false, 10, "overshoot");

    // When we set position beyond total
    bar.set_position(20);

    // Then no panic occurs
    bar.finish_and_clear();
}

#[test]
fn scenario_bar_set_length_after_creation() {
    // Given a bar with initial total=10
    let bar = ProgressBarWithEta::new(false, 10, "resize");

    // When we update the total
    bar.inc_by(5);
    bar.set_length(100);
    bar.inc_by(95);

    // Then the bar accepts the new length
    bar.finish_with_message("resized");
}

#[test]
fn scenario_bar_set_length_to_zero() {
    // Given a bar with initial total=50
    let bar = ProgressBarWithEta::new(false, 50, "shrink");

    // When we set length to zero
    bar.set_length(0);
    bar.inc();

    // Then no division-by-zero or panic
    bar.finish_and_clear();
}

#[test]
fn scenario_bar_finish_before_any_progress() {
    // Given a bar that was just created
    let bar = ProgressBarWithEta::new(false, 100, "immediate");

    // When we finish immediately
    bar.finish_with_message("skipped");

    // Then no panic occurs
}

#[test]
fn scenario_bar_finish_and_clear_before_any_progress() {
    // Given a bar that was just created
    let bar = ProgressBarWithEta::new(false, 100, "immediate");

    // When we finish and clear immediately
    bar.finish_and_clear();

    // Then no panic occurs
}

#[test]
fn scenario_bar_drop_without_finish() {
    // Given a bar that is never explicitly finished
    let bar = ProgressBarWithEta::new(false, 50, "abandoned");
    bar.inc_by(10);

    // When it is dropped
    drop(bar);

    // Then Drop impl cleans up without panic
}

#[test]
fn scenario_bar_message_updates() {
    // Given a bar
    let bar = ProgressBarWithEta::new(false, 10, "start");

    // When we update the message multiple times
    bar.set_message("phase 1");
    bar.set_message("phase 2");
    bar.set_message("");
    bar.set_message("final phase");

    // Then all updates succeed
    bar.finish_and_clear();
}

#[test]
fn scenario_bar_empty_message() {
    // Given a bar created with an empty message
    let bar = ProgressBarWithEta::new(false, 10, "");

    // When we use it normally
    bar.inc();
    bar.finish_with_message("");

    // Then empty strings are handled gracefully
}
