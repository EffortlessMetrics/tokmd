//! Deep tests for tokmd-progress (W76).
//!
//! ~10 tests covering: Progress spinner creation (disabled/enabled-in-CI),
//! message variants, lifecycle (finish/drop), ProgressBarWithEta creation
//! edge cases, bar operations, and full quiet-mode lifecycle.
//!
//! All tests run with `enabled=false` (no-op mode) so they are deterministic
//! and safe in CI (no TTY required).

use tokmd_progress::{Progress, ProgressBarWithEta};

// ===========================================================================
// 1. Progress spinner - creation
// ===========================================================================

#[test]
fn w76_progress_disabled_creates_without_panic() {
    let _p = Progress::new(false);
}

#[test]
fn w76_progress_enabled_in_ci_creates_without_panic() {
    // In non-TTY CI, enabled=true still works (noop internally).
    let _p = Progress::new(true);
}

// ===========================================================================
// 2. Progress spinner - message variants
// ===========================================================================

#[test]
fn w76_progress_set_message_borrowed_owned_empty() {
    let p = Progress::new(false);
    p.set_message("literal &str");
    p.set_message(String::from("owned String"));
    p.set_message("");
}

#[test]
fn w76_progress_set_message_unicode_and_long() {
    let p = Progress::new(false);
    p.set_message("analysing...");
    p.set_message("a".repeat(100_000));
}

// ===========================================================================
// 3. Progress spinner - lifecycle (finish, double-finish, drop)
// ===========================================================================

#[test]
fn w76_progress_finish_and_clear_no_panic() {
    let p = Progress::new(false);
    p.set_message("working");
    p.finish_and_clear();
}

#[test]
fn w76_progress_double_finish_is_safe() {
    let p = Progress::new(false);
    p.finish_and_clear();
    p.finish_and_clear();
}

#[test]
fn w76_progress_drop_does_not_panic() {
    {
        let p = Progress::new(false);
        p.set_message("about to drop");
    }
    // implicit drop - no panic
}

// ===========================================================================
// 4. ProgressBarWithEta - creation edge cases
// ===========================================================================

#[test]
fn w76_bar_zero_total_is_valid() {
    let b = ProgressBarWithEta::new(false, 0, "nothing");
    b.inc();
    b.finish_and_clear();
}

#[test]
fn w76_bar_max_total_is_valid() {
    let b = ProgressBarWithEta::new(false, u64::MAX, "enormous");
    b.inc();
    b.finish_and_clear();
}

// ===========================================================================
// 5. ProgressBarWithEta - operations after finish
// ===========================================================================

#[test]
fn w76_bar_full_operation_sequence() {
    let b = ProgressBarWithEta::new(false, 50, "scan");
    b.inc();
    b.inc_by(5);
    b.set_position(20);
    b.set_length(100);
    b.set_message("halfway");
    b.finish_with_message("complete");
    // Operations after finish must not panic.
    b.inc();
    b.set_position(0);
    b.set_message("post-finish");
    b.finish_and_clear();
}

// ===========================================================================
// 6. Full quiet-mode lifecycle (spinner + bar together)
// ===========================================================================

#[test]
fn w76_quiet_full_lifecycle_spinner_and_bar() {
    // Spinner
    let p = Progress::new(false);
    for i in 0..100 {
        p.set_message(format!("step {i}"));
    }
    p.finish_and_clear();

    // Bar
    let b = ProgressBarWithEta::new(false, 500, "batch");
    for _ in 0..500 {
        b.inc();
    }
    b.finish_with_message("done");
}
