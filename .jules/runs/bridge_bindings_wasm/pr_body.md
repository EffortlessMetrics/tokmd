## 💡 Summary
Updated the Node and Python bindings for the `export` function to properly expose the `meta` and `strip_prefix` parameters. These settings are natively supported by the underlying Rust core (`ExportSettings`) but were missing from the binding signatures and documentation.

## 🎯 Why
There was cross-surface drift between the Rust core and the Python/Node bindings. Specifically, the core `tokmd` parser supports `meta` and `strip_prefix` settings for export operations (`crates/tokmd-settings/src/commands.rs` -> `crates/tokmd-core/src/ffi/settings_parse.rs`), but the Python and Node FFI wrappers lacked docstring documentation and parameter passing for these arguments. This created friction for users trying to disable the metadata record or modify export output prefixes from bindings.

## 🔎 Evidence
Minimal proof:
- `crates/tokmd-core/src/ffi/settings_parse.rs` parses `meta` and `strip_prefix` keys.
- `crates/tokmd-node/src/lib.rs` and `crates/tokmd-python/src/lib.rs` omitted these options in their TS/Python docstrings and parameters.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update bindings documentation and function signatures to include the missing fields.
- why it fits this repo and shard: It directly addresses the target ranking #2 "Rust core ↔ Python/Node drift".
- trade-offs:
  - Structure: Increases parameter count in Python, but aligns with existing pattern.
  - Velocity: Low effort, high reward to fix drift.
  - Governance: Maintains feature parity across platforms.

### Option B
- what it is: Leave bindings out of sync.
- when to choose it instead: If the parameters were intentionally hidden or unsafe for bindings, which they aren't.
- trade-offs: Frustrates users expecting full API capabilities in bindings.

## ✅ Decision
Option A. It accurately fixes the Python/Node drift with Rust core without breaking existing calls, fulfilling the persona mission of reducing cross-interface drift.

## 🧱 Changes made (SRP)
- `crates/tokmd-node/src/lib.rs`: Updated TS docstring to include `meta` and `strip_prefix` in `ExportOptions`.
- `crates/tokmd-python/src/lib.rs`: Added `meta` and `strip_prefix` to the Python docstring, `pyo3` signature, and parameter list for `export()`.

## 🧪 Verification receipts
```text
{"timestamp": "2024-05-12T14:36:00Z", "command": "patch -p0 < update.patch", "output": "patching file crates/tokmd-node/src/lib.rs\nHunk #1 succeeded at 177 (offset -6 lines).\npatching file crates/tokmd-python/src/lib.rs\nHunk #1 succeeded at 188 (offset -44 lines).\nHunk #2 succeeded at 201 (offset -43 lines).\nHunk #3 succeeded at 216 (offset -43 lines).\nHunk #4 succeeded at 235 (offset -43 lines)."}
{"timestamp": "2024-05-12T14:36:10Z", "command": "cargo check -p tokmd-node -p tokmd-python", "output": "    Finished `dev` profile [unoptimized + debuginfo] target(s) in 21.65s"}
```

## 🧭 Telemetry
- Change shape: Cross-surface drift repair
- Blast radius: API (Bindings only)
- Risk class: Low - strictly expands optional arguments and docs.
- Rollback: Revert the signature/doc patches.
- Gates run: `cargo check -p tokmd-node -p tokmd-python`, `npm --prefix web/runner test`

## 🗂️ .jules artifacts
- `.jules/runs/bridge_bindings_wasm/envelope.json`
- `.jules/runs/bridge_bindings_wasm/decision.md`
- `.jules/runs/bridge_bindings_wasm/receipts.jsonl`
- `.jules/runs/bridge_bindings_wasm/result.json`
- `.jules/runs/bridge_bindings_wasm/pr_body.md`

## 🔜 Follow-ups
None.
