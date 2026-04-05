//! Integration tests for `tokmd-progress`.
//!
//! These tests exercise realistic workflows and property-based invariants.

use tokmd_progress::{Progress, ProgressBarWithEta};

// ── Workflow: simulated scan pipeline ───────────────────────────────────────

#[test]
fn workflow_scan_pipeline_with_spinner_and_bar() {
    // Phase 1: spinner while discovering files
    let spinner = Progress::new(false);
    spinner.set_message("discovering files…");
    // Simulate discovery
    let file_count: u64 = 42;
    spinner.finish_and_clear();

    // Phase 2: progress bar while processing files
    let bar = ProgressBarWithEta::new(false, file_count, "processing");
    for i in 0..file_count {
        bar.inc();
        if i == file_count / 2 {
            bar.set_message("halfway done");
        }
    }
    bar.finish_with_message("scan complete");
}

#[test]
fn workflow_analysis_preset_pipeline() {
    // Simulate running multiple analysis enrichers with a shared bar
    let enricher_count: u64 = 8;
    let bar = ProgressBarWithEta::new(false, enricher_count, "enrichers");

    let enrichers = [
        "density",
        "cocomo",
        "distribution",
        "todo",
        "hotspots",
        "coupling",
        "freshness",
        "topics",
    ];

    for name in &enrichers {
        bar.set_message(name);
        bar.inc();
    }

    bar.finish_with_message("analysis complete");
}

#[test]
fn workflow_dynamic_total_update() {
    // Simulate a scenario where we discover more work mid-stream
    let bar = ProgressBarWithEta::new(false, 10, "initial");

    bar.inc_by(5);
    // Discovered more items
    bar.set_length(20);
    bar.inc_by(10);
    // Discovered even more
    bar.set_length(30);
    bar.inc_by(15);

    bar.finish_with_message("all done");
}

#[test]
fn workflow_multiple_sequential_spinners() {
    // Simulate multiple sequential phases, each with its own spinner
    for phase in &["scanning", "modeling", "formatting", "writing"] {
        let spinner = Progress::new(false);
        spinner.set_message(*phase);
        spinner.finish_and_clear();
    }
}

#[test]
fn workflow_multiple_sequential_bars() {
    // Simulate multiple sequential progress bars
    for (total, name) in [(100, "files"), (50, "modules"), (10, "languages")] {
        let bar = ProgressBarWithEta::new(false, total, name);
        for _ in 0..total {
            bar.inc();
        }
        bar.finish_and_clear();
    }
}

// ── Noop (disabled) mode ────────────────────────────────────────────────────

#[test]
fn noop_spinner_all_methods_are_safe() {
    let p = Progress::new(false);
    p.set_message("a");
    p.set_message(String::new());
    p.finish_and_clear();
    // double-finish
    p.finish_and_clear();
    drop(p);
}

#[test]
fn noop_bar_all_methods_are_safe() {
    let b = ProgressBarWithEta::new(false, 0, "");
    b.inc();
    b.inc_by(0);
    b.inc_by(u64::MAX);
    b.set_position(0);
    b.set_position(u64::MAX);
    b.set_message("");
    b.set_length(0);
    b.set_length(u64::MAX);
    b.finish_with_message("");
    b.finish_and_clear();
    // double-finish
    b.finish_with_message("again");
    b.finish_and_clear();
    drop(b);
}

// ── Enabled mode in non-TTY (CI) ──────────────────────────────────────────

#[test]
fn enabled_non_tty_spinner_degrades_gracefully() {
    // In CI, enabled=true still produces a noop since stderr is not a TTY
    let p = Progress::new(true);
    p.set_message("should be noop in CI");
    p.finish_and_clear();
}

#[test]
fn enabled_non_tty_bar_degrades_gracefully() {
    let b = ProgressBarWithEta::new(true, 100, "ci test");
    for _ in 0..100 {
        b.inc();
    }
    b.finish_with_message("done");
}

// ── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn edge_case_unicode_messages() {
    let bar = ProgressBarWithEta::new(false, 5, "🚀 starting");
    bar.set_message("处理中…");
    bar.inc();
    bar.set_message("обработка");
    bar.inc();
    bar.set_message("🎉 完了");
    bar.finish_with_message("✅ done");
}

#[test]
fn edge_case_very_long_message() {
    let long_msg = "x".repeat(10_000);
    let bar = ProgressBarWithEta::new(false, 1, &long_msg);
    bar.set_message(&long_msg);
    bar.inc();
    bar.finish_with_message(&long_msg);
}

#[test]
fn edge_case_newlines_in_message() {
    let bar = ProgressBarWithEta::new(false, 1, "line1\nline2\r\nline3");
    bar.set_message("a\nb\nc");
    bar.inc();
    bar.finish_with_message("done\n");
}

#[test]
fn edge_case_rapid_create_drop() {
    // Stress test: create and drop many bars rapidly
    for i in 0..200 {
        let _ = ProgressBarWithEta::new(false, i, "rapid");
    }
}

#[test]
fn edge_case_rapid_spinner_create_drop() {
    for _ in 0..200 {
        let _ = Progress::new(false);
    }
}

// ── Property-based tests ────────────────────────────────────────────────────

mod properties {
    use proptest::prelude::*;
    use tokmd_progress::ProgressBarWithEta;

    proptest! {
        #[test]
        fn bar_never_panics_on_arbitrary_total(total in 0u64..=u64::MAX) {
            let bar = ProgressBarWithEta::new(false, total, "prop");
            bar.inc();
            bar.finish_and_clear();
        }

        #[test]
        fn bar_never_panics_on_arbitrary_inc(delta in 0u64..=1_000_000u64) {
            let bar = ProgressBarWithEta::new(false, 100, "prop");
            bar.inc_by(delta);
            bar.finish_and_clear();
        }

        #[test]
        fn bar_never_panics_on_arbitrary_position(pos in 0u64..=u64::MAX) {
            let bar = ProgressBarWithEta::new(false, 100, "prop");
            bar.set_position(pos);
            bar.finish_and_clear();
        }

        #[test]
        fn bar_never_panics_on_arbitrary_length(len in 0u64..=u64::MAX) {
            let bar = ProgressBarWithEta::new(false, 50, "prop");
            bar.set_length(len);
            bar.inc();
            bar.finish_and_clear();
        }

        #[test]
        fn bar_never_panics_on_arbitrary_message(msg in ".*") {
            let bar = ProgressBarWithEta::new(false, 10, &msg);
            bar.set_message(&msg);
            bar.finish_with_message(&msg);
        }

        #[test]
        fn bar_survives_arbitrary_lifecycle(
            total in 0u64..1000u64,
            increments in proptest::collection::vec(0u64..100u64, 0..50),
        ) {
            let bar = ProgressBarWithEta::new(false, total, "lifecycle");
            for delta in &increments {
                bar.inc_by(*delta);
            }
            bar.finish_and_clear();
        }
    }
}
