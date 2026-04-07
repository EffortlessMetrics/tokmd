## 💡 Summary
Replaced pure fuzzer runs that were previously identifying deterministic parser invariants with standard `proptest` suites, enabling CI and standard `cargo test` execution of these proofs without requiring nightly toolchains or `cargo-fuzz`. Additionally fixed a broken rustdoc link in `tokmd-gate` that triggered intra-doc link warnings.

## 🎯 Why
The Fuzzer persona is tasked with improving input hardening and fuzz coverage. However, the existing `cargo-fuzz` targets rely heavily on deterministic parser invariants rather than purely stateful bugs. Running these routinely requires nightly and specific feature flags per crate. Replacing deterministic invariant checks from `cargo-fuzz` with `proptest` locks in the coverage natively via standard `cargo test`, making these tests accessible and fast for all contributors. Also, `cargo doc` produced warnings due to unescaped bracket notation in `tokmd-gate` documentation which were interpreted as broken intra-doc links.

## 🔎 Evidence
- `crates/tokmd-gate/src/pointer.rs`
- `cargo doc --no-deps -p tokmd-gate` (warning resolved)
- `crates/tokmd-scan-args/tests/proptest_scan_args.rs`
- `crates/tokmd-config/tests/proptest_config.rs`
- `crates/tokmd-gate/tests/proptest_gate.rs`
- Proptest execution: `cargo test -p tokmd-config -p tokmd-gate -p tokmd-scan-args` passing reliably.

## 🧭 Options considered
### Option A (recommended)
- what it is: Extract deterministic fuzzer inputs into `proptest` suites on primary shards (config, scan-args, gate) and fix the broken doclink.
- why it fits this repo and shard: Achieves proof-improvement for parser/input hardening seamlessly within standard `cargo test`, ensuring tests run on every commit across all platforms, not just during dedicated fuzzing sessions. Fixes immediate technical debt (doclink warning) safely.
- trade-offs: High velocity and robust governance alignment via `proptest` usage over raw `cargo fuzz`.

### Option B
- what it is: Rewrite the `cargo-fuzz` targets directly and attempt to execute them via a nightly container, leaving `proptest` implementation for another persona.
- when to choose it instead: If the bugs being sought were highly stateful crashes requiring sanitizers rather than deterministic parser invariants.
- trade-offs: Extremely slow and brittle CI integration, failing the "proof-improvement" mission by hiding proofs behind specialized tooling.

## ✅ Decision
Option A. Migrating deterministic fuzz checks to `proptest` is vastly superior for repository velocity and CI stability, actively hardening the input parser layer in an accessible manner. The doclink fix is a trivial but necessary inclusion for overall workspace health.

## 🧱 Changes made (SRP)
- `crates/tokmd-gate/src/pointer.rs`: Escaped brackets `\[` and `\]` in documentation string.
- `crates/tokmd-scan-args/tests/proptest_scan_args.rs`: Added proptests for path normalization determinism.
- `crates/tokmd-config/tests/proptest_config.rs`: Added proptests ensuring `TomlConfig::parse` never panics.
- `crates/tokmd-gate/tests/proptest_gate.rs`: Added proptests ensuring pointer resolution and policy evaluation never panic.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd-config -p tokmd-gate -p tokmd-scan-args
test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
...
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
...
test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 🧭 Telemetry
- Change shape: Test Addition, Doc Fix
- Blast radius: Internal tests and docs. No API or structural changes.
- Risk class + why: Low. Additive tests and doc corrections only.
- Rollback: Revert branch.
- Gates run: `cargo clippy`, `cargo test`, `cargo fmt`, `cargo doc`

## 🗂️ .jules artifacts
- `.jules/runs/fuzzer_input_hardening/envelope.json`
- `.jules/runs/fuzzer_input_hardening/decision.md`
- `.jules/runs/fuzzer_input_hardening/receipts.jsonl`
- `.jules/runs/fuzzer_input_hardening/result.json`
- `.jules/runs/fuzzer_input_hardening/pr_body.md`

## 🔜 Follow-ups
None immediately.
