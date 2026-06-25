//! Progress spinner utilities for long-running operations.

#[cfg(feature = "ui")]
use std::io::IsTerminal;

const PROGRESS_EVENT_NAME: &str = "tokmd.progress";
const PROGRESS_EVENT_SCHEMA_VERSION: u8 = 1;

fn progress_events_enabled() -> bool {
    std::env::var_os("TOKMD_PROGRESS_EVENTS").is_some()
}

fn progress_event_json(kind: &str, message: &str) -> String {
    serde_json::json!({
        "event": PROGRESS_EVENT_NAME,
        "schema_version": PROGRESS_EVENT_SCHEMA_VERSION,
        "kind": kind,
        "message": message,
    })
    .to_string()
}

fn emit_progress_event(kind: &str, message: &str) {
    if progress_events_enabled() {
        eprintln!("{}", progress_event_json(kind, message));
    }
}

/// Check if we should show interactive output.
#[cfg(feature = "ui")]
fn is_interactive() -> bool {
    // Check if stderr is a TTY (since the spinner writes to stderr).
    if !std::io::stderr().is_terminal() {
        return false;
    }

    // Respect standard and tool-specific controls.
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    if std::env::var("TOKMD_NO_PROGRESS").is_ok() {
        return false;
    }

    true
}

#[cfg(feature = "ui")]
mod ui_impl {
    use super::{emit_progress_event, is_interactive};
    use indicatif::{ProgressBar, ProgressStyle};
    use std::time::Duration;

    /// A progress indicator that wraps indicatif.
    pub struct Progress {
        bar: Option<ProgressBar>,
    }

    impl Progress {
        /// Create a new progress indicator.
        ///
        /// The spinner is only shown if:
        /// - `enabled` is true
        /// - stderr is a TTY
        /// - NO_COLOR env var is not set
        /// - TOKMD_NO_PROGRESS env var is not set
        pub fn new(enabled: bool) -> Self {
            let should_show = enabled && is_interactive();

            let bar = if should_show {
                let pb = ProgressBar::new_spinner();
                pb.set_style(
                    ProgressStyle::with_template("{spinner:.cyan} {msg}")
                        .expect("progress template is static and must be valid")
                        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", " "]),
                );
                pb.enable_steady_tick(Duration::from_millis(80));
                Some(pb)
            } else {
                None
            };

            Self { bar }
        }

        /// Set the progress message.
        pub fn set_message(&self, msg: impl Into<String>) {
            let msg = msg.into();
            emit_progress_event("update", &msg);
            if let Some(bar) = &self.bar {
                bar.set_message(msg);
            }
        }

        /// Finish and clear the spinner.
        pub fn finish_and_clear(&self) {
            emit_progress_event("finish", "done");
            if let Some(bar) = &self.bar {
                bar.finish_and_clear();
            }
        }
    }

    impl Drop for Progress {
        fn drop(&mut self) {
            if let Some(bar) = &self.bar {
                bar.finish_and_clear();
            }
        }
    }

    /// A progress bar with ETA support for long-running operations.
    #[cfg(test)]
    pub struct ProgressBarWithEta {
        bar: Option<indicatif::ProgressBar>,
    }

    #[cfg(test)]
    impl ProgressBarWithEta {
        /// Create a new progress bar with ETA.
        pub fn new(enabled: bool, total: u64, message: &str) -> Self {
            let should_show = enabled && is_interactive();

            let bar = if should_show {
                let pb = indicatif::ProgressBar::new(total);
                pb.set_style(
                    ProgressStyle::with_template(
                        "{spinner:.cyan} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}",
                    )
                    .expect("progress template is static and must be valid"),
                );
                pb.set_message(message.to_string());
                pb.enable_steady_tick(Duration::from_millis(100));
                Some(pb)
            } else {
                None
            };

            Self { bar }
        }

        /// Increment the progress by 1.
        pub fn inc(&self) {
            if let Some(bar) = &self.bar {
                bar.inc(1);
            }
        }

        /// Increment the progress by a specific amount.
        pub fn inc_by(&self, delta: u64) {
            if let Some(bar) = &self.bar {
                bar.inc(delta);
            }
        }

        /// Set the current progress position.
        pub fn set_position(&self, pos: u64) {
            if let Some(bar) = &self.bar {
                bar.set_position(pos);
            }
        }

        /// Set the progress message.
        pub fn set_message(&self, msg: &str) {
            emit_progress_event("update", msg);
            if let Some(bar) = &self.bar {
                bar.set_message(msg.to_string());
            }
        }

