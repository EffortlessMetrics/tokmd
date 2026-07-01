## 💡 Summary
Added a deterministic fuzzer regression test for the CLI parser to prove safe handling of invalid UTF-8 inputs. This expands the input hardening guarantees without needing `cargo fuzz`.

## 🎯 Why
The Fuzzer persona is tasked with improving input hardening and fuzzability of interface surfaces. Since `cargo fuzz` is unavailable in the environment, we must lock in deterministic behavior for fuzzed/malformed inputs. We want to guarantee that feeding arbitrary malformed byte sequences (like invalid UTF-8 in OsString) to string-expecting CLI arguments (e.g., `--exclude`) safely returns a parser error rather than causing a panic.

## 🔎 Evidence
- `crates/tokmd/tests/cli_parser_properties.rs` (Existing property tests proving parser safety)
- The newly created `cli_parser_fuzz_regression` test confirms that providing `[0x66, 0x6f, 0x80, 0x6f]` to `--exclude` correctly yields a clap `InvalidUtf8` error without crashing.
- `cargo test --test cli_parser_fuzz_regression` succeeds.

## 🧭 Options considered
### Option A (recommended)
- Land the deterministic regression test for the CLI parser demonstrating safe rejection of invalid UTF-8.
- Fits the repo and shard by reinforcing `crates/tokmd/tests` interface guarantees.
- trade-offs: Structure (clean), Velocity (fast), Governance (satisfies fuzz fallback expectations).

### Option B
- Attempt to implement manual fuzzer stubs (e.g., libfuzzer-sys).
- when to choose it instead: If fuzzing infrastructure was natively supported and reliable.
- trade-offs: High friction, likely build errors.

## ✅ Decision
Option A. It's safe, proven, and explicitly meets the Fuzzer requirements to provide deterministic input hardening proof when fuzzers are unavailable.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/cli_parser_fuzz_regression.rs`

## 🧪 Verification receipts
```text
$ CI=true cargo test --test cli_parser_fuzz_regression --verbose
running 1 test
test cli_parser_fuzz_regression_invalid_utf8 ... ok
```

## 🧭 Telemetry
- Change shape: Test addition
- Blast radius: None (tests only)
- Risk class: Low - pure proof improvement.
- Rollback: Revert test file.
- Gates run: `CI=true cargo test --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/run-fuzzer-001/envelope.json`
- `.jules/runs/run-fuzzer-001/decision.md`
- `.jules/runs/run-fuzzer-001/receipts.jsonl`
- `.jules/runs/run-fuzzer-001/result.json`
- `.jules/runs/run-fuzzer-001/pr_body.md`

## 🔜 Follow-ups
None.
