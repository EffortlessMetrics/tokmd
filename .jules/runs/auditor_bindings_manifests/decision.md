# Decision

## Option A
Remove duplicate/redundant dependency declarations in `tokmd-wasm` and tighten feature flags in `tokmd-node`.
- Fits the Auditor's mission to remove redundant items and tighten feature flags.
- Removes an explicit path dependency in favor of the cleaner workspace inheritance in `tokmd-wasm`.
- Drops an unnecessary explicit `rt-multi-thread` feature request from `tokio` in `tokmd-node`.

## Option B
Attempt to drop `napi-derive` or `js-sys`.
- Less clear since they are directly used in bindings code.

## Decision
Option A. It's safe, addresses redundant definitions, tightens an explicit dependency request, and passes all existing tests.
