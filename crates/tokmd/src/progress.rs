//! Progress spinner utilities for long-running operations.

#[cfg(feature = "ui")]
use std::io::IsTerminal;

/// Check if we should show interactive output.
#[cfg(feature = "ui")]
fn is_interactive() -> bool {
    // Check if stderr is a TTY (since the spinner writes to stderr)
    if !std::io::stderr().is_terminal() {
        return false;
    }

    // Check NO_COLOR env var
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check TOKMD_NO_PROGRESS env var
    if std::env::var("TOKMD_NO_PROGRESS").is_ok() {
        return false;
    }

    true
}

#[cfg(feature = "ui")]
mod ui_impl {
    use super::is_interactive;
    use indicatif::{ProgressBar, ProgressStyle};
    use std::time::{Duration, Instant};

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
                        .unwrap()
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
            if let Some(bar) = &self.bar {
                bar.set_message(msg.into());
            }
        }

        /// Finish and clear the spinner.
        pub fn finish_and_clear(&self) {
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
    #[allow(dead_code)]
    pub struct ProgressBarWithEta {
        bar: Option<indicatif::ProgressBar>,
        start_time: Option<Instant>,
    }

    #[allow(dead_code)]
    impl ProgressBarWithEta {
        /// Create a new progress bar with ETA.
        ///
        /// The progress bar is only shown if:
        /// - `enabled` is true
        /// - stderr is a TTY
        /// - NO_COLOR env var is not set
        /// - TOKMD_NO_PROGRESS env var is not set
        pub fn new(enabled: bool, total: u64, message: &str) -> Self {
            let should_show = enabled && is_interactive();

            let (bar, start_time) = if should_show {
                let pb = indicatif::ProgressBar::new(total);
                pb.set_style(
                    ProgressStyle::with_template(
                        "{spinner:.cyan} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}",
                    )
                    .unwrap(),
                );
                pb.set_message(message.to_string());
                pb.enable_steady_tick(Duration::from_millis(100));
                (Some(pb), Some(Instant::now()))
            } else {
                (None, None)
            };

            Self { bar, start_time }
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
            if let Some(bar) = &self.bar {
                bar.finish_with_message(msg.to_string());
            }
        }

        /// Finish and clear the progress bar.
        pub fn finish_and_clear(&self) {
            if let Some(bar) = &self.bar {
                bar.finish_and_clear();
            }
        }
    }

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
    /// A no-op progress indicator when the `ui` feature is disabled.
    pub struct Progress;

    impl Progress {
        /// Create a new progress indicator (no-op without `ui` feature).
        pub fn new(_enabled: bool) -> Self {
            Self
        }

        /// Set the progress message (no-op without `ui` feature).
        pub fn set_message(&self, _msg: impl Into<String>) {}

        /// Finish and clear the spinner (no-op without `ui` feature).
        pub fn finish_and_clear(&self) {}
    }

    /// A no-op progress bar when `ui` feature is disabled.
    #[allow(dead_code)]
    pub struct ProgressBarWithEta;

    #[allow(dead_code)]
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
        pub fn set_message(&self, _msg: &str) {}

        /// Update the total length (no-op without `ui` feature).
        pub fn set_length(&self, _len: u64) {}

        /// Finish the progress bar (no-op without `ui` feature).
        pub fn finish_with_message(&self, _msg: &str) {}

        /// Finish and clear the progress bar (no-op without `ui` feature).
        pub fn finish_and_clear(&self) {}
    }
}

pub use ui_impl::Progress;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_disabled() {
        let progress = Progress::new(false);
        progress.set_message("test");
        progress.finish_and_clear();
    }

    #[test]
    fn test_progress_methods_no_panic() {
        let progress = Progress::new(false);
        progress.set_message("test");
        progress.finish_and_clear();
    }
}
