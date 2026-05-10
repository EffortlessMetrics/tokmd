## 💡 Summary
Expanded the property tests in `crates/tokmd/tests/cli_parser_properties.rs` to include all currently defined CLI subcommands.

## 🎯 Why
The previous property test for the CLI parser (`cli_parser_never_panics_on_subcommand_with_arbitrary_args`) only checked a subset of subcommands (like `lang`, `module`, and `export`). This left commands such as `run`, `gate`, `tools`, `context`, `completions`, and others unchecked against edge-case regression and panics when fed arbitrary string arguments. Locking in deterministic, non-panicking parsing for the full CLI surface reduces uncertainty.

## 🔎 Evidence
- `crates/tokmd/tests/cli_parser_properties.rs` had a `prop_oneof!` list missing several commands that exist in `crates/tokmd/src/cli/parser.rs`.
- Running `cargo test -p tokmd --test cli_parser_properties` with the added commands succeeds, proving the invariant holds for all tested inputs on the full interface surface.

## 🧭 Options considered
### Option A (recommended)
- Add all missing subcommands to the `prop_oneof!` generator in the existing property test.
- Fits this repo and shard because it directly bolsters edge-case polish and regression coverage for CLI interfaces.
- Trade-offs: Structure is preserved; velocity is slightly reduced by more property cases; governance is improved by hardening the CLI against panics.

### Option B
- Wait for bug reports or write targeted unit tests for individual parser failures.
- When to choose: When property testing is unavailable or too slow.
- Trade-offs: Misses the opportunity to lock in parser invariance broadly across the entire CLI surface.

## ✅ Decision
Chosen Option A. It maximizes testing coverage over CLI interfaces and provides high-signal invariant guarantees for all commands, aligning with the Specsmith persona.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/cli_parser_properties.rs`: Added missing subcommands (`run`, `check-ignore`, `tools`, `gate`, `baseline`, `badge`, `init`, `completions`, `sensor`) to the `cli_parser_never_panics_on_subcommand_with_arbitrary_args` property test.

## 🧪 Verification receipts
```text
cargo test -p tokmd --test cli_parser_properties
running 2 tests
test cli_parser_never_panics_on_arbitrary_args ... ok
test cli_parser_never_panics_on_subcommand_with_arbitrary_args ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.21s
```

## 🧭 Telemetry
- Change shape: Test/Proof improvement
- Blast radius: Internal (tests only)
- Risk class: Low (no production code changed)
- Rollback: Revert the test file
- Gates run: `cargo test -p tokmd --test cli_parser_properties`

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_interfaces/envelope.json`
- `.jules/runs/specsmith_interfaces/decision.md`
- `.jules/runs/specsmith_interfaces/receipts.jsonl`
- `.jules/runs/specsmith_interfaces/result.json`
- `.jules/runs/specsmith_interfaces/pr_body.md`

## 🔜 Follow-ups
None.
