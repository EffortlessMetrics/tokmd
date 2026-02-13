use anyhow::Result;
use tokmd_config as cli;

#[cfg(feature = "git")]
pub(crate) mod impl_git;

#[cfg(feature = "git")]
pub(crate) use impl_git::*;

pub(crate) fn handle(args: cli::CockpitArgs, global: &cli::GlobalArgs) -> Result<()> {
    #[cfg(feature = "git")]
    {
        impl_git::handle(args, global)
    }

    #[cfg(not(feature = "git"))]
    {
        let _ = (args, global);
        anyhow::bail!("The cockpit command requires the 'git' feature. Rebuild with --features git");
    }
}
