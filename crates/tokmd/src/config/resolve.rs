//! CLI argument resolution from legacy JSON profiles and TOML config views.

mod export;
mod lang;
mod module;
mod parse;

pub use export::{resolve_export, resolve_export_with_config};
pub use lang::{resolve_lang, resolve_lang_with_config};
pub use module::{resolve_module, resolve_module_with_config};
