//! Behaviour tests for [`Progress`] and [`ProgressBarWithEta`].
//!
//! Focuses on creation → update → completion lifecycle, boundary
//! values, and defensive-programming edge cases.

use tokmd_progress::{Progress, ProgressBarWithEta};

// ── Spinner lifecycle ───────────────────────────────────────────────

#[test]
fn spinner_create_message_finish_lifecycle() {
    let s = Progress::new(false);
    s.set_message("discovering");
    s.set_message("modelling");
    s.finish_and_clear();
}

#[test]
fn spinner_enabled_non_tty_lifecycle() {
    // enabled=true but CI has no TTY → graceful noop
    let s = Progress::new(true);
    s.set_message("working");
    s.finish_and_clear();
}

#[test]
fn spinner_set_message_accepts_owned_and_borrowed() {
    let s = Progress::new(false);
    s.set_message("borrowed");
    s.set_message(String::from("owned"));
    s.set_message(format!("{} {}", "formatted", "msg"));
    s.finish_and_clear();
}

#[test]
fn spinner_finish_is_idempotent() {
    let s = Progress::new(false);
    s.finish_and_clear();
    s.finish_and_clear();
    s.finish_and_clear();
}

#[test]
fn spinner_drop_after_finish_is_safe() {
    let s = Progress::new(false);
    s.set_message("work");
    s.finish_and_clear();
    drop(s);
}

#[test]
fn spinner_drop_without_finish_is_safe() {
    let s = Progress::new(false);
    s.set_message("abandoned");
    drop(s);
}

// ── ProgressBarWithEta: creation ────────────────────────────────────

#[test]
fn bar_creation_zero_total() {
    let b = ProgressBarWithEta::new(false, 0, "zero");
    b.inc();
    b.finish_and_clear();
}

#[test]
fn bar_creation_one_total() {
    let b = ProgressBarWithEta::new(false, 1, "one");
    b.inc();
    b.finish_and_clear();
}

#[test]
fn bar_creation_u64_max_total() {
    let b = ProgressBarWithEta::new(false, u64::MAX, "max");
    b.inc();
    b.finish_and_clear();
}

#[test]
fn bar_creation_enabled_non_tty() {
    let b = ProgressBarWithEta::new(true, 50, "ci");
    b.inc();
    b.finish_and_clear();
}

// ── ProgressBarWithEta: update ──────────────────────────────────────

#[test]
fn bar_inc_one_at_a_time() {
    let b = ProgressBarWithEta::new(false, 5, "inc1");
    for _ in 0..5 {
        b.inc();
    }
    b.finish_and_clear();
}

#[test]
fn bar_inc_by_chunks() {
    let b = ProgressBarWithEta::new(false, 100, "chunks");
    b.inc_by(30);
    b.inc_by(30);
    b.inc_by(40);
    b.finish_and_clear();
}

#[test]
fn bar_inc_by_zero_is_noop() {
    let b = ProgressBarWithEta::new(false, 10, "zero-delta");
    b.inc_by(0);
    b.inc_by(0);
    b.finish_and_clear();
}

#[test]
fn bar_set_position_forward_and_backward() {
    let b = ProgressBarWithEta::new(false, 100, "pos");
    b.set_position(50);
    b.set_position(25); // rewind
    b.set_position(100);
    b.finish_and_clear();
}

#[test]
fn bar_set_position_beyond_total() {
    let b = ProgressBarWithEta::new(false, 10, "overshoot");
    b.set_position(999);
    b.finish_and_clear();
}

#[test]
fn bar_set_length_grow_and_shrink() {
    let b = ProgressBarWithEta::new(false, 10, "resize");
    b.set_length(200);
    b.set_length(5);
    b.set_length(0);
    b.finish_and_clear();
}

#[test]
fn bar_set_message_multiple_times() {
    let b = ProgressBarWithEta::new(false, 10, "init");
    b.set_message("phase 1");
    b.set_message("phase 2");
    b.set_message("");
    b.set_message("final");
    b.finish_and_clear();
}

// ── ProgressBarWithEta: completion ──────────────────────────────────

