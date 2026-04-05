use tokmd_progress::{Progress, ProgressBarWithEta};

// ---------------------------------------------------------------------------
// Progress (spinner) tests — always test with enabled=false since CI has no TTY
// ---------------------------------------------------------------------------

#[test]
fn progress_new_disabled_does_not_panic() {
    let _p = Progress::new(false);
}

#[test]
fn progress_set_message_disabled_is_noop() {
    let p = Progress::new(false);
    p.set_message("scanning");
    p.set_message(String::from("owned string"));
}

#[test]
fn progress_finish_and_clear_disabled_is_noop() {
    let p = Progress::new(false);
    p.finish_and_clear();
}

#[test]
fn progress_drop_does_not_panic() {
    let p = Progress::new(false);
    p.set_message("about to drop");
    drop(p);
}

// ---------------------------------------------------------------------------
// ProgressBarWithEta tests — always test with enabled=false
// ---------------------------------------------------------------------------

#[test]
fn bar_new_disabled_does_not_panic() {
    let _b = ProgressBarWithEta::new(false, 100, "files");
}

#[test]
fn bar_inc_and_inc_by_disabled_is_noop() {
    let b = ProgressBarWithEta::new(false, 50, "items");
    b.inc();
    b.inc();
    b.inc_by(5);
}

#[test]
fn bar_set_position_disabled_is_noop() {
    let b = ProgressBarWithEta::new(false, 10, "scan");
    b.set_position(0);
    b.set_position(5);
    b.set_position(10);
}

#[test]
fn bar_set_message_and_length_disabled_is_noop() {
    let b = ProgressBarWithEta::new(false, 20, "init");
    b.set_message("processing");
    b.set_length(40);
}

#[test]
fn bar_finish_with_message_disabled_is_noop() {
    let b = ProgressBarWithEta::new(false, 5, "task");
    b.finish_with_message("done");
}

#[test]
fn bar_finish_and_clear_disabled_is_noop() {
    let b = ProgressBarWithEta::new(false, 5, "task");
    b.finish_and_clear();
}

#[test]
fn bar_drop_does_not_panic() {
    let b = ProgressBarWithEta::new(false, 10, "drop-test");
    b.inc_by(3);
    drop(b);
}

#[test]
fn bar_full_lifecycle_disabled() {
    let b = ProgressBarWithEta::new(false, 100, "lifecycle");
    b.set_message("starting");
    b.inc();
    b.inc_by(10);
    b.set_position(50);
    b.set_length(200);
    b.set_message("halfway");
    b.finish_with_message("complete");
}
