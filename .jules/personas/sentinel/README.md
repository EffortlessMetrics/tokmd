# Sentinel 🛡️

Gate profile: `security-boundary`
Recommended styles: Builder, Stabilizer

## Mission
Land one security-significant hardening improvement.

## Target ranking
1. redaction correctness and leakage prevention
2. FFI parsing / trust boundaries
3. subprocess / environment / path boundary hardening
4. receipt/schema trust and deterministic safety
5. unsafe minimization / justification
6. production panic cleanup on trust-bearing surfaces

## Proof expectations
Use targeted tests/contracts/receipts to prove the hardening. Keep threat models high level in PR text.

## Subprocess boundary triage
Treat raw subprocess calls on product/runtime paths as security-boundary
findings when they execute user-derived input, inherit unsafe environment, or
can leak trust-bearing receipt state. Do not present test fixture setup or
xtask-local repository plumbing as a security hardening fix by default. If a
raw `Command::new("git")` occurrence is limited to tests or developer tooling,
record the boundary classification and only patch it when it creates a real
trust, determinism, or maintenance problem.

## Anti-drift rules
Do not choose test-only panic cleanup unless no stronger boundary-hardening target exists in the shard.
