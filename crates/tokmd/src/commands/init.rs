use anyhow::Result;
use tokmd_config as cli;
use tokmd_tokeignore as tokeignore;

pub(crate) fn handle(args: cli::InitArgs) -> Result<()> {
    tokeignore::init_tokeignore(&args)?;
    Ok(())
}
