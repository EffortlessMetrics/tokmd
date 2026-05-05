## 💡 Summary
Replaced unrunnable fuzz targets with deterministic `proptest` suites for core interfaces and parser invariants.

## 🎯 Why
The `cargo fuzz` toolchain requires nightly compiler features (`-Zsanitizer=address`) that fail to build in standard sandbox environments. Since the `fuzz` tests inside `fuzz/fuzz_targets` contained highly valuable property invariants for path normalization, redaction, JSON extraction, and import parsing, converting them to `proptest` suites allows us to lock in this determinism without relying on broken fuzz environments.

## 🔎 Evidence
Minimal proof:
- Attempting to run the existing fuzz tests (e.g. `cargo fuzz run fuzz_toml_config`) fails locally with: `error: the option 'Z' is only accepted on the nightly compiler`.
- Migrated 6 major target domains to `proptest`:
  - `cargo test -p tokmd --test fuzz_toml_config_proptests`
  - `cargo test -p tokmd --test fuzz_json_types_proptests`
  - `cargo test -p tokmd --test fuzz_scan_args_proptests`
  - `cargo test -p tokmd --test fuzz_context_policy_proptests`
  - `cargo test -p tokmd --test fuzz_import_parser_proptests`
  - `cargo test -p tokmd --test fuzz_run_json_proptests`

## 🧭 Options considered
### Option A (recommended)
- what it is: Extracting the core `fuzz_target` closures and moving them into isolated deterministic `proptest!` suites inside `crates/tokmd/tests/`.
- why it fits this repo and shard: Fulfills the Fuzzer mission of input hardening around parsers and configs while sidestepping broken dependencies.
- trade-offs: Structure / Velocity / Governance: Loses the automated mutation exploration of a true fuzzer, but guarantees that the invariants are run reliably on every standard test suite invocation.

### Option B
- what it is: Attempt to upgrade or force nightly installation.
- when to choose it instead: If true fuzzing is an absolute requirement that justifies environmental mutability risk.
- trade-offs: Introduces high environment-mutating friction and might break standard CI pipelines.

## ✅ Decision
Option A was chosen to maximize deterministic proof surfaces with the current environmental constraints.

## 🧱 Changes made (SRP)
- Created `crates/tokmd/tests/fuzz_toml_config_proptests.rs`
- Created `crates/tokmd/tests/fuzz_json_types_proptests.rs`
- Created `crates/tokmd/tests/fuzz_scan_args_proptests.rs`
- Created `crates/tokmd/tests/fuzz_context_policy_proptests.rs`
- Created `crates/tokmd/tests/fuzz_import_parser_proptests.rs`
- Created `crates/tokmd/tests/fuzz_run_json_proptests.rs`

## 🧪 Verification receipts
```text
$ cargo test -p tokmd --test fuzz_toml_config_proptests
ok. 3 passed

$ cargo test -p tokmd --test fuzz_json_types_proptests
ok. 8 passed

$ cargo test -p tokmd --test fuzz_scan_args_proptests
ok. 1 passed

$ cargo test -p tokmd --test fuzz_context_policy_proptests
ok. 1 passed

$ cargo test -p tokmd --test fuzz_import_parser_proptests
ok. 28 passed

$ cargo test -p tokmd --test fuzz_run_json_proptests
ok. 1 passed
```

## 🧭 Telemetry
- Change shape: Test Addition
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): No production changes.
- Risk class + why: Very low. It purely expands deterministic testing.
- Rollback: Revert the new test files.
- Gates run: `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/run-fuzzer-input-hardening-1/envelope.json`
- `.jules/runs/run-fuzzer-input-hardening-1/decision.md`
- `.jules/runs/run-fuzzer-input-hardening-1/receipts.jsonl`
- `.jules/runs/run-fuzzer-input-hardening-1/result.json`
- `.jules/runs/run-fuzzer-input-hardening-1/pr_body.md`

## 🔜 Follow-ups
None.
