# Option A: Remove `js` feature from `uuid` in `tokmd-format`

**What it is:**
The `uuid` crate is a direct dependency of `tokmd-format` with features `["v4", "js"]`. The `js` feature is intended for WebAssembly/JS targets when using wasm-bindgen to get random number generation for v4 UUIDs. Since `tokmd-format` is a CLI/library for formatting code analysis receipts natively and does not seem to compile to JS/WASM as a primary target (or if it does, `js` feature might be too broad to unconditionally enable), removing `js` tightens the feature flags and removes unnecessary JavaScript-related dependencies (`js-sys`, `wasm-bindgen`) from the native compile surface.

**Why it fits this repo and shard:**
- Aligns with the target ranking: "3) tighten feature flags to reduce compile surface".
- Target is in `crates/tokmd-format/Cargo.toml`, which is in the primary paths allowed.

**Trade-offs:**
- Structure: Minor manifest change.
- Velocity: Faster compile times on non-wasm targets by dropping potential wasm-bindgen dependencies (though cargo often ignores target-specific dependencies, `uuid` might still parse/resolve them, or it just reduces visual noise and intent confusion). Actually, the `js` feature on `uuid` unconditionally pulls in `wasm-bindgen` when compiling for `wasm32-*`. If `tokmd` doesn't target wasm, this feature is dead.
- Governance: Improves dependency hygiene.

# Option B: Remove `tempfile` direct dependency from `tokmd-scan` and move to `[dev-dependencies]`

**What it is:**
`tempfile` is a direct dependency in `tokmd-scan`. A `grep` shows `tempfile` is only used for `tempfile::TempDir`, `tempfile::tempdir()`. Oh wait, if it's used in `src/lib.rs` (as seen in `crates/tokmd-scan/src/lib.rs`), then it's actually required at runtime, not just in tests. Let's check `crates/tokmd-scan/src/lib.rs` to see if it's test-only code.
If it is used in the library's actual functionality (e.g. creating temp dirs for scanning mock files, or handling git clones), then it is legitimately used.
Let's stick with tightening feature flags.

Let's double check if `tempfile` is used outside of tests in `tokmd-scan`.

Let's do Option A.
