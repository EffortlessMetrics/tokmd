//! Handler for the `tokmd cockpit` command.

use anyhow::Result;
#[cfg(not(feature = "git"))]
use anyhow::bail;
use tokmd_config as cli;

#[cfg(feature = "git")]
pub(crate) mod impl_git;

#[cfg(feature = "git")]
pub use impl_git::*;

pub(crate) fn handle(args: cli::CockpitArgs, global: &cli::GlobalArgs) -> Result<()> {
    #[cfg(feature = "git")]
    {
        return impl_git::handle(args, global);
    }

    #[cfg(not(feature = "git"))]
    {
        let _ = (args, global);
        bail!("The cockpit command requires the 'git' feature. Rebuild with --features git");
    }
}
