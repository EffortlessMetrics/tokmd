---
id: fuzzer-scan-args-collision
persona: fuzzer
style: prover
shard: interfaces
status: open
---
# Fuzzer scan_args Hardening Collision

Attempted to port `fuzz_scan_args` into deterministic property tests inside `crates/tokmd/tests/properties.rs` because the fuzz target failed locally to compile (`libfuzzer-sys` link error).

However, `scan_args` invariants (determinism, redaction, normalization, no_ignore fan-out) were already covered by recent upstream changes in `crates/tokmd-format/src/scan_args/mod.rs` (under the `format_redaction_scan_args` scope).

Adding tests to `crates/tokmd/tests/properties.rs` and tracking it in `ci/proof.toml` under `schema_contracts` was incorrect ownership boundary routing. Future improvements to scan-args must extend existing tests in `tokmd-format` directly.

The work was gracefully aborted as it was superseded.
