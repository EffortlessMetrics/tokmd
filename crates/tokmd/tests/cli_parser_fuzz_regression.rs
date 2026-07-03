//! Deterministic fuzz regression tests for the CLI parser.
//!
//! `cli_parser_properties.rs` proves the clap parser never panics on arbitrary
//! *UTF-8* argument strings. This file locks in the harder case that a fuzzer
//! reaches through `OsString`: raw argument bytes that are **not** valid UTF-8.
//!
//! A value-taking, `String`-typed flag (`--exclude`, a global `Vec<String>`
//! argument) must reject invalid UTF-8 with a clean clap `InvalidUtf8` error
//! rather than panicking. These cases are deterministic so they hold as
//! regressions without a live `cargo fuzz` run (which is not available in every
//! environment).

use std::ffi::OsString;

use clap::Parser;
use clap::error::ErrorKind;
use tokmd::cli::Cli;

/// An `OsString` that is deliberately not valid UTF-8.
///
/// On Unix the raw byte `0x80` is a lone continuation byte (invalid UTF-8). On
/// Windows argument strings are UTF-16, so `0xD800` (an unpaired high surrogate)
/// is used to build an `OsString` that has no valid Unicode scalar
/// representation and therefore no valid UTF-8 form.
#[cfg(any(unix, windows))]
fn invalid_utf8_osstring() -> OsString {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        std::ffi::OsStr::from_bytes(&[0x66, 0x6f, 0x80, 0x6f]).to_os_string()
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt;
        OsString::from_wide(&[0x0066, 0x006f, 0xD800, 0x006f])
    }
}

#[cfg(any(unix, windows))]
#[test]
fn cli_parser_rejects_invalid_utf8_exclude_value() {
    let bad = invalid_utf8_osstring();
    assert!(
        bad.to_str().is_none(),
        "fixture must be invalid UTF-8 for this regression to be meaningful"
    );

    let args: Vec<OsString> = vec![
        OsString::from("tokmd"),
        OsString::from("lang"),
        OsString::from("--exclude"),
        bad,
    ];

    // The parser must not panic and must surface a typed UTF-8 error.
    let err = Cli::try_parse_from(args).expect_err("invalid UTF-8 value must be rejected");
    assert_eq!(
        err.kind(),
        ErrorKind::InvalidUtf8,
        "expected a clap InvalidUtf8 error, got {:?}",
        err.kind()
    );
}

#[cfg(any(unix, windows))]
#[test]
fn cli_parser_rejects_invalid_utf8_global_exclude_before_subcommand() {
    let bad = invalid_utf8_osstring();

    let args: Vec<OsString> = vec![
        OsString::from("tokmd"),
        OsString::from("--exclude"),
        bad,
        OsString::from("module"),
    ];

    let err = Cli::try_parse_from(args).expect_err("invalid UTF-8 value must be rejected");
    assert_eq!(err.kind(), ErrorKind::InvalidUtf8);
}
