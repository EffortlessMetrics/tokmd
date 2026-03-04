//! Wave-49 deep tests for tokmd-progress: spinner creation, progress bar
//! lifecycle, message updates, completion, property tests, and edge cases.

use proptest::prelude::*;
use tokmd_progress::{Progress, ProgressBarWithEta};

// ── Progress spinner creation and state ──────────────────────────────────

#[test]
fn spinner_disabled_creation() {
    let _p = Progress::new(false);
}

#[test]
fn spinner_enabled_non_tty_creation() {
    // In test/CI, stderr is not a TTY so this degrades to no-op.
    let _p = Progress::new(true);
}

#[test]
fn spinner_disabled_set_message_str_ref() {
    let p = Progress::new(false);
    p.set_message("scanning...");
}

#[test]
fn spinner_disabled_set_message_owned_string() {
    let p = Progress::new(false);
    p.set_message(String::from("owned message"));
}

#[test]
fn spinner_disabled_set_message_empty() {
    let p = Progress::new(false);
    p.set_message("");
}

#[test]
fn spinner_set_message_unicode() {
    let p = Progress::new(false);
    p.set_message("分析中… 🔍");
    p.set_message("完了 ✅");
}

#[test]
fn spinner_set_message_very_long() {
    let p = Progress::new(false);
    let long_msg = "x".repeat(10_000);
    p.set_message(long_msg);
}

// ── Spinner completion handling ──────────────────────────────────────────

#[test]
fn spinner_finish_and_clear_once() {
    let p = Progress::new(false);
    p.finish_and_clear();
}

#[test]
fn spinner_finish_idempotent() {
    let p = Progress::new(false);
    p.finish_and_clear();
    p.finish_and_clear();
    p.finish_and_clear();
}

#[test]
fn spinner_finish_after_messages() {
    let p = Progress::new(false);
    p.set_message("step 1");
    p.set_message("step 2");
    p.set_message("step 3");
    p.finish_and_clear();
}

#[test]
fn spinner_drop_without_finish() {
    let p = Progress::new(false);
    p.set_message("about to drop");
    drop(p);
}

#[test]
fn spinner_enabled_drop_in_non_tty() {
    {
        let p = Progress::new(true);
        p.set_message("test");
    }
    // No panic
}

// ── Progress bar creation and increment ──────────────────────────────────

#[test]
fn bar_disabled_creation() {
    let _bar = ProgressBarWithEta::new(false, 100, "test");
}

#[test]
fn bar_enabled_non_tty_creation() {
    let _bar = ProgressBarWithEta::new(true, 100, "test");
}

#[test]
fn bar_inc_single() {
    let bar = ProgressBarWithEta::new(false, 10, "scan");
    bar.inc();
}

#[test]
fn bar_inc_by_amount() {
    let bar = ProgressBarWithEta::new(false, 100, "processing");
    bar.inc_by(25);
}

#[test]
fn bar_inc_by_zero() {
    let bar = ProgressBarWithEta::new(false, 10, "test");
    bar.inc_by(0);
}

#[test]
fn bar_inc_by_u64_max() {
    let bar = ProgressBarWithEta::new(false, u64::MAX, "huge");
    bar.inc_by(u64::MAX);
}

#[test]
fn bar_set_position() {
    let bar = ProgressBarWithEta::new(false, 100, "test");
    bar.set_position(50);
}

#[test]
fn bar_sequential_inc_to_total() {
    let bar = ProgressBarWithEta::new(false, 20, "counting");
    for _ in 0..20 {
        bar.inc();
    }
    bar.finish_and_clear();
}

#[test]
fn bar_inc_beyond_total() {
    // Should not panic even when exceeding total
    let bar = ProgressBarWithEta::new(false, 5, "overflow");
    for _ in 0..10 {
        bar.inc();
    }
    bar.finish_and_clear();
}

// ── Message updates ──────────────────────────────────────────────────────

#[test]
fn bar_set_message_str_ref() {
    let bar = ProgressBarWithEta::new(false, 10, "initial");
    bar.set_message("updated");
}

#[test]
fn bar_set_message_empty() {
    let bar = ProgressBarWithEta::new(false, 10, "");
    bar.set_message("");
}

#[test]
fn bar_set_message_unicode() {
    let bar = ProgressBarWithEta::new(false, 10, "スキャン");
    bar.set_message("完了 ✅");
}

#[test]
fn bar_set_message_long() {
    let bar = ProgressBarWithEta::new(false, 10, "test");
    bar.set_message(&"y".repeat(5_000));
}

#[test]
fn bar_set_length() {
    let bar = ProgressBarWithEta::new(false, 10, "resize");
    bar.set_length(100);
}

#[test]
fn bar_set_length_to_zero() {
    let bar = ProgressBarWithEta::new(false, 100, "shrink");
    bar.set_length(0);
    bar.inc(); // should not panic after setting length to 0
}

