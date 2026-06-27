## 💡 Summary
Fix JSON validation drift across FFI bindings (Wasm, Python, Node). The bindings now strictly validate that raw JSON arguments represent a top-level object before passing them to the core execution engine, failing fast with native errors.

## 🎯 Why
`tokmd-core` explicitly rejects top-level scalar or array JSON payloads (`"0"`, `"[]"`) with a clear error: `Top-level JSON value must be an object`. However, the Wasm, Python, and Node bindings were not enforcing this check consistently before delegating to the core, leading to behavioral drift and differing error representations across targets.

## 🔎 Evidence
Memory context dictates:
- "In `tokmd` FFI bindings, raw JSON arguments must always be strictly validated as top-level JSON objects to maintain parity with `tokmd-core` (which fails fast if `!args.is_object()`)."

## 🧭 Options considered
### Option A (recommended)
- Fix strict JSON object validation drift in FFI bindings.
- Fits the `bindings-targets` shard by reducing drift between Rust core and target boundaries.
- Trade-offs: Improves correctness and error parity at the cost of duplicate parsing at the FFI boundary.

### Option B
- Rely solely on `tokmd-core` error propagation.
- When to choose: If avoiding duplicate JSON parsing is critical.
- Trade-offs: Prevents bindings from raising early native exceptions (e.g., Python `ValueError` before GIL release).

## ✅ Decision
Option A was chosen to fulfill the specific memory instruction to strictly validate top-level JSON objects in FFI bindings, ensuring consistent failure modes across all targets.

## 🧱 Changes made (SRP)
- `crates/tokmd-wasm/src/lib.rs`: Added `value.is_object()` check in `normalize_raw_json_args` and a regression test.
- `crates/tokmd-python/src/runtime.rs`: Added `value.is_object()` check in `run_json`.
- `crates/tokmd-python/tests/test_basic.py`: Updated invalid JSON test to expect `ValueError` instead of an error envelope, and added a test for scalar/array JSON inputs.
- `crates/tokmd-node/src/lib.rs`: Added `value.is_object()` check in `run_json` and a regression test.

## 🧪 Verification receipts
```text
cargo test -p tokmd-wasm -p tokmd-python -p tokmd-node --no-default-features
cargo test -p tokmd-wasm -p tokmd-python -p tokmd-node --all-features
wasm-pack test --node (inside crates/tokmd-wasm)
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Logic modification + tests
- Blast radius: FFI bindings inputs (API)
- Risk class: Low, only rejects strictly invalid arguments that would fail in core anyway.
- Rollback: Revert PR.
- Gates run: `compat-matrix` fallback validation.

## 🗂️ .jules artifacts
- `.jules/runs/bridge_bindings_wasm/envelope.json`
- `.jules/runs/bridge_bindings_wasm/decision.md`
- `.jules/runs/bridge_bindings_wasm/receipts.jsonl`
- `.jules/runs/bridge_bindings_wasm/result.json`
- `.jules/runs/bridge_bindings_wasm/pr_body.md`

## 🔜 Follow-ups
None.
