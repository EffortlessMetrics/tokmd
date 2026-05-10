//! Context packing helpers for LLM context window optimization.

mod budget;
mod manifest;
mod render;
mod select;

pub(crate) use budget::parse_budget;
pub(crate) use manifest::write_bundle_directory;
pub(crate) use render::{CountingWriter, format_list_output, write_bundle_output, write_head_tail};
pub(crate) use select::{SelectOptions, SelectResult, select_files_with_options};
