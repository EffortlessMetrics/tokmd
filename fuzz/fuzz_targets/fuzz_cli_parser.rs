#![no_main]
use clap::Parser;
use libfuzzer_sys::fuzz_target;
use tokmd::cli::Cli;

fuzz_target!(|data: &[u8]| {
    if data.len() > 1024 {
        return;
    }

    if let Ok(s) = std::str::from_utf8(data) {
        // Split by null byte to simulate argv
        let mut args = vec!["tokmd"];
        args.extend(s.split('\0'));

        let _ = Cli::try_parse_from(args);
    }
});
