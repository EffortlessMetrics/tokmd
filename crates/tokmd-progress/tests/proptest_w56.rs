#![allow(clippy::drop_non_drop)]
use proptest::prelude::*;
use tokmd_progress::{Progress, ProgressBarWithEta};

proptest! {
    /// Progress::new never panics regardless of enabled flag.
    #[test]
    fn progress_new_never_panics(enabled in any::<bool>()) {
        let _p = Progress::new(enabled);
    }

    /// set_message never panics with arbitrary strings.
    #[test]
    fn progress_set_message_never_panics(msg in "\\PC{0,200}") {
        let p = Progress::new(false);
        p.set_message(msg);
    }

    /// finish_and_clear never panics when disabled.
    #[test]
    fn progress_finish_never_panics(_dummy in 0..5u8) {
        let p = Progress::new(false);
        p.set_message("working...");
        p.finish_and_clear();
    }

    /// ProgressBarWithEta::new never panics with any total.
    #[test]
    fn progress_bar_new_never_panics(total in 0u64..1_000_000) {
        let _pb = ProgressBarWithEta::new(false, total, "test");
    }

    /// All ProgressBarWithEta methods are safe when disabled.
    #[test]
    fn progress_bar_all_methods_safe(
        total in 0u64..1000,
        pos in 0u64..1000,
        delta in 0u64..100,
        new_len in 0u64..1000,
    ) {
        let pb = ProgressBarWithEta::new(false, total, "scan");
        pb.inc();
        pb.inc_by(delta);
        pb.set_position(pos);
        pb.set_message("updated");
        pb.set_length(new_len);
        pb.finish_with_message("done");
        pb.finish_and_clear();
    }

    /// set_message accepts arbitrary UTF-8 without panicking.
    #[test]
    fn progress_bar_arbitrary_message(msg in "\\PC{0,100}") {
        let pb = ProgressBarWithEta::new(false, 10, "init");
        pb.set_message(&msg);
    }

    /// finish_with_message accepts arbitrary UTF-8 without panicking.
    #[test]
    fn progress_bar_finish_arbitrary_message(msg in "\\PC{0,100}") {
        let pb = ProgressBarWithEta::new(false, 10, "init");
        pb.finish_with_message(&msg);
    }

    /// Creating and dropping progress bars in sequence doesn't panic.
    #[test]
    fn progress_bar_create_drop_cycle(count in 1usize..20) {
        for _ in 0..count {
            let pb = ProgressBarWithEta::new(false, 100, "cycle");
            pb.inc();
            pb.finish_and_clear();
        }
    }

    /// Progress with zero total doesn't panic.
    #[test]
    fn progress_bar_zero_total(_dummy in 0..5u8) {
        let pb = ProgressBarWithEta::new(false, 0, "empty");
        pb.inc();
        pb.inc_by(5);
        pb.set_position(0);
        pb.finish_and_clear();
    }
}
