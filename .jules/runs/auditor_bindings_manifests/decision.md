# Decision

## Option A (recommended)
**Action**: Remove the `napi-build` build dependency and the associated `build.rs` from `crates/tokmd-node`.
**Why**: According to cargo machete and our testing, `napi-build` is an unused build dependency since the native build process can be managed via the `napi` CLI (which we use through `@napi-rs/cli` and `napi build --platform` in our npm scripts). Running `npm run build:debug` and `npm run test` completes successfully without the `build.rs` script and `napi-build` crate. This falls in line with the target ranking to remove an unused direct dependency.

## Option B
**Action**: Try to find duplicate or redundant dependency declarations/features.
**Why**: If we aren't fully sure about `napi-build` safely being removed. However, testing confirmed the node tests still build and run successfully without it, making Option A the stronger signal and safer choice.
