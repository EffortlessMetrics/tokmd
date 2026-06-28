## 💡 Summary
Removed the unused `napi-build` build dependency and `build.rs` from `crates/tokmd-node`. The native build process works seamlessly via the `napi` CLI and `napi-derive` without explicitly needing the build script to run `napi_build::setup()`.

## 🎯 Why
Dependency hygiene. Removing unused dependencies reduces the compile surface area and build times, aligning with the Auditor's mission of boring, high-signal dependency cleanup.

## 🔎 Evidence
`cargo machete` flagged `napi-build` as unused in `tokmd-node`.
```text
cargo-machete found the following unused dependencies in this directory:
tokmd-node -- ./crates/tokmd-node/Cargo.toml:
	napi-build
```
Source inspection showed it was only used for `napi_build::setup()` in `build.rs`, which is redundant when using `@napi-rs/cli`. Testing locally verified the addon builds and tests correctly without it.

## 🧭 Options considered
### Option A (recommended)
- Remove `napi-build` and `build.rs` from `crates/tokmd-node`.
- Fits the repo and shard by reducing unnecessary dependencies in the Node bindings crate.
- Trade-offs: Structure is cleaner; Velocity slightly improved (less to compile).

### Option B
- Tighten other feature flags.
- Choose this if `napi-build` was actually needed for linking.
- Trade-offs: Testing showed `napi-build` was safe to remove, making Option A the stronger choice.

## ✅ Decision
Option A. It's a proven, high-signal dependency removal that directly reduces the crate's build surface.

## 🧱 Changes made (SRP)
- `crates/tokmd-node/Cargo.toml`: Removed `napi-build` from `[build-dependencies]`.
- `crates/tokmd-node/build.rs`: Deleted.

## 🧪 Verification receipts
```text
cargo machete --with-metadata
Found napi-build as unused dependency in tokmd-node

cat crates/tokmd-node/build.rs
Verified it only calls napi_build::setup()

rm crates/tokmd-node/build.rs && sed -i '/napi-build = "2"/d' crates/tokmd-node/Cargo.toml && sed -i '/\[build-dependencies\]/d' crates/tokmd-node/Cargo.toml
Removed build.rs and napi-build dependency

cargo build -p tokmd-node && cd crates/tokmd-node && npm ci && npm run build:debug && npm run test
Successfully built and ran node tests without napi-build
```

## 🧭 Telemetry
- Change shape: Dependency removal
- Blast radius: API/dependencies (Node bindings crate only)
- Risk class: Low - native build process functions correctly without it.
- Rollback: Revert `Cargo.toml` and restore `build.rs`.
- Gates run: `deps-hygiene`

## 🗂️ .jules artifacts
- `.jules/runs/auditor_bindings_manifests/envelope.json`
- `.jules/runs/auditor_bindings_manifests/decision.md`
- `.jules/runs/auditor_bindings_manifests/receipts.jsonl`
- `.jules/runs/auditor_bindings_manifests/result.json`
- `.jules/runs/auditor_bindings_manifests/pr_body.md`

## 🔜 Follow-ups
None.