        /// Update the total length.
        pub fn set_length(&self, len: u64) {
            if let Some(bar) = &self.bar {
                bar.set_length(len);
            }
        }

        /// Finish the progress bar with a message.
        pub fn finish_with_message(&self, msg: &str) {
            emit_progress_event("finish", msg);
            if let Some(bar) = &self.bar {
                bar.finish_with_message(msg.to_string());
            }
        }

        /// Finish and clear the progress bar.
        pub fn finish_and_clear(&self) {
            emit_progress_event("finish", "done");
            if let Some(bar) = &self.bar {
                bar.finish_and_clear();
            }
        }
    }

    #[cfg(test)]
    impl Drop for ProgressBarWithEta {
        fn drop(&mut self) {
            if let Some(bar) = &self.bar {
                bar.finish_and_clear();
            }
        }
    }
}

#[cfg(not(feature = "ui"))]
mod ui_impl {
    use super::emit_progress_event;

    /// A no-op progress indicator when the `ui` feature is disabled.
    pub struct Progress;

    impl Progress {
        /// Create a new progress indicator (no-op without `ui` feature).
        pub fn new(_enabled: bool) -> Self {
            Self
        }

        /// Set the progress message (no-op without `ui` feature).
        pub fn set_message(&self, msg: impl Into<String>) {
            let msg = msg.into();
            emit_progress_event("update", &msg);
        }

        /// Finish and clear the spinner (no-op without `ui` feature).
        pub fn finish_and_clear(&self) {
            emit_progress_event("finish", "done");
        }
    }

    /// A no-op progress bar when `ui` feature is disabled.
    #[cfg(test)]
    pub struct ProgressBarWithEta;

    #[cfg(test)]
    impl ProgressBarWithEta {
        /// Create a new progress bar (no-op without `ui` feature).
        pub fn new(_enabled: bool, _total: u64, _message: &str) -> Self {
            Self
        }

        /// Increment the progress (no-op without `ui` feature).
        pub fn inc(&self) {}

        /// Increment the progress by a specific amount (no-op without `ui` feature).
        pub fn inc_by(&self, _delta: u64) {}

        /// Set the current progress position (no-op without `ui` feature).
        pub fn set_position(&self, _pos: u64) {}

        /// Set the progress message (no-op without `ui` feature).
        pub fn set_message(&self, msg: &str) {
            emit_progress_event("update", msg);
        }

        /// Update the total length (no-op without `ui` feature).
        pub fn set_length(&self, _len: u64) {}

        /// Finish the progress bar (no-op without `ui` feature).
        pub fn finish_with_message(&self, msg: &str) {
            emit_progress_event("finish", msg);
        }

        /// Finish and clear the progress bar (no-op without `ui` feature).
        pub fn finish_and_clear(&self) {
            emit_progress_event("finish", "done");
        }
    }
}

pub use ui_impl::Progress;
#[cfg(test)]
use ui_impl::ProgressBarWithEta;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_methods_do_not_panic_when_disabled() {
        let progress = Progress::new(false);
        progress.set_message("test");
        progress.finish_and_clear();
    }

    #[test]
    fn progress_bar_methods_do_not_panic_when_disabled() {
        let progress = ProgressBarWithEta::new(false, 10, "scan");
        progress.inc();
        progress.inc_by(2);
        progress.set_position(3);
        progress.set_message("updated");
        progress.set_length(20);
        progress.finish_with_message("done");
        progress.finish_and_clear();
    }

    #[test]
    fn progress_event_json_is_stable_and_parseable() {
        let line = progress_event_json("update", "Scanning codebase...");
        let parsed: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(parsed["event"], "tokmd.progress");
        assert_eq!(parsed["schema_version"], 1);
        assert_eq!(parsed["kind"], "update");
        assert_eq!(parsed["message"], "Scanning codebase...");
    }

    #[test]
    fn progress_event_json_escapes_control_characters() {
        let line = progress_event_json("update", "line one\nline two");
        assert!(!line.contains('\n'));
        let parsed: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(parsed["message"], "line one\nline two");
    }

    #[test]
    fn progress_event_fixtures_match_emitted_json() {
        let cases = [
            (
                include_str!("../../../fixtures/progress-events/update.json"),
                "update",
                "Scanning codebase...",
            ),
            (
                include_str!("../../../fixtures/progress-events/finish.json"),
                "finish",
                "done",
            ),
        ];

        for (fixture, kind, message) in cases {
            assert_eq!(fixture.trim(), progress_event_json(kind, message));
        }
    }
}
