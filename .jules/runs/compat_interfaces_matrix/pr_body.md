## 💡 Summary
Removed the `ast` feature from the `default` features of `tokmd`. This allows the crate to successfully compile for the `wasm32-unknown-unknown` target.

## 🎯 Why
The `ast` feature pulls in `tree-sitter` and its C-dependencies. Building these for the WASM target requires a C standard library (`stdlib.h`), which is not available natively in `wasm32-unknown-unknown`. Removing it from the defaults allows basic integration and WASM runners to build `tokmd` cleanly.

## 🔎 Evidence
- `crates/tokmd/Cargo.toml`
- Observed behavior: `cargo check -p tokmd --target wasm32-unknown-unknown` fails with `fatal error: 'stdlib.h' file not found`.

## 🧭 Options considered
### Option A (recommended)
- Decouple the `ast` feature from defaults.
- This fits the Compat persona by fixing a direct feature/target compatibility matrix failure in the core CLI/facade crate.
- Trade-offs: Structure is improved by avoiding unnecessary heavy C-dependencies by default; Velocity is maintained; Governance is satisfied by keeping WASM targets first-class.

### Option B
- Ignore the WASM failure and find another `--no-default-features` issue.
- Choose when WASM support is deliberately dropped.
- Trade-offs: Degrades WASM compatibility without justification.

## ✅ Decision
Option A was chosen to fix the direct build failure on `wasm32-unknown-unknown` caused by C-dependencies in `tree-sitter`.

## 🧱 Changes made (SRP)
- `crates/tokmd/Cargo.toml`: Removed `"ast"` from `default` features.

## 🧪 Verification receipts
```text
Checking tokmd-git v1.13.1 (/app/crates/tokmd-git)
Checking console v0.16.3
Checking dialoguer v0.12.0
Checking tokmd-format v1.13.1 (/app/crates/tokmd-format)
Checking indicatif v0.18.4
Checking tokmd-analysis v1.13.1 (/app/crates/tokmd-analysis)
Checking tokmd-core v1.13.1 (/app/crates/tokmd-core)
Checking tokmd-cockpit v1.13.1 (/app/crates/tokmd-cockpit)
Checking tokmd v1.13.1 (/app/crates/tokmd)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.24s
```

## 🧭 Telemetry
- Change shape: Removed feature from defaults.
- Blast radius: API/dependencies (users relying on `ast` by default will now need to opt-in).
- Risk class: Low, purely a feature definition change.
- Rollback: Revert the change to `Cargo.toml`.
- Gates run: `cargo check --target wasm32-unknown-unknown`, `cargo test --all-features`.

## 🗂️ .jules artifacts
- `.jules/runs/compat_interfaces_matrix/envelope.json`
- `.jules/runs/compat_interfaces_matrix/decision.md`
- `.jules/runs/compat_interfaces_matrix/receipts.jsonl`
- `.jules/runs/compat_interfaces_matrix/result.json`
- `.jules/runs/compat_interfaces_matrix/pr_body.md`

## 🔜 Follow-ups
None.
