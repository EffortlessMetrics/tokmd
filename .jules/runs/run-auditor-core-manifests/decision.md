# Decision: Remove `js` feature from `uuid` dependency in `tokmd-format`

## Option A (recommended)
- **What it is:** Remove the `js` feature from the `uuid` dependency in `crates/tokmd-format/Cargo.toml`.
- **Why it fits this repo and shard:** The `tokmd-format` crate is part of the `core-pipeline` shard. As documented in the provided memory: "In Rust native applications or CLIs (like those in `tokmd`), avoid enabling the `js` feature on the `uuid` dependency, as it introduces unnecessary WebAssembly/JavaScript transitive dependencies (`wasm-bindgen`, `js-sys`) to the native compile surface." By removing this feature, we reduce the compile surface and adhere strictly to dependency hygiene, completely aligning with the Auditor persona's goals (target ranking #3: tighten feature flags to reduce compile surface).
- **Trade-offs:**
  - *Structure:* Cleaner dependency graph for native targets.
  - *Velocity:* Slightly faster builds due to removed transitive dependencies on target platforms (or skipped processing).
  - *Governance:* Reduces risk of unintended JS/WASM ecosystem dependencies leaking into a native core crate. No negative impacts since `tokmd-format` does not depend on WASM/JS environments.

## Option B
- **What it is:** Create a PR to remove unused `tempfile` dependencies.
- **When to choose it instead:** If there was a clear unused dependency across multiple crates that `cargo machete` or manual review confirmed was entirely unused.
- **Trade-offs:** `cargo machete` did not find unused direct dependencies. Finding unused dependencies manually could be noisy and `tempfile` was found to be used in tests, even though not explicitly in `src/` for `tokmd-model` (it is in `dev-dependencies`). The `js` feature removal is a more concrete, documented issue that fits the Auditor's target ranking #3.

## ✅ Decision
I choose **Option A**. The memory specifically calls out the `js` feature on the `uuid` dependency as an anti-pattern for this project's native compilation surface. It is a clean, boring, high-signal dependency hygiene improvement perfectly aligned with the Auditor persona.
