//! Tools command parser types.
//!
//! This module owns the clap contract for `tokmd tools` while the parent parser
//! module keeps the top-level command dispatch shape.

use crate::tool_schema::ToolSchemaFormat;
use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct ToolsArgs {
    /// Output format for the tool schema.
    #[arg(long, value_enum, default_value_t = ToolSchemaFormat::Jsonschema)]
    pub format: ToolSchemaFormat,

    /// Pretty-print JSON output.
    #[arg(long)]
    pub pretty: bool,
}
