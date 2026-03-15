#![allow(clippy::drop_non_drop)]
//! W59 – Property-based tests for `tokmd-progress`.
//!
//! Since progress bars are no-op without a TTY, these tests verify that
//! arbitrary inputs never cause panics.

use proptest::prelude::*;
use tokmd_progress::{Progress, ProgressBarWithEta};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    // ── Progress (spinner) ─────────────────────────────────────────────

    #[test]
    fn spinner_any_message_no_panic(msg in ".*{0,100}") {
        let p = Progress::new(false);
        p.set_message(msg);
        p.finish_and_clear();
    }

    #[test]
    fn spinner_enabled_any_message_no_panic(msg in ".*{0,50}") {
        // Even with enabled=true, non-TTY CI should be safe.
        let p = Progress::new(true);
        p.set_message(msg);
        p.finish_and_clear();
    }

    #[test]
    fn spinner_rapid_set_no_panic(count in 0u32..500) {
        let p = Progress::new(false);
        for i in 0..count {
            p.set_message(format!("m{i}"));
        }
        p.finish_and_clear();
    }

    // ── ProgressBarWithEta ─────────────────────────────────────────────

    #[test]
    fn bar_any_total_no_panic(total in 0u64..=u64::MAX) {
        let b = ProgressBarWithEta::new(false, total, "test");
        b.inc();
        b.finish_and_clear();
    }

    #[test]
    fn bar_any_message_no_panic(msg in ".*{0,100}") {
        let b = ProgressBarWithEta::new(false, 10, &msg);
        b.set_message(&msg);
        b.finish_with_message(&msg);
    }

    #[test]
    fn bar_any_position_no_panic(pos in 0u64..=u64::MAX) {
        let b = ProgressBarWithEta::new(false, 100, "t");
        b.set_position(pos);
        b.finish_and_clear();
    }

    #[test]
    fn bar_any_length_no_panic(len in 0u64..=u64::MAX) {
        let b = ProgressBarWithEta::new(false, 10, "t");
        b.set_length(len);
        b.finish_and_clear();
    }

    #[test]
    fn bar_any_inc_delta_no_panic(delta in 0u64..=u64::MAX) {
        let b = ProgressBarWithEta::new(false, 100, "t");
        b.inc_by(delta);
        b.finish_and_clear();
    }

    #[test]
    fn bar_lifecycle_no_panic(
        total in 1u64..10_000,
        steps in 0u32..100,
        msg in "[A-Za-z ]{0,30}",
    ) {
        let b = ProgressBarWithEta::new(false, total, &msg);
        for _ in 0..steps {
            b.inc();
        }
        b.set_message(&msg);
        b.finish_and_clear();
    }

    #[test]
    fn bar_position_beyond_total_no_panic(
        total in 1u64..1000,
        pos in 1000u64..u64::MAX,
    ) {
        let b = ProgressBarWithEta::new(false, total, "t");
        b.set_position(pos);
        b.finish_and_clear();
    }
}
