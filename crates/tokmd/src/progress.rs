//! Progress spinner utilities for long-running operations.

use indicatif::{ProgressBar, ProgressStyle};
use std::io::IsTerminal;
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
    /// - stdout is a TTY
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

/// Check if we should show interactive output.
fn is_interactive() -> bool {
    // Check if stdout is a TTY
    if !std::io::stdout().is_terminal() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_disabled() {
        let progress = Progress::new(false);
        assert!(progress.bar.is_none());
    }

    #[test]
    fn test_progress_methods_no_panic() {
        let progress = Progress::new(false);
        progress.set_message("test");
        progress.finish_and_clear();
    }
}
