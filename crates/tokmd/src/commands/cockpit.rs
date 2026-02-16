//! Wrapper for the `tokmd cockpit` command.
//!
//! This module conditionally includes the implementation when the `git` feature is enabled.
//! When disabled, it provides a stub handler that returns an error.

use anyhow::Result;
use tokmd_config as cli;

#[cfg(feature = "git")]
pub(crate) mod impl_git;

/// Handle the cockpit command.
pub(crate) fn handle(args: cli::CockpitArgs, global: &cli::GlobalArgs) -> Result<()> {
    #[cfg(feature = "git")]
    {
        impl_git::handle(args, global)
    }

    #[cfg(not(feature = "git"))]
    {
        let _ = args;
        let _ = global;
        anyhow::bail!(
            "The cockpit command requires the 'git' feature. Rebuild with --features git"
        );
    }
}
