//! TTY detection utilities.

use std::io::IsTerminal;

/// Pure decision logic for interactive mode detection.
///
/// This is extracted to enable deterministic testing without real TTY checks.
fn should_be_interactive_inner(
    is_tty_in: bool,
    is_tty_out: bool,
    is_ci: bool,
    non_interactive_env: bool,
) -> bool {
    if !is_tty_in {
        return false;
    }
    if !is_tty_out {
        return false;
    }
    if is_ci {
        return false;
    }
    if non_interactive_env {
        return false;
    }
    true
}

/// Check if the CLI should run in interactive mode.
///
/// Returns true if:
/// - stdin is a TTY
/// - stdout is a TTY
/// - CI environment variable is not set
/// - TOKMD_NON_INTERACTIVE is not set
pub fn should_be_interactive() -> bool {
    let is_tty_in = std::io::stdin().is_terminal();
    let is_tty_out = std::io::stdout().is_terminal();
    let is_ci = std::env::var("CI").is_ok();
    let non_interactive_env = std::env::var("TOKMD_NON_INTERACTIVE").is_ok();

    should_be_interactive_inner(is_tty_in, is_tty_out, is_ci, non_interactive_env)
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

    #[test]
    fn test_inner_all_true_returns_true() {
        // Kills "replace with false" mutant
        assert!(should_be_interactive_inner(true, true, false, false));
    }

    #[test]
    fn test_inner_stdin_not_tty_returns_false() {
        // Kills "delete ! around tty_in" mutant
        assert!(!should_be_interactive_inner(false, true, false, false));
    }

    #[test]
    fn test_inner_stdout_not_tty_returns_false() {
        // Kills "delete ! around tty_out" mutant
        assert!(!should_be_interactive_inner(true, false, false, false));
    }

    #[test]
    fn test_inner_ci_set_returns_false() {
        // Kills "delete ! around CI check" or "is_ci -> !is_ci" mutant
        assert!(!should_be_interactive_inner(true, true, true, false));
    }

    #[test]
    fn test_inner_non_interactive_env_returns_false() {
        // Kills "delete ! around env check" mutant
        assert!(!should_be_interactive_inner(true, true, false, true));
    }

    #[test]
    fn test_inner_both_tty_false_returns_false() {
        // Additional coverage for combined conditions
        assert!(!should_be_interactive_inner(false, false, false, false));
    }

    #[test]
    fn test_inner_all_blocking_conditions_returns_false() {
        // Even with TTYs, blocking conditions should prevent interactive mode
        assert!(!should_be_interactive_inner(true, true, true, true));
    }
}
