## 💡 Summary
Removed the `pyo3-build-config` build dependency from `tokmd-python`. Since the crate lacks a `build.rs` file, this build dependency was completely inert and unresolvable by the build process, serving only as manifest noise.

## 🎯 Why
Dependency hygiene. Removing unused build dependencies speeds up lockfile resolution, trims the dependency graph, and avoids confusion for future maintainers.

## 🔎 Evidence
- **File**: `crates/tokmd-python/Cargo.toml`
- **Finding**: `pyo3-build-config` was listed under `[build-dependencies]`, but `ls -la crates/tokmd-python/build.rs` confirmed the file does not exist. Cargo ignores `build-dependencies` entirely without a `build.rs`.

## 🧭 Options considered
### Option A (recommended)
- **What it is**: Remove `pyo3-build-config` from `crates/tokmd-python/Cargo.toml`.
- **Why it fits this repo and shard**: Aligns perfectly with the Auditor persona's mission to remove unused direct dependencies in the `bindings-targets` shard. The `tokmd-python` crate has no `build.rs`.
- **Trade-offs**: Cleaner manifest, slightly faster resolution, no downside since the code literally cannot use a build-dependency without a `build.rs`.

### Option B
- **What it is**: Leave `pyo3-build-config` in `tokmd-python`.
- **When to choose it instead**: Never, unless a tool specifically inspects the manifest without using Cargo's standard build process (which is not standard practice).
- **Trade-offs**: Leaves an unused dependency in the manifest, violating dependency hygiene goals.

## ✅ Decision
Option A. Removed the unused build dependency to improve manifest hygiene and build graph clarity, perfectly fulfilling the assignment requirements.

## 🧱 Changes made (SRP)
- Removed `[build-dependencies]` and `pyo3-build-config = "0.28.3"` from `crates/tokmd-python/Cargo.toml`.

## 🧪 Verification receipts
```text
$ ls -la crates/tokmd-python/build.rs
ls: cannot access 'crates/tokmd-python/build.rs': No such file or directory

$ cargo build -p tokmd-python
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.24s

$ cargo test -p tokmd-python
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

$ cargo xtask version-consistency
Version consistency checks passed.
```

## 🧭 Telemetry
- **Change shape**: Dependency removal (hygiene).
- **Blast radius**: Very low. `tokmd-python` builds correctly without it because there is no `build.rs`.
- **Risk class**: Trivial. Only modifying a package manifest.
- **Rollback**: Revert the commit.
- **Gates run**: `cargo build`, `cargo test`, `cargo xtask version-consistency`, `cargo xtask publish --plan`. `cargo deny` is unavailable in the environment.

## 🗂️ .jules artifacts
- `.jules/runs/auditor_bindings_manifests/envelope.json`
- `.jules/runs/auditor_bindings_manifests/decision.md`
- `.jules/runs/auditor_bindings_manifests/receipts.jsonl`
- `.jules/runs/auditor_bindings_manifests/result.json`
- `.jules/runs/auditor_bindings_manifests/pr_body.md`
- Added friction item: `.jules/friction/open/cargo_deny_missing.md`

## 🔜 Follow-ups
- `cargo deny` is not installed in the environment (documented in friction item).
