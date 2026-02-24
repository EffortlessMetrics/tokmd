//! Handler for the `tokmd tools` command.

use anyhow::Result;
use clap::CommandFactory;
use tokmd_config as cli;
use tokmd_tool_schema::{build_tool_schema, render_output};

/// Handle the tools command.
pub(crate) fn handle(args: cli::ToolsArgs) -> Result<()> {
    let cmd = cli::Cli::command();
    let schema = build_tool_schema(&cmd);
    let output = render_output(&schema, args.format, args.pretty)?;
    println!("{}", output);
    Ok(())
}
