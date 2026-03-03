//! Deep tests for tokmd-progress spinner and progress bar abstractions.

use tokmd_progress::{Progress, ProgressBarWithEta};

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
