use clap::Parser;
use proptest::prelude::*;
use tokmd::cli::Cli;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]
    #[test]
    fn cli_never_panics_on_arbitrary_args(args in prop::collection::vec(".*", 0..20)) {
        // Try parsing arbitrary string arguments.
        // It's expected to fail parsing on garbage input, but it must never panic.
        let _ = Cli::try_parse_from(std::iter::once("tokmd".to_string()).chain(args));
    }
}
