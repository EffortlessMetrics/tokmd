//! Handler for the `tokmd cockpit` command.
//!
//! Generates PR cockpit metrics for code review automation.

use anyhow::Result;
#[cfg(not(feature = "git"))]
use anyhow::bail;
use tokmd_config as cli;

#[cfg(feature = "git")]
mod impl_git;

#[cfg(feature = "git")]
pub(crate) use impl_git::*;

/// Handle the cockpit command.
pub(crate) fn handle(args: cli::CockpitArgs, global: &cli::GlobalArgs) -> Result<()> {
    #[cfg(not(feature = "git"))]
    {
        let _ = (args, global); // Silence unused warning
        bail!("The cockpit command requires the 'git' feature. Rebuild with --features git");
    }

    #[cfg(feature = "git")]
    {
        impl_git::handle_inner(args, global)
    }
}
