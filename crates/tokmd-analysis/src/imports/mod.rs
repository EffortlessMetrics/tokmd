//! Language-aware import extraction and deterministic target normalization.
//!
//! This module intentionally keeps only parsing and normalization logic for
//! import-like statements so analysis code can compose it without filesystem
//! or receipt dependencies.

#![forbid(unsafe_code)]

mod parser;

pub(crate) use parser::{normalize_import_target, parse_imports, supports_language};

#[cfg(test)]
#[path = "tests.rs"]
mod moved_tests;