#[test]
fn bar_set_length_increase_midway() {
    let bar = ProgressBarWithEta::new(false, 10, "test");
    bar.inc_by(5);
    bar.set_length(20);
    bar.inc_by(10);
    bar.finish_and_clear();
}

// ── Completion handling ──────────────────────────────────────────────────

#[test]
fn bar_finish_with_message() {
    let bar = ProgressBarWithEta::new(false, 10, "test");
    bar.finish_with_message("done");
}

#[test]
fn bar_finish_and_clear() {
    let bar = ProgressBarWithEta::new(false, 10, "test");
    bar.finish_and_clear();
}

#[test]
fn bar_finish_idempotent() {
    let bar = ProgressBarWithEta::new(false, 10, "test");
    bar.finish_and_clear();
    bar.finish_and_clear();
    bar.finish_with_message("also done");
}

#[test]
fn bar_drop_without_finish() {
    let bar = ProgressBarWithEta::new(false, 10, "will drop");
    bar.inc_by(3);
    drop(bar);
}

#[test]
fn bar_enabled_drop_in_non_tty() {
    {
        let bar = ProgressBarWithEta::new(true, 50, "test");
        bar.inc();
    }
    // No panic
}

// ── Edge cases: zero total ───────────────────────────────────────────────

#[test]
fn bar_zero_total_inc() {
    let bar = ProgressBarWithEta::new(false, 0, "zero");
    bar.inc();
    bar.finish_and_clear();
}

#[test]
fn bar_zero_total_set_position() {
    let bar = ProgressBarWithEta::new(false, 0, "zero");
    bar.set_position(0);
    bar.finish_and_clear();
}

#[test]
fn bar_zero_total_finish_with_message() {
    let bar = ProgressBarWithEta::new(false, 0, "zero");
    bar.finish_with_message("nothing to do");
}

// ── Edge cases: very large total ─────────────────────────────────────────

#[test]
fn bar_u64_max_total() {
    let bar = ProgressBarWithEta::new(false, u64::MAX, "max");
    bar.inc();
    bar.inc_by(1_000_000);
    bar.set_position(u64::MAX / 2);
    bar.finish_and_clear();
}

#[test]
fn bar_large_total_rapid_increments() {
    let bar = ProgressBarWithEta::new(false, 1_000_000, "rapid");
    for _ in 0..1_000 {
        bar.inc_by(1_000);
    }
    bar.finish_and_clear();
}

// ── Multiple instances ───────────────────────────────────────────────────

#[test]
fn multiple_spinners_coexist() {
    let s1 = Progress::new(false);
    let s2 = Progress::new(false);
    let s3 = Progress::new(false);
    s1.set_message("one");
    s2.set_message("two");
    s3.set_message("three");
    s1.finish_and_clear();
    s2.finish_and_clear();
    s3.finish_and_clear();
}

#[test]
fn multiple_bars_coexist() {
    let b1 = ProgressBarWithEta::new(false, 10, "first");
    let b2 = ProgressBarWithEta::new(false, 20, "second");
    b1.inc();
    b2.inc_by(5);
    b1.finish_and_clear();
    b2.finish_and_clear();
}

#[test]
fn spinner_and_bar_coexist() {
    let spinner = Progress::new(false);
    let bar = ProgressBarWithEta::new(false, 50, "files");
    spinner.set_message("scanning...");
    for _ in 0..50 {
        bar.inc();
    }
    bar.finish_with_message("complete");
    spinner.finish_and_clear();
}

// ── Property tests ───────────────────────────────────────────────────────

proptest! {
    #[test]
    fn progress_bar_never_panics_on_inc(total in 0u64..10_000, steps in 0u64..10_000) {
        let bar = ProgressBarWithEta::new(false, total, "prop");
        for _ in 0..steps {
            bar.inc();
        }
        bar.finish_and_clear();
    }

    #[test]
    fn progress_bar_never_panics_on_inc_by(total in 0u64..10_000, delta in 0u64..10_000) {
        let bar = ProgressBarWithEta::new(false, total, "prop");
        bar.inc_by(delta);
        bar.finish_and_clear();
    }

    #[test]
    fn progress_bar_never_panics_on_set_position(total in 0u64..10_000, pos in 0u64..10_000) {
        let bar = ProgressBarWithEta::new(false, total, "prop");
        bar.set_position(pos);
        bar.finish_and_clear();
    }

    #[test]
    fn progress_bar_never_panics_on_set_length(
        initial in 0u64..10_000,
        new_len in 0u64..10_000,
    ) {
        let bar = ProgressBarWithEta::new(false, initial, "prop");
        bar.set_length(new_len);
        bar.finish_and_clear();
    }

    #[test]
    fn spinner_never_panics_with_arbitrary_messages(msg in "\\PC{0,200}") {
        let p = Progress::new(false);
        p.set_message(msg);
        p.finish_and_clear();
    }
}
