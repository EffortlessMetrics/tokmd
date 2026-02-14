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
                        .expect("hardcoded progress template should be valid")
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
