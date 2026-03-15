//! W67 deep tests for tokmd-progress crate.
//!
//! ~15 tests covering: Progress spinner creation, message setting,
//! lifecycle (finish/clear/drop), ProgressBarWithEta creation with various
//! totals, inc/set_position/set_length/finish, quiet mode, stress tests.
//!
//! All tests run with `enabled=false` (no-op mode) so they are deterministic
//! and safe in CI (no TTY required).

use tokmd_progress::{Progress, ProgressBarWithEta};

// ═══════════════════════════════════════════════════════════════════════════
// 1. Progress spinner – creation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_progress_new_disabled() {
    let _p = Progress::new(false);
}

#[test]
fn w67_progress_new_enabled_in_ci() {
    // In non-TTY (CI), enabled=true still succeeds (noop internally)
    let _p = Progress::new(true);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Progress spinner – set_message varieties
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_progress_set_message_str_and_string() {
    let p = Progress::new(false);
    p.set_message("borrowed");
    p.set_message(String::from("owned"));
    p.set_message("");
}

#[test]
fn w67_progress_set_message_unicode() {
    let p = Progress::new(false);
    p.set_message("スキャン中 🔍");
}

#[test]
fn w67_progress_set_message_very_long() {
    let p = Progress::new(false);
    p.set_message("x".repeat(50_000));
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Progress spinner – lifecycle
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_progress_finish_and_clear_no_panic() {
    let p = Progress::new(false);
    p.set_message("working");
    p.finish_and_clear();
}

#[test]
fn w67_progress_double_finish_no_panic() {
    let p = Progress::new(false);
    p.finish_and_clear();
    p.finish_and_clear();
}

#[test]
fn w67_progress_drop_cleans_up() {
    {
        let p = Progress::new(false);
        p.set_message("will be dropped");
    }
    // implicit drop — no panic
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. ProgressBarWithEta – creation edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_bar_zero_total() {
    let b = ProgressBarWithEta::new(false, 0, "empty");
    b.inc();
    b.finish_and_clear();
}

#[test]
fn w67_bar_large_total() {
    let b = ProgressBarWithEta::new(false, u64::MAX, "huge");
    b.inc();
    b.finish_and_clear();
}

#[test]
fn w67_bar_empty_message() {
    let b = ProgressBarWithEta::new(false, 10, "");
    b.finish_and_clear();
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. ProgressBarWithEta – operations
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_bar_inc_by_and_set_position() {
    let b = ProgressBarWithEta::new(false, 100, "scan");
    b.inc();
    b.inc_by(10);
    b.set_position(50);
    b.set_position(0); // backwards
    b.inc_by(0); // zero delta
    b.finish_and_clear();
}

#[test]
fn w67_bar_set_length_and_message() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.set_length(200);
    b.set_length(0);
    b.set_message("updated");
    b.set_message("");
    b.set_message("Unicode 🌍");
    b.finish_with_message("done");
}

#[test]
fn w67_bar_finish_then_operate_no_panic() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.finish_and_clear();
    b.inc();
    b.set_position(5);
    b.set_message("after finish");
    b.finish_with_message("again");
    b.finish_and_clear();
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Quiet / disabled full lifecycle
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_quiet_full_lifecycle() {
    // Spinner
    let p = Progress::new(false);
    for i in 0..50 {
        p.set_message(format!("step {i}"));
    }
    p.finish_and_clear();

    // Bar
    let b = ProgressBarWithEta::new(false, 1000, "batch");
    for _ in 0..1000 {
        b.inc();
    }
    b.finish_with_message("complete");
}
