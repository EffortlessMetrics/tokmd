#![allow(clippy::drop_non_drop)]
//! W59 – Depth tests for `tokmd-progress` no-op paths.
//!
//! All tests use `enabled = false` (or rely on non-TTY CI) so that the no-op
//! code paths are exercised without requiring a real terminal.

use tokmd_progress::{Progress, ProgressBarWithEta};

// ═══════════════════════════════════════════════════════════════════════
// Progress (spinner) – disabled
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn spinner_new_disabled() {
    let _p = Progress::new(false);
}

#[test]
fn spinner_set_message_str_ref() {
    let p = Progress::new(false);
    p.set_message("hello");
}

#[test]
fn spinner_set_message_owned_string() {
    let p = Progress::new(false);
    p.set_message(String::from("owned"));
}

#[test]
fn spinner_set_message_empty() {
    let p = Progress::new(false);
    p.set_message("");
}

#[test]
fn spinner_finish_and_clear() {
    let p = Progress::new(false);
    p.finish_and_clear();
}

#[test]
fn spinner_finish_is_idempotent() {
    let p = Progress::new(false);
    p.finish_and_clear();
    p.finish_and_clear();
    p.finish_and_clear();
}

#[test]
fn spinner_message_after_finish() {
    let p = Progress::new(false);
    p.finish_and_clear();
    p.set_message("after finish"); // should not panic
}

#[test]
fn spinner_drop_is_safe() {
    let p = Progress::new(false);
    p.set_message("about to drop");
    drop(p);
}

#[test]
fn spinner_rapid_messages() {
    let p = Progress::new(false);
    for i in 0..1_000 {
        p.set_message(format!("step {i}"));
    }
    p.finish_and_clear();
}

// ═══════════════════════════════════════════════════════════════════════
// Progress (spinner) – enabled in non-TTY (acts as no-op)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn spinner_enabled_nontty_does_not_panic() {
    let p = Progress::new(true);
    p.set_message("scan");
    p.finish_and_clear();
}

#[test]
fn spinner_enabled_nontty_drop() {
    let p = Progress::new(true);
    p.set_message("will drop");
    drop(p);
}

// ═══════════════════════════════════════════════════════════════════════
// ProgressBarWithEta – disabled
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn bar_new_disabled_zero_total() {
    let _b = ProgressBarWithEta::new(false, 0, "empty");
}

#[test]
fn bar_new_disabled_large_total() {
    let _b = ProgressBarWithEta::new(false, u64::MAX, "huge");
}

#[test]
fn bar_inc_disabled() {
    let b = ProgressBarWithEta::new(false, 10, "t");
    b.inc();
    b.inc();
    b.inc();
}

#[test]
fn bar_inc_by_disabled() {
    let b = ProgressBarWithEta::new(false, 100, "t");
    b.inc_by(0);
    b.inc_by(1);
    b.inc_by(50);
    b.inc_by(u64::MAX);
}

#[test]
fn bar_set_position_zero() {
    let b = ProgressBarWithEta::new(false, 10, "t");
    b.set_position(0);
}

#[test]
fn bar_set_position_at_total() {
    let b = ProgressBarWithEta::new(false, 10, "t");
    b.set_position(10);
}

#[test]
fn bar_set_position_beyond_total() {
    let b = ProgressBarWithEta::new(false, 10, "t");
    b.set_position(999);
}

#[test]
fn bar_set_message_disabled() {
    let b = ProgressBarWithEta::new(false, 5, "t");
    b.set_message("updated");
    b.set_message("");
}

#[test]
fn bar_set_length_disabled() {
    let b = ProgressBarWithEta::new(false, 5, "t");
    b.set_length(0);
    b.set_length(100);
    b.set_length(u64::MAX);
}

#[test]
fn bar_finish_with_message_disabled() {
    let b = ProgressBarWithEta::new(false, 5, "t");
    b.finish_with_message("done");
}

#[test]
fn bar_finish_and_clear_disabled() {
    let b = ProgressBarWithEta::new(false, 5, "t");
    b.finish_and_clear();
}

#[test]
fn bar_finish_is_idempotent() {
    let b = ProgressBarWithEta::new(false, 10, "t");
    b.finish_and_clear();
    b.finish_and_clear();
    b.finish_and_clear();
}

#[test]
fn bar_ops_after_finish() {
    let b = ProgressBarWithEta::new(false, 10, "t");
    b.finish_and_clear();
    // All of these should be no-ops, not panics
    b.inc();
    b.inc_by(5);
    b.set_position(7);
    b.set_message("post-finish");
    b.set_length(20);
}

#[test]
fn bar_drop_is_safe() {
    let b = ProgressBarWithEta::new(false, 10, "t");
    b.inc_by(3);
    drop(b);
}

#[test]
fn bar_full_lifecycle() {
    let b = ProgressBarWithEta::new(false, 100, "lifecycle");
    b.set_message("starting");
    for _ in 0..10 {
        b.inc();
    }
    b.inc_by(40);
    b.set_position(80);
    b.set_length(200);
    b.set_message("halfway");
    b.inc_by(120);
    b.finish_with_message("complete");
}

// ═══════════════════════════════════════════════════════════════════════
// ProgressBarWithEta – enabled in non-TTY
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn bar_enabled_nontty_does_not_panic() {
    let b = ProgressBarWithEta::new(true, 50, "scan");
    b.inc();
    b.set_message("processing");
    b.finish_and_clear();
}

#[test]
fn bar_enabled_nontty_drop() {
    let b = ProgressBarWithEta::new(true, 50, "scan");
    b.inc_by(10);
    drop(b);
}

// ═══════════════════════════════════════════════════════════════════════
// Edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn bar_empty_message_string() {
    let b = ProgressBarWithEta::new(false, 10, "");
    b.set_message("");
    b.finish_with_message("");
}

#[test]
fn bar_unicode_messages() {
    let b = ProgressBarWithEta::new(false, 10, "スキャン");
    b.set_message("処理中…");
    b.finish_with_message("完了 ✅");
}

#[test]
fn spinner_unicode_messages() {
    let p = Progress::new(false);
    p.set_message("日本語テスト");
    p.set_message("🚀 launching");
    p.finish_and_clear();
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
fn multiple_spinners_coexist() {
    let s1 = Progress::new(false);
    let s2 = Progress::new(false);
    s1.set_message("one");
    s2.set_message("two");
    s1.finish_and_clear();
    s2.finish_and_clear();
}

#[test]
fn spinner_and_bar_coexist() {
    let spinner = Progress::new(false);
    let bar = ProgressBarWithEta::new(false, 10, "bar");
    spinner.set_message("scanning");
    bar.inc_by(5);
    spinner.finish_and_clear();
    bar.finish_and_clear();
}
