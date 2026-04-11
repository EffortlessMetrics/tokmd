## 💡 Summary
Optimized dependency hygiene in `tokmd-format` by restricting the `js` feature on the `uuid` crate exclusively to Wasm targets. This avoids compiling unnecessary JavaScript transitives (like `wasm-bindgen` and `js-sys`) when building the native CLI or standard Rust binaries.

## 🎯 Why
The `tokmd-format` crate had `uuid` configured with the `js` feature unconditionally. In Rust, enabling this feature pulls in web and JS interop dependencies to allow `uuid` to use Web Crypto APIs. For native applications (like the CLI `tokmd`), these transitives are completely dead weight and unnecessarily bloat the compile surface and dependency tree.

## 🔎 Evidence
File path: `crates/tokmd-format/Cargo.toml`
Observation: The `uuid` dependency was declared as `uuid = { version = "1.22", features = ["v4", "js"] }`.

## 🧭 Options considered
### Option A (recommended)
- Move the `js` feature of the `uuid` dependency to a `[target.'cfg(target_arch = "wasm32")'.dependencies]` block.
- **Why it fits:** It correctly maps the target-specific requirement to the target arch, ensuring native builds remain lean.
- **Trade-offs:** Minimal complexity increase in `Cargo.toml`. Keeps native build fast and clean.

### Option B
- Keep the `js` feature unconditionally.
- **When to choose:** If the primary targets are exclusively Wasm and there's no native build.
- **Trade-offs:** Harms native compile velocity and brings unnecessary WASM/JS transitives into standard native toolchains.

## ✅ Decision
Option A. It isolates the `js` feature to `wasm32` architectures, fixing the dependency drift without breaking web usage.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/Cargo.toml`

## 🧪 Verification receipts
```text
cargo check -p tokmd-format
cargo test -p tokmd-format
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Dependency feature restriction
- Blast radius: `tokmd-format` compilation surface
- Risk class: Low
- Rollback: Revert Cargo.toml changes
- Gates run: `cargo check`, `cargo test`, `cargo clippy`

## 🗂️ .jules artifacts
- `.jules/runs/auditor_core_manifests/envelope.json`
- `.jules/runs/auditor_core_manifests/decision.md`
- `.jules/runs/auditor_core_manifests/receipts.jsonl`
- `.jules/runs/auditor_core_manifests/result.json`
- `.jules/runs/auditor_core_manifests/pr_body.md`

## 🔜 Follow-ups
None.
