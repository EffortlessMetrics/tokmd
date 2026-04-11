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

## Anti-drift rules
Keep the change matrix-focused. Do not change public behavior unless required and documented.
