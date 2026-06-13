## 💡 Summary
Removed the `ast` feature from the `default` features of the `tokmd` crate to allow building for `wasm32-unknown-unknown` out of the box.

## 🎯 Why
The `ast` feature pulls in `tree-sitter` and its parsers, which rely on the C standard library (`stdlib.h`). This breaks builds for `wasm32-unknown-unknown` where `stdlib.h` is not available by default. By removing it from `default` features, WASM compatibility is restored without breaking existing functionality.

## 🔎 Evidence
Running `cargo check -p tokmd --no-default-features --features ast --target wasm32-unknown-unknown` fails with:
`fatal error: 'stdlib.h' file not found`

## 🧭 Options considered
### Option A (recommended)
- what it is: Remove `ast` from the `default` features of the `tokmd` crate.
- why it fits this repo and shard: It restores WASM compatibility which aligns with the `compat-matrix` gate, and is requested directly in the instructions.
- trade-offs: Users relying on the `ast` feature will need to explicitly enable it.

### Option B
- what it is: Provide a custom `cfg` attribute in `Cargo.toml` to disable the `ast` feature for wasm targets.
- when to choose it instead: When `ast` MUST be default for all non-wasm targets.
- trade-offs: Increases complexity in Cargo manifests and feature propagation.

## ✅ Decision
Proceeded with Option A as it is the most robust way to handle this compatibility constraint, aligning perfectly with memory directives.

## 🧱 Changes made (SRP)
- `crates/tokmd/Cargo.toml`: Removed `ast` from `default` features.

## 🧪 Verification receipts
`cargo check -p tokmd --target wasm32-unknown-unknown` completes successfully.

## 🧭 Telemetry
- Change shape: small config change
- Blast radius: feature boundaries
- Risk class: low
- Rollback: Re-add `ast` to `default` features.
- Gates run: `compat-matrix` fallback tests.

## 🗂️ .jules artifacts
- `.jules/runs/compat_interfaces_matrix/envelope.json`
- `.jules/runs/compat_interfaces_matrix/decision.md`
- `.jules/runs/compat_interfaces_matrix/receipts.jsonl`
- `.jules/runs/compat_interfaces_matrix/result.json`
- `.jules/runs/compat_interfaces_matrix/pr_body.md`

## 🔜 Follow-ups
None.
