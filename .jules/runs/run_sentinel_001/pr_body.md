## 💡 Summary
Tightened FFI trust boundary validation by strictly enforcing JSON object types on configuration blocks. This prevents a validation bypass where non-object values (like strings or arrays) would fall back to the root `args` or cause a panic during argument parsing.

## 🎯 Why
In `tokmd-core/src/ffi/settings_parse.rs`, JSON configuration blocks (e.g., `scan`, `lang`, `module`) were fetched using `args.get(field).unwrap_or(args)`. If a caller passed a non-object value, `unwrap_or` wouldn't trigger, returning the non-object which later would either fail unpredictably or cause the system to ignore the actual settings. This strengthens the `security-boundary` gate by guaranteeing the shape of the config object.

## 🔎 Evidence
- `crates/tokmd-core/src/ffi/settings_parse.rs`
- Observed behavior: `unwrap_or` relies on the key missing to fallback, but an invalid type present at that key circumvents the fallback logic and proceeds with a malformed value.
- Added `ffi_rejects_non_object_config_blocks` test in `ffi_trust_boundary.rs` demonstrating the fix correctly returns an error instead of a panic/success.

## 🧭 Options considered
### Option A (recommended)
- Add `get_config_block` helper that explicitly checks if the block is an object.
- Fits this repo and shard because it directly addresses the FFI trust boundary for `tokmd-core`.
- Trade-offs: Minor code change, significantly improves structural safety of the FFI interface.

### Option B
- Change `serde_json` parsing to fail immediately on any invalid type.
- This would break backward compatibility if some parameters casually allow nulls or fallbacks. Option A specifically targets the configuration blocks.

## ✅ Decision
Implemented Option A. A new `get_config_block` helper ensures nested configuration objects are actually objects or fall back securely to the root.

## 🧱 Changes made (SRP)
- `crates/tokmd-core/src/ffi/parse.rs`: Added `get_config_block` and tests.
- `crates/tokmd-core/src/ffi/settings_parse.rs`: Updated all configuration block retrievals to use `get_config_block`.
- `crates/tokmd-core/tests/ffi_trust_boundary.rs`: Added validation test.

## 🧪 Verification receipts
```text
cargo test --test ffi_trust_boundary
cargo test -p tokmd-core
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Structural hardening in FFI parse layer.
- Blast radius: API/schema. Safe fallback to default object structure.
- Risk class: Low risk. Fixes a clear validation hole.
- Rollback: Revert the PR.
- Gates run: `security-boundary`

## 🗂️ .jules artifacts
- `.jules/runs/run_sentinel_001/envelope.json`
- `.jules/runs/run_sentinel_001/decision.md`
- `.jules/runs/run_sentinel_001/receipts.jsonl`
- `.jules/runs/run_sentinel_001/result.json`
- `.jules/runs/run_sentinel_001/pr_body.md`

## 🔜 Follow-ups
None.
