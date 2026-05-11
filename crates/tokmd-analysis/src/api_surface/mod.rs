//! API surface report construction.
//!
//! The root module keeps the public crate-internal entrypoint stable while
//! owner modules handle report aggregation and language-specific symbol scans.

#[cfg(test)]
#[path = "tests.rs"]
mod moved_tests;

mod report;
mod symbols;

pub(crate) use report::build_api_surface_report;
