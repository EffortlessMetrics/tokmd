//! Depth tests for tokmd-progress crate.

use tokmd_progress::{Progress, ProgressBarWithEta};

// ── 1. Progress spinner – creation ────────────────────────────────

#[test]
fn progress_new_disabled_does_not_panic() {
    let _p = Progress::new(false);
}

#[test]
fn progress_new_enabled_does_not_panic() {
    // In CI / non-TTY, `enabled=true` still creates the struct (noop spinner)
    let _p = Progress::new(true);
}

// ── 2. Progress spinner – set_message ─────────────────────────────

#[test]
fn progress_set_message_empty() {
    let p = Progress::new(false);
    p.set_message("");
}

#[test]
fn progress_set_message_normal() {
    let p = Progress::new(false);
    p.set_message("scanning files…");
}

#[test]
fn progress_set_message_unicode() {
    let p = Progress::new(false);
    p.set_message("分析中 🔍");
}

#[test]
fn progress_set_message_long_string() {
    let p = Progress::new(false);
    p.set_message("x".repeat(10_000));
}

#[test]
fn progress_set_message_with_newlines() {
    let p = Progress::new(false);
    p.set_message("line1\nline2\nline3");
}

#[test]
fn progress_set_message_special_chars() {
    let p = Progress::new(false);
    p.set_message("<script>alert('xss')</script>");
}

#[test]
fn progress_set_message_multiple_times() {
    let p = Progress::new(false);
    for i in 0..100 {
        p.set_message(format!("step {i}"));
    }
}

#[test]
fn progress_set_message_accepts_string() {
    let p = Progress::new(false);
    p.set_message(String::from("owned string"));
}

#[test]
fn progress_set_message_accepts_str() {
    let p = Progress::new(false);
    p.set_message("borrowed str");
}

// ── 3. Progress spinner – finish/clear lifecycle ──────────────────

#[test]
fn progress_finish_and_clear_disabled() {
    let p = Progress::new(false);
    p.finish_and_clear();
}

#[test]
fn progress_finish_after_message() {
    let p = Progress::new(false);
    p.set_message("working…");
    p.finish_and_clear();
}

#[test]
fn progress_double_finish_no_panic() {
    let p = Progress::new(false);
    p.finish_and_clear();
    p.finish_and_clear();
}

#[test]
fn progress_message_after_finish_no_panic() {
    let p = Progress::new(false);
    p.finish_and_clear();
    p.set_message("after finish");
}

#[test]
fn progress_drop_cleans_up() {
    {
        let p = Progress::new(false);
        p.set_message("will be dropped");
    }
    // No panic during drop
}

// ── 4. ProgressBarWithEta – creation ──────────────────────────────

#[test]
fn bar_new_disabled_does_not_panic() {
    let _b = ProgressBarWithEta::new(false, 100, "scan");
}

#[test]
fn bar_new_enabled_does_not_panic() {
    let _b = ProgressBarWithEta::new(true, 100, "scan");
}

#[test]
fn bar_new_zero_total() {
    let b = ProgressBarWithEta::new(false, 0, "empty");
    b.inc();
    b.finish_and_clear();
}

#[test]
fn bar_new_large_total() {
    let b = ProgressBarWithEta::new(false, u64::MAX, "huge");
    b.inc();
    b.finish_and_clear();
}

#[test]
fn bar_new_empty_message() {
    let _b = ProgressBarWithEta::new(false, 10, "");
}

// ── 5. ProgressBarWithEta – inc ───────────────────────────────────

#[test]
fn bar_inc_once() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.inc();
}

#[test]
fn bar_inc_to_completion() {
    let b = ProgressBarWithEta::new(false, 5, "scan");
    for _ in 0..5 {
        b.inc();
    }
    b.finish_and_clear();
}

#[test]
fn bar_inc_past_total() {
    let b = ProgressBarWithEta::new(false, 3, "scan");
    for _ in 0..10 {
        b.inc();
    }
}

#[test]
fn bar_inc_by_zero() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.inc_by(0);
}

#[test]
fn bar_inc_by_large_delta() {
    let b = ProgressBarWithEta::new(false, 100, "scan");
    b.inc_by(50);
    b.inc_by(50);
}

#[test]
fn bar_inc_by_past_total() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.inc_by(100);
}

// ── 6. ProgressBarWithEta – set_position ──────────────────────────

#[test]
fn bar_set_position_zero() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.set_position(0);
}

#[test]
fn bar_set_position_middle() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.set_position(5);
}

#[test]
fn bar_set_position_at_total() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.set_position(10);
}

#[test]
fn bar_set_position_past_total() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.set_position(999);
}

