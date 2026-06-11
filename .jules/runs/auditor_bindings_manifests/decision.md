# Decision

## Option A
Tighten feature flags in `tokmd-node/Cargo.toml`.
- Fits the Auditor's mission to tighten feature flags.
- Drops an unnecessary explicit `rt-multi-thread` feature request from `tokio` in `tokmd-node` (as it only needs `tokio` for types/basic capabilities in production, reducing explicit feature constraint).
- Fits within the CI PR budget of 125 LEM.

## Option B
Attempt to drop `napi-derive` or `js-sys`.
- Less clear since they are directly used in bindings code.

## Decision
Option A. It's safe, tightens an explicit dependency feature request, and passes all existing tests.
