# Friction Item: Config Edge Cases

## Observation
I explored edge-case testing in `interfaces`, specifically the CLI parser edge cases (`cli_error_paths_w51.rs`, `cli_errors_w66.rs`, `edge_cases_cli_w50.rs`) and configuration resolution logic (`config_resolution.rs`, `src/config.rs`).

The current coverage for CLI edge conditions and config parsing is generally solid (e.g., parsing invariants are locked in with proptest in `cli_parser_properties.rs`, `config_resolution.rs` checks TOML vs JSON profile overrides thoroughly, and `edge_cases_cli_w50.rs` checks basic flag combination constraints).

## Gap
However, there are still subtle un-verified edge cases regarding `profile` selection via `TOKMD_PROFILE` vs CLI argument, particularly concerning malformed characters (e.g. `\0` or control characters) that could cause implicit logic issues further down the pipeline in `resolve_config`, although they are sanitized.

This seems like an area where BDD test scenarios might improve coverage but not clear enough that a patch here is necessarily a "coherent reviewer story" that doesn't just feel like generic test cleanup.
