#[cfg(not(feature = "git"))]
use anyhow::Result;
#[cfg(not(feature = "git"))]
use tokmd_config as cli;

#[cfg(feature = "git")]
mod impl_git;

#[cfg(feature = "git")]
pub(crate) use impl_git::*;

#[cfg(not(feature = "git"))]
pub(crate) fn handle(args: cli::CockpitArgs, _global: &cli::GlobalArgs) -> Result<()> {
    let _ = args;
    anyhow::bail!("The cockpit command requires the 'git' feature. Rebuild with --features git");
}
