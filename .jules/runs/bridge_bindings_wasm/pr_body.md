## 💡 Summary
Bridged a cross-surface drift by exposing `run_json_bytes` in the `tokmd-node` and `tokmd-python` bindings. This aligns the target surfaces with `tokmd-core` and `tokmd-wasm`.

## 🎯 Why
The `tokmd-core` provided `run_json_bytes` to support fail-closed archive processing (ZIPs) without host-path IO, and `tokmd-wasm` bridged this via `runJsonBytes`. However, `tokmd-node` and `tokmd-python` lacked these bindings, creating an interface drift where Node and Python consumers could not utilize the `archive-zip` fast path.

## 🔎 Evidence
- `crates/tokmd-core/src/ffi/byte_mode.rs` exports `run_json_bytes`
- `crates/tokmd-wasm/src/lib.rs` exports `runJsonBytes`
- Prior to this patch, grepping for `run_json_bytes` in `crates/tokmd-node` and `crates/tokmd-python` yielded no results, while `tokmd-core` and `tokmd-wasm` fully supported it.

## 🧭 Options considered
### Option A (recommended)
- what it is: Implement `run_json_bytes` in the Node.js and Python bindings.
- why it fits this repo and shard: Directly targets `bindings-targets` cross-interface drift as prescribed.
- trade-offs:
    - Structure: High alignment. Closes the gap between targets.
    - Velocity: Fast to implement since the underlying Rust logic is identical.
    - Governance: Complies with `tokmd`'s fail-closed zip admission strategy.

### Option B
- what it is: Leave Python and Node bindings without archive buffer support.
- when to choose it instead: If these languages were constrained against buffer manipulation, which they are not.
- trade-offs: Cements a permanent divergence between how browser Wasm and backend Node/Python use `tokmd`.

## ✅ Decision
Option A was chosen to fulfill the Bridge persona's directive of reducing interface drift across bindings.

## 🧱 Changes made (SRP)
- `crates/tokmd-node/src/lib.rs`: Exposed `run_json_bytes` via napi.
- `crates/tokmd-node/index.d.ts`: Documented and typed `runJsonBytes`.
- `crates/tokmd-node/index.js`: Exported `runJsonBytes`.
- `crates/tokmd-node/Cargo.toml`: Enabled `archive-zip` feature in `tokmd-core` dependency.
- `crates/tokmd-python/src/runtime.rs`: Added `run_json_bytes` via PyO3, taking care to copy the bytes into an owned `Vec` before releasing the GIL.
- `crates/tokmd-python/src/lib.rs`: Registered `run_json_bytes`.
- `crates/tokmd-python/Cargo.toml`: Enabled `archive-zip` feature in `tokmd-core` dependency.

## 🧪 Verification receipts
```text
{"cmd": "mkdir -p .jules/runs/bridge_bindings_wasm && create envelope.json", "status": "success"}
{"cmd": "patch crates/tokmd-node/src/lib.rs (run_json_bytes)", "status": "success"}
{"cmd": "patch crates/tokmd-node/index.d.ts (runJsonBytes)", "status": "success"}
{"cmd": "patch crates/tokmd-node/index.js (runJsonBytes)", "status": "success"}
{"cmd": "patch crates/tokmd-python/Cargo.toml (archive-zip feature)", "status": "success"}
{"cmd": "patch crates/tokmd-python/src/runtime.rs (run_json_bytes)", "status": "success"}
{"cmd": "patch crates/tokmd-python/src/lib.rs (run_json_bytes)", "status": "success"}
{"cmd": "patch crates/tokmd-node/Cargo.toml (archive-zip feature)", "status": "success"}
{"cmd": "cargo check -p tokmd-node --all-features", "status": "success"}
{"cmd": "cargo check -p tokmd-python --all-features", "status": "success"}
{"cmd": "cargo test -p tokmd-node --all-features", "status": "success"}
{"cmd": "cargo test -p tokmd-python --all-features", "status": "success"}
```

## 🧭 Telemetry
- Change shape: Feature parity / drift resolution
- Blast radius: API (adds new FFI function), IO (none). Safe isolated change.
- Risk class + why: Low. The new functionality is gated under the same safe core primitives `tokmd-wasm` uses.
- Rollback: Revert the PR, which hides the new binding functions.
- Gates run: `cargo check` and `cargo test` on affected crates (`tokmd-node`, `tokmd-python`) with `--all-features`.

## 🗂️ .jules artifacts
- `.jules/runs/bridge_bindings_wasm/envelope.json`
- `.jules/runs/bridge_bindings_wasm/decision.md`
- `.jules/runs/bridge_bindings_wasm/receipts.jsonl`
- `.jules/runs/bridge_bindings_wasm/result.json`
- `.jules/runs/bridge_bindings_wasm/pr_body.md`

## 🔜 Follow-ups
None required for this specific drift.
