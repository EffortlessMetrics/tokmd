//! TTY detection utilities.

use std::io::IsTerminal;

/// Check if the CLI should run in interactive mode.
///
/// Returns true if:
/// - stdin is a TTY
/// - stdout is a TTY
/// - CI environment variable is not set
/// - TOKMD_NON_INTERACTIVE is not set
pub fn should_be_interactive() -> bool {
    // Check if stdin and stdout are TTYs
    if !std::io::stdin().is_terminal() {
        return false;
    }
    if !std::io::stdout().is_terminal() {
        return false;
    }

    // Check for CI environments
    if std::env::var("CI").is_ok() {
        return false;
    }

    // Check for explicit non-interactive flag
    if std::env::var("TOKMD_NON_INTERACTIVE").is_ok() {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_be_interactive() {
        // This test will vary based on environment
        // Just ensure it doesn't panic
        let _ = should_be_interactive();
    }
}
