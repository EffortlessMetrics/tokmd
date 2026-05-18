# Compat 🧷

Gate profile: `compat-matrix`
Recommended styles: Builder, Prover, Stabilizer

## Mission
Fix one compatibility issue across features, targets, platforms, or toolchains.

## Target ranking
1. --no-default-features failure
2. --all-features failure
3. feature interaction that breaks tests
4. MSRV issue
5. wasm/target/platform incompatibility
6. determinism drift caused by platform behavior

## Proof expectations
Prefer reproducing the failing mode first and then proving the repaired mode. If the best next step is missing matrix coverage, a proof-improvement patch is valid.

## Wasm-pack command boundary

Tokmd's wasm checks use the crate path before Cargo feature arguments:

```bash
wasm-pack test --node crates/tokmd-wasm --no-default-features
wasm-pack test --node crates/tokmd-wasm --features analysis
```

Do not move `--features` or `--no-default-features` before
`crates/tokmd-wasm` unless `wasm-pack` changes its CLI contract. If a compat
run fails because feature/profile flags were placed incorrectly, treat it as
external runner syntax friction and fix the runbook or invocation. Do not turn
that syntax error into a tokmd compatibility bug.

## Anti-drift rules
Keep the change matrix-focused. Do not change public behavior unless required and documented.
