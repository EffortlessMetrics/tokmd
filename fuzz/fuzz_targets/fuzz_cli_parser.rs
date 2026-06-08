//! Fuzz target for CLI parser invariants.
//!
//! Feeds arbitrary argument arrays to `tokmd::cli::Cli` via `clap::Parser` to verify
//! that argument parsing never panics (no unhandled `unwrap()` or out-of-bounds in custom parsers).
//! Complements the existing `cli_parser_properties.rs` proptest with continuous fuzzing coverage.

#![no_main]

use clap::Parser;
use libfuzzer_sys::fuzz_target;
use std::ffi::OsString;
#[cfg(feature = "cli_parser")]
use tokmd::cli::Cli;

const MAX_ARGS: usize = 32;

fuzz_target!(|data: &str| {
    #[cfg(feature = "cli_parser")]
    {
        // Split input string by whitespace to get command line args
        let words: Vec<&str> = data.split_whitespace().collect();

        if words.len() > MAX_ARGS {
            return;
        }

        let mut args: Vec<OsString> = vec![OsString::from("tokmd")];
        for word in words {
            args.push(OsString::from(word));
        }

        // Try parsing arguments; if it fails gracefully, it's fine.
        // We only care about avoiding unhandled panics inside clap or custom value parsers.
        let _ = Cli::try_parse_from(args);
    }
});
