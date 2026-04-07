## 💡 Summary
Opt the `tokmd-python` extension out of standard tests via `test = false` in its `Cargo.toml`. This resolves severe PyO3 linker errors (`undefined symbol: PyErr_NewExceptionWithDoc`) triggered by workspace-wide `cargo test --all-features` runs.

## 🎯 Why
When `cargo test --all-features --workspace` is executed, Cargo attempts to build PyO3 libraries that rely on the `extension-module` feature as standard test binaries. Because these test binaries do not link against the Python shared library themselves (unlike standard usage via `python/pytest`), linker errors occur due to missing Python symbols. This prevents developers from cleanly running all workspace tests without complex configuration.

## 🔎 Evidence
- File path: `crates/tokmd-python/Cargo.toml`
- Observed behavior: `cargo test --all-features -p tokmd-python` failed with linker errors.
- Output excerpt before fix:
  ```text
  rust-lld: error: undefined symbol: PyErr_NewExceptionWithDoc
  >>> referenced by mod.rs:451 (src/err/mod.rs:451)
  error: could not compile `tokmd-python` (lib test) due to 1 previous error
  ```

## 🧭 Options considered
### Option A (recommended)
- Explicitly set `test = false` in the `[lib]` section of `tokmd-python`'s `Cargo.toml`.
- Fits the repo and shard because it leverages Cargo's built-in target filtering to skip linking tests for native extensions entirely, which is the official recommendation in PyO3 documentation for the `extension-module` feature.
- Trade-offs:
  - Structure: Minimal change localized to the specific crate.
  - Velocity: Restores the ability to simply run `cargo test --workspace`.
  - Governance: Consistent with Rust bindings ecosystem standards.

### Option B
- Dynamically build and test extension modules using custom test runners or complex `#[cfg(not(test))]` feature flags around `extension-module`.
- When to choose it instead: If the `tokmd-python` crate actually contained complex internal Rust integration tests rather than purely being an FFI shim for pytest.
- Trade-offs: Substantially more fragile, higher overhead, and requires teaching developers a non-standard workflow.

## ✅ Decision
Option A. It's the most idiomatic and reliable way to ensure PyO3 `extension-module` crates do not break `cargo test` in a workspace.

## 🧱 Changes made (SRP)
- `crates/tokmd-python/Cargo.toml`: Added `test = false` under `[lib]`.

## 🧪 Verification receipts
```text
cargo test --all-features -p tokmd-python
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
```

## 🧭 Telemetry
- Change shape: Build configuration patch.
- Blast radius: Only affects the build pipeline and test runner behavior of `tokmd-python`.
- Risk class: Low, purely a build tooling fix with no runtime changes.
- Rollback: Revert the `Cargo.toml` change.
- Gates run: `cargo test --all-features -p tokmd-python`, `cargo fmt -- --check`, `cargo clippy -p tokmd-python -- -D warnings`.

## 🗂️ .jules artifacts
- `.jules/runs/compat_targets_matrix/envelope.json`
- `.jules/runs/compat_targets_matrix/decision.md`
- `.jules/runs/compat_targets_matrix/receipts.jsonl`
- `.jules/runs/compat_targets_matrix/result.json`
- `.jules/runs/compat_targets_matrix/pr_body.md`

## 🔜 Follow-ups
None required.
