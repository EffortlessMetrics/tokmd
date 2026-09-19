# Decision

## Option A (Add `diff` and `cockpit` methods to `tokmd-wasm` binding)
**What it is:**
Add `run_diff` and `run_cockpit` functions to the `tokmd-wasm` export boundary. The `tokmd_core` already implements these via `run_json` for the `diff` and `cockpit` modes. `tokmd-node` and `tokmd-python` have been updated to export `diff` and `cockpit` functions natively to users of those bindings. `tokmd-wasm` currently implements `run_lang`, `run_module`, `run_export`, and `run_analyze`, but lacks explicit bindings for `run_diff` and `run_cockpit`. The core capabilities are there via `run_json` but the idiomatic JavaScript interfaces `runDiff` and `runCockpit` are missing. Note: `cockpit` requires the `cockpit` feature. Since `tokmd-wasm` delegates directly to `tokmd_core::ffi::run_json`, the implementation is simply a few thin wrappers calling `run_mode_js("diff", args)` and `run_mode_js("cockpit", args)` with appropriate feature flagging.

**Why it fits:**
The primary shard is `bindings-targets` and the assignment specifically notes: `Rust core <-> wasm/browser-runner drift`. The `diff` and `cockpit` features were clearly added to Python and Node interfaces but were omitted from WASM interfaces, leading to API drift across targets.

**Trade-offs:**
- Structure: Improves structural coherence across all binding interfaces.
- Velocity: Unblocks the use of `diff` and `cockpit` within browser/WASM-based runners in an idiomatic way (vs raw `runJson`).
- Governance: Low risk, maintains the established `run_mode_js` pattern.

## Option B (Remove `diff` and `cockpit` from other bindings or leave as-is)
**What it is:**
Remove `diff` and `cockpit` from `tokmd-python` and `tokmd-node`, assuming they are not supposed to be widely exposed, or leave the WASM API drifted.

**When to choose it instead:**
If `diff` and `cockpit` are strictly server-side features that don't make sense in WASM. However, `diff` is a pure data operation comparing receipts. `cockpit` might require git integration, but `tokmd_core::ffi::run_json` handles the capability gate (if `cockpit` feature is enabled). WASM often acts as a UI for rendering analysis, so computing diffs locally is highly relevant.

## Decision
Choose Option A. It addresses a clear cross-interface drift where Node and Python bindings expose `diff` and `cockpit`, but WASM bindings lack the equivalent `runDiff` and `runCockpit` methods.
