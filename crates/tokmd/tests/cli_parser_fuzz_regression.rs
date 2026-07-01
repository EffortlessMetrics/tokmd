use clap::Parser;
use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
use tokmd::cli::Cli;

#[test]
fn cli_parser_fuzz_regression_invalid_utf8() {
    #[cfg(unix)]
    {
        // An invalid UTF-8 byte sequence
        let invalid_utf8 = OsString::from_vec(vec![0x66, 0x6f, 0x80, 0x6f]);
        // The parser should safely reject the invalid UTF-8 without panicking
        // Since `lang` subcommand paths argument allows invalid UTF-8 (via PathBuf),
        // we test a flag that requires string validation instead (e.g., --exclude)
        let result = Cli::try_parse_from(vec![
            OsString::from("tokmd"),
            OsString::from("--exclude"),
            invalid_utf8,
        ]);
        assert!(
            result.is_err(),
            "Parser should safely reject invalid UTF-8 without panicking"
        );
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::InvalidUtf8);
    }
}
