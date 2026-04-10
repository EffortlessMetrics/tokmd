## 💡 Summary
Fixed `--no-default-features` compilation errors in the `tokmd` crate by conditionally gating `tokmd_git::git_cmd()` invocations under `#[cfg(feature = "git")]`. Fallback paths are provided to respect the feature boundary.

## 🎯 Why
The `tokmd` crate failed to compile when default features were disabled due to unguarded usages of the optional `tokmd-git` dependency in CLI surfaces. This violates the `compat-matrix` expectation that the core crate can be built without the external toolchain integrations.

## 🔎 Evidence
- `crates/tokmd/src/commands/check_ignore.rs`
- `crates/tokmd/src/commands/handoff.rs`
- Running `cargo check -p tokmd --no-default-features` previously resulted in `failed to resolve: use of unresolved module or unlinked crate tokmd_git`.

## 🧭 Options considered
### Option A (recommended)
- **What it is:** Wrap `tokmd_git::git_cmd` invocations in `#[cfg(feature = "git")]` and add dummy fallbacks using `#[cfg(not(feature = "git"))]`.
- **Why it fits:** Preserves the existing feature matrix and respects the conditional dependencies defined in `Cargo.toml`.
- **Trade-offs:** Adds slight conditional compilation noise to CLI commands, but correctly isolates external boundaries.

### Option B
- **What it is:** Make `tokmd-git` a required dependency in `Cargo.toml`.
- **When to choose it:** If git support is deemed mandatory for all environments.
- **Trade-offs:** Breaks the ability to compile the CLI without git features, reducing portability and violating existing matrix bounds.

## ✅ Decision
Option A was chosen because it correctly fixes the `--no-default-features` compatibility issue by adhering to the documented expectation that external tool invocations must be feature-gated.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/commands/check_ignore.rs`: Gated git tracked and ignored status checks.
- `crates/tokmd/src/commands/handoff.rs`: Gated git availability, repository root, and shallow clone checks.

## 🧪 Verification receipts
```text
cargo check -p tokmd --no-default-features # success
cargo test -p tokmd --all-features         # success
cargo fmt -- --check                       # success
```

## 🧭 Telemetry
- **Change shape:** Conditional compilation gates
- **Blast radius:** Compatibility build targets
- **Risk class:** Low (No behavioral changes when the feature is enabled)
- **Rollback:** Safe to revert
- **Gates run:** `cargo check --no-default-features`, `cargo test`, `cargo fmt`

## 🗂️ .jules artifacts
- `.jules/runs/run-compat-interfaces/envelope.json`
- `.jules/runs/run-compat-interfaces/decision.md`
- `.jules/runs/run-compat-interfaces/receipts.jsonl`
- `.jules/runs/run-compat-interfaces/result.json`
- `.jules/runs/run-compat-interfaces/pr_body.md`

## 🔜 Follow-ups
None
