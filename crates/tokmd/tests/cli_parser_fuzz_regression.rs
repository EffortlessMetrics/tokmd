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

#[cfg(any(unix, windows))]
#[test]
fn cli_parser_rejects_invalid_utf8_numeric_flags() {
    let bad = invalid_utf8_osstring();

    // Verify numerical flags utilizing custom value_parser gracefully reject
    // invalid UTF-8 byte inputs with an InvalidUtf8 error rather than panicking.

    // --max-commits on badge
    let args1: Vec<OsString> = vec![
        OsString::from("tokmd"),
        OsString::from("badge"),
        OsString::from("--max-commits"),
        bad.clone(),
    ];
    let err1 = Cli::try_parse_from(args1).expect_err("invalid UTF-8 value must be rejected");
    assert_eq!(err1.kind(), ErrorKind::InvalidUtf8);

    // --max-commit-files on context
    let args2: Vec<OsString> = vec![
        OsString::from("tokmd"),
        OsString::from("context"),
        OsString::from("--max-commit-files"),
        bad.clone(),
    ];
    let err2 = Cli::try_parse_from(args2).expect_err("invalid UTF-8 value must be rejected");
    assert_eq!(err2.kind(), ErrorKind::InvalidUtf8);

    // --max-file-tokens on context
    let args3: Vec<OsString> = vec![
        OsString::from("tokmd"),
        OsString::from("context"),
        OsString::from("--max-file-tokens"),
        bad.clone(),
    ];
    let err3 = Cli::try_parse_from(args3).expect_err("invalid UTF-8 value must be rejected");
    assert_eq!(err3.kind(), ErrorKind::InvalidUtf8);
}
