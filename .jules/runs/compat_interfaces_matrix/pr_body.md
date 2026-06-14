## 💡 Summary
Removed `ast` from the `default` features of the `tokmd` crate to restore `wasm32-unknown-unknown` compatibility. The `ast` feature relies on `tree-sitter`, which strictly requires a C standard library (`stdlib.h`) and breaks compilation for targets without it.

## 🎯 Why
WASM compatibility is a hard requirement for parts of the `tokmd` CLI facade that run in restricted environments. Because `ast` was added to `default`, standard invocations like `cargo build` on WASM fail with `stdlib.h not found`. This drift breaks the `compat-matrix` gates. By moving `ast` to an opt-in feature, we keep the capability for environments that support C-stdlib but restore the baseline matrix.

## 🔎 Evidence
- `crates/tokmd/Cargo.toml`
- **Observed behavior**:
  ```text
  warning: tree-sitter-rust@0.24.2: src/tree_sitter/parser.h:10:10: fatal error: 'stdlib.h' file not found
  ```
- **Receipts**: Removing `ast` from `default` makes `cargo check -p tokmd --target wasm32-unknown-unknown` succeed without `--no-default-features`.

## 🧭 Options considered
### Option A (recommended)
- **What it is**: Remove `ast` from `tokmd`'s `default` features.
- **Why it fits this repo and shard**: Directly addresses the compatibility breakdown while retaining functionality via explicit `--features ast`. Aligns with memory guidelines that standard WASM should not pull in `ast` by default.
- **Trade-offs**:
  - *Structure*: Corrects dependency boundary.
  - *Velocity*: Fast to land.
  - *Governance*: Requires consumers needing AST parsing to explicitly opt in.

### Option B
- **What it is**: Add conditional `cfg` flags around `tree-sitter` in `tokmd-analysis` to disable it only on WASM, even when the `ast` feature is enabled.
- **When to choose it instead**: If the `ast` feature had other significant capabilities that *did* work on WASM.
- **Trade-offs**: More complex `Cargo.toml` configuration and potential confusion where enabling a feature doesn't actually enable it on certain targets. Option A is cleaner.

## ✅ Decision
Option A. It's the simplest, most structurally sound way to fix the matrix without complicating feature flags. `ast` is explicitly designed as opt-in infrastructure for now.

## 🧱 Changes made (SRP)
- Modified `crates/tokmd/Cargo.toml` to remove `"ast"` from the `default` feature list.

## 🧪 Verification receipts
```text
$ cargo check -p tokmd --target wasm32-unknown-unknown
    Checking tokmd-cockpit v1.13.1 (/app/crates/tokmd-cockpit)
    Checking tokmd-core v1.13.1 (/app/crates/tokmd-core)
    Checking tokmd v1.13.1 (/app/crates/tokmd)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.07s
```

## 🧭 Telemetry
- Change shape: Feature flag reduction
- Blast radius: API (consumers relying on default `ast` features will need to opt-in)
- Risk class: Low (matrix repair)
- Rollback: Revert `Cargo.toml` change
- Gates run: `cargo check -p tokmd --target wasm32-unknown-unknown`, `cargo check -p tokmd --no-default-features`, `cargo build --verbose`, `cargo test -p tokmd --verbose`

## 🗂️ .jules artifacts
- `.jules/runs/compat_interfaces_matrix/envelope.json`
- `.jules/runs/compat_interfaces_matrix/decision.md`
- `.jules/runs/compat_interfaces_matrix/receipts.jsonl`
- `.jules/runs/compat_interfaces_matrix/result.json`
- `.jules/runs/compat_interfaces_matrix/pr_body.md`

## 🔜 Follow-ups
None.
