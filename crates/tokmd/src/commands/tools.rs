//! Handler for the `tokmd tools` command.

use crate::cli;
use crate::tool_schema::{build_tool_schema, render_output};
use anyhow::Result;
use clap::CommandFactory;

/// Handle the tools command.
pub(crate) fn handle(args: cli::ToolsArgs) -> Result<()> {
    let cmd = cli::Cli::command();
    let schema = build_tool_schema(&cmd);
    let output = render_output(&schema, args.format, args.pretty)?;
    println!("{}", output);
    Ok(())
}
