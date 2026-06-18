## 💡 Summary
Tightened the `tokio` dependency in `tokmd-node` by removing the explicit `rt-multi-thread` feature requirement. The `napi` crate (with the `async` feature) already automatically pulls in the `rt-multi-thread` runtime, making the direct declaration redundant.

## 🎯 Why
Dependency hygiene. Explicitly requesting duplicate features that are already guaranteed by another core dependency increases manifest noise and potential for conflicting feature requests if defaults change upstream. Removing it is a clean, boring improvement.

## 🔎 Evidence
- `crates/tokmd-node/Cargo.toml`
- Dependency inspection via `cargo tree`:
```text
├── tokio feature "rt-multi-thread" (*)
│   ├── tokio v1.52.3 (*)
│   └── tokio feature "rt" (*)
└── napi feature "tokio_rt"
    ├── napi v3.9.0 (*)
```

## 🧭 Options considered
### Option A (recommended)
- Remove the redundant `rt-multi-thread` feature request from the direct `tokio` dependency in `crates/tokmd-node/Cargo.toml`.
- Fits the Auditor persona's dependency hygiene goals.
- Trade-offs: Structure is improved by removing redundancy. Velocity is unaffected. Governance matches dependency hygiene goals.

### Option B
- Remove `tokio` entirely from `crates/tokmd-node/Cargo.toml`.
- When to choose: If the crate doesn't use `tokio` directly in its source code.
- Trade-offs: `crates/tokmd-node/src/lib.rs` uses `tokio::task::spawn_blocking`, so it needs the direct `tokio` dependency. Breaking the build.

## ✅ Decision
Option A was chosen as it safely removes a redundant feature without breaking compilation, ensuring clean dependency hygiene.

## 🧱 Changes made (SRP)
- `crates/tokmd-node/Cargo.toml`

## 🧪 Verification receipts
```text
$ cd crates/tokmd-node && cargo tree -p tokmd-node | grep tokio
├── tokio v1.52.3
```

## 🧭 Telemetry
- Change shape: Removal of redundant feature flag
- Blast radius: None. Safe patch within the manifest.
- Risk class: Low
- Rollback: Revert the Cargo.toml change
- Gates run: `cargo build -p tokmd-node`, `cargo fmt`, `cargo clippy`, `cargo deny`

## 🗂️ .jules artifacts
- `.jules/runs/auditor_bindings_manifests/envelope.json`
- `.jules/runs/auditor_bindings_manifests/decision.md`
- `.jules/runs/auditor_bindings_manifests/receipts.jsonl`
- `.jules/runs/auditor_bindings_manifests/result.json`
- `.jules/runs/auditor_bindings_manifests/pr_body.md`

## 🔜 Follow-ups
None.
