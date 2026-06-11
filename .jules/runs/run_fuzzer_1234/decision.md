# Option A: Improve `fuzz_toml_config.rs` Corpus Generativity

The current TOML dictionary only covers a very basic set of keys and values. A lot of recent structure additions or valid strings for things like formatting, context strategy, diff tools are missing. Improving the fuzz dictionary (`fuzz/dict/toml.dict`) makes the existing fuzzer substantially more effective without writing new Rust code, directly improving input hardening for parser surfaces.

* **Structure**: High. Updates an existing truth artifact designed to guide the fuzzer.
* **Velocity**: High. Fast to implement, requires no code changes.
* **Governance**: Low risk.

# Option B: Add a `fuzz_cli_parser.rs` Target

Write a new libfuzzer target that takes random input strings, splits them into argv-like arrays, and passes them to `tokmd::cli::Cli::try_parse_from`. This directly hardens the CLI input parsing boundary.

* **Structure**: High. Adds a new fuzzer for the outermost parser boundary.
* **Velocity**: Medium. Requires adding a new binary target to `fuzz/Cargo.toml` and writing the fuzzer.
* **Governance**: Medium. We need to be careful not to trigger `exit()` on parse failure inside the fuzzer.

# Decision

**Option B** is chosen. Fuzzing the CLI parser boundary is a critical input hardening step. We already have a property test (`cli_parser_properties.rs`) doing something similar with `proptest`, but a proper libfuzzer target can explore the input space more thoroughly and persistently. I will implement a new target `fuzz_cli_parser.rs` and update `fuzz/Cargo.toml` appropriately, satisfying the `fuzzer` persona's mission to improve fuzzability around parser/input surfaces.