#[test]
fn bar_finish_with_message_before_any_progress() {
    let b = ProgressBarWithEta::new(false, 100, "skip");
    b.finish_with_message("skipped");
}

#[test]
fn bar_finish_and_clear_before_any_progress() {
    let b = ProgressBarWithEta::new(false, 100, "clear");
    b.finish_and_clear();
}

#[test]
fn bar_finish_is_idempotent() {
    let b = ProgressBarWithEta::new(false, 10, "idem");
    b.inc_by(10);
    b.finish_with_message("done");
    b.finish_with_message("again");
    b.finish_and_clear();
}

#[test]
fn bar_drop_without_finish_is_safe() {
    let b = ProgressBarWithEta::new(false, 50, "abandon");
    b.inc_by(10);
    drop(b);
}

// ── Edge cases: very large increments ───────────────────────────────

#[test]
fn bar_inc_by_u64_max_does_not_panic() {
    let b = ProgressBarWithEta::new(false, 10, "huge-inc");
    b.inc_by(u64::MAX);
    b.finish_and_clear();
}

#[test]
fn bar_set_position_u64_max_then_inc() {
    let b = ProgressBarWithEta::new(false, 10, "max-pos");
    b.set_position(u64::MAX);
    b.inc(); // potential overflow
    b.finish_and_clear();
}

#[test]
fn bar_set_length_u64_max() {
    let b = ProgressBarWithEta::new(false, 10, "max-len");
    b.set_length(u64::MAX);
    b.inc();
    b.finish_and_clear();
}

// ── Edge cases: zero total ──────────────────────────────────────────

#[test]
fn bar_zero_total_full_lifecycle() {
    let b = ProgressBarWithEta::new(false, 0, "empty");
    b.inc();
    b.inc_by(100);
    b.set_position(0);
    b.set_message("still empty");
    b.set_length(0);
    b.finish_with_message("done with nothing");
}

#[test]
fn bar_zero_total_set_length_to_nonzero_then_progress() {
    let b = ProgressBarWithEta::new(false, 0, "grow");
    b.set_length(10);
    for _ in 0..10 {
        b.inc();
    }
    b.finish_and_clear();
}

// ── Edge cases: rapid create/drop stress ────────────────────────────

#[test]
fn stress_rapid_bar_create_drop() {
    for i in 0u64..100 {
        let b = ProgressBarWithEta::new(false, i, "stress");
        b.inc();
        drop(b);
    }
}

#[test]
fn stress_rapid_spinner_create_drop() {
    for _ in 0..100 {
        let s = Progress::new(false);
        s.set_message("spin");
        drop(s);
    }
}

// ── Interleaved spinner + bar workflow ──────────────────────────────

#[test]
fn workflow_spinner_then_bar_pipeline() {
    // Phase 1: discover
    let spinner = Progress::new(false);
    spinner.set_message("discovering files");
    let total = 25u64;
    spinner.finish_and_clear();

    // Phase 2: process
    let bar = ProgressBarWithEta::new(false, total, "processing");
    for _ in 0..total {
        bar.inc();
    }
    bar.finish_with_message("complete");
}

#[test]
fn workflow_multiple_bars_sequential() {
    for (label, total) in [("scan", 100u64), ("model", 50), ("format", 10)] {
        let b = ProgressBarWithEta::new(false, total, label);
        for _ in 0..total {
            b.inc();
        }
        b.finish_and_clear();
    }
}

// ── Message edge cases ──────────────────────────────────────────────

#[test]
fn bar_unicode_messages() {
    let b = ProgressBarWithEta::new(false, 3, "🔍 scan");
    b.set_message("处理中…");
    b.inc();
    b.set_message("обработка");
    b.inc();
    b.set_message("完了 ✅");
    b.inc();
    b.finish_with_message("✅");
}

#[test]
fn bar_empty_initial_message() {
    let b = ProgressBarWithEta::new(false, 5, "");
    b.inc();
    b.set_message("now has message");
    b.finish_and_clear();
}

#[test]
fn bar_newlines_in_messages() {
    let b = ProgressBarWithEta::new(false, 1, "line1\nline2");
    b.set_message("a\r\nb");
    b.inc();
    b.finish_with_message("done\n");
}