#[test]
fn bar_set_position_backwards() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.set_position(8);
    b.set_position(3);
}

// ── 7. ProgressBarWithEta – set_message ───────────────────────────

#[test]
fn bar_set_message_normal() {
    let b = ProgressBarWithEta::new(false, 10, "init");
    b.set_message("updated");
}

#[test]
fn bar_set_message_empty() {
    let b = ProgressBarWithEta::new(false, 10, "init");
    b.set_message("");
}

#[test]
fn bar_set_message_unicode() {
    let b = ProgressBarWithEta::new(false, 10, "init");
    b.set_message("処理中 ⏳");
}

// ── 8. ProgressBarWithEta – set_length ────────────────────────────

#[test]
fn bar_set_length_increase() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.set_length(20);
}

#[test]
fn bar_set_length_decrease() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.set_length(5);
}

#[test]
fn bar_set_length_zero() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.set_length(0);
}

// ── 9. ProgressBarWithEta – finish ────────────────────────────────

#[test]
fn bar_finish_with_message() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.inc_by(10);
    b.finish_with_message("done!");
}

#[test]
fn bar_finish_and_clear() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.finish_and_clear();
}

#[test]
fn bar_double_finish_no_panic() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.finish_and_clear();
    b.finish_and_clear();
}

#[test]
fn bar_finish_then_inc_no_panic() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.finish_and_clear();
    b.inc();
}

#[test]
fn bar_finish_with_message_then_clear_no_panic() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.finish_with_message("done");
    b.finish_and_clear();
}

// ── 10. ProgressBarWithEta – drop ─────────────────────────────────

#[test]
fn bar_drop_without_finish() {
    {
        let b = ProgressBarWithEta::new(false, 10, "scan");
        b.inc_by(5);
    }
    // No panic during drop
}

#[test]
fn bar_drop_after_finish() {
    {
        let b = ProgressBarWithEta::new(false, 10, "scan");
        b.finish_and_clear();
    }
}

// ── 11. Silent/quiet mode ─────────────────────────────────────────

#[test]
fn progress_disabled_full_lifecycle() {
    let p = Progress::new(false);
    p.set_message("step 1");
    p.set_message("step 2");
    p.set_message("step 3");
    p.finish_and_clear();
}

#[test]
fn bar_disabled_full_lifecycle() {
    let b = ProgressBarWithEta::new(false, 100, "scan");
    b.set_message("phase 1");
    b.inc_by(25);
    b.set_message("phase 2");
    b.inc_by(25);
    b.set_position(75);
    b.set_length(200);
    b.set_message("phase 3");
    b.inc_by(125);
    b.finish_with_message("complete");
}

// ── 12. Stress tests ──────────────────────────────────────────────

#[test]
fn progress_rapid_message_updates() {
    let p = Progress::new(false);
    for i in 0..1_000 {
        p.set_message(format!("iteration {i}"));
    }
    p.finish_and_clear();
}

#[test]
fn bar_rapid_increments() {
    let b = ProgressBarWithEta::new(false, 10_000, "stress");
    for _ in 0..10_000 {
        b.inc();
    }
    b.finish_and_clear();
}

// ── 13. Property tests ────────────────────────────────────────────

mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn progress_set_message_never_panics(msg in ".*") {
            let p = Progress::new(false);
            p.set_message(msg);
            p.finish_and_clear();
        }

        #[test]
        fn bar_any_total_never_panics(total in 0u64..1_000_000) {
            let b = ProgressBarWithEta::new(false, total, "test");
            b.inc();
            b.finish_and_clear();
        }

        #[test]
        fn bar_any_position_never_panics(pos in 0u64..1_000_000) {
            let b = ProgressBarWithEta::new(false, 100, "test");
            b.set_position(pos);
            b.finish_and_clear();
        }

        #[test]
        fn bar_any_delta_never_panics(delta in 0u64..1_000_000) {
            let b = ProgressBarWithEta::new(false, 100, "test");
            b.inc_by(delta);
            b.finish_and_clear();
        }

        #[test]
        fn bar_any_length_never_panics(len in 0u64..1_000_000) {
            let b = ProgressBarWithEta::new(false, 100, "test");
            b.set_length(len);
            b.finish_and_clear();
        }

        #[test]
        fn bar_any_message_never_panics(msg in ".*") {
            let b = ProgressBarWithEta::new(false, 10, "test");
            b.set_message(&msg);
            b.finish_and_clear();
        }

        #[test]
        fn bar_finish_message_never_panics(msg in ".*") {
            let b = ProgressBarWithEta::new(false, 10, "test");
            b.finish_with_message(&msg);
        }
    }
}
