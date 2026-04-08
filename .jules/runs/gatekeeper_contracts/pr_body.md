## 💡 Summary
Configured the `tokmd-python` PyO3 extension crate to opt-out of standard cargo tests and removed the manual `--exclude tokmd-python` overrides from workspace quality gates. This locks in deterministic testing expectations across the workspace without sacrificing type-checking and linting.

## 🎯 Why
Running `cargo test --workspace --all-features` natively caused linking errors for `tokmd-python` because it produces a native python extension (via PyO3's `extension-module` feature) which cannot find python interpreter symbols during standard binary linking. To work around this, `cargo xtask gate` was previously skipping `tokmd-python` altogether during `cargo check` and `cargo clippy`, unnecessarily leaving the extension crate without automated linting or checking during PR pre-flight checks.

## 🔎 Evidence
Observed in `xtask/src/tasks/gate.rs`:
```rust
    Step {
        label: "clippy",
        cmd: "cargo",
        args: &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--exclude",
            "tokmd-python",
            "--",
            "-D",
            "warnings",
        ],
//...
```
This is a sharp edge for deterministic execution, as it required custom scripting memory just to avoid testing linking bugs instead of natively configuring `Cargo.toml`.

## 🧭 Options considered
### Option A (recommended)
- Add `test = false` to the `[lib]` section of `tokmd-python`'s `Cargo.toml` and remove `--exclude` from `gate.rs`.
- Why it fits: This tells cargo explicitly that the library target shouldn't be built as a standalone test executable, which is the idiomatic PyO3 fix.
- Trade-offs: Structure is much better because we no longer have to manually remember to exclude the crate when running full workspace checks. Velocity improves as standard cargo workspace commands work natively.

### Option B
- Keep the explicit exclusions in `gate.rs` and ignore the linker error for manual test runs.
- When to choose it instead: If testing the extension module locally using native `cargo test` required custom linkage flags anyway.
- Trade-offs: Degrades developer experience and leaves a sharp edge for workspace actions. It also drops `check` and `clippy` coverage for the python crate.

## ✅ Decision
Implemented Option A. It correctly conveys our testing expectations to Cargo, resolving the linker errors globally and allowing us to remove the `exclude` overrides from the `xtask` gate, bringing `tokmd-python` back into standard workspace static analysis checks.

## 🧱 Changes made (SRP)
- `crates/tokmd-python/Cargo.toml`: Added `test = false` to the `[lib]` block.
- `xtask/src/tasks/gate.rs`: Removed `--exclude tokmd-python` from the `check`, `clippy`, and `test` execution steps.

## 🧪 Verification receipts
```text
cargo check --workspace --all-features
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.02s

cargo test --workspace --all-features --no-run
    Executable unittests src/lib.rs ... (all crates compile successfully without linker errors)

cargo xtask gate
[1/4] fmt
   ✅ Step 1 (fmt) passed
[2/4] check (warm graph)
   ✅ Step 2 (check (warm graph)) passed
[3/4] clippy
   ✅ Step 3 (clippy) passed
[4/4] test (compile-only)
   ✅ Step 4 (test (compile-only)) passed
gate result: 4/4 steps passed
```

## 🧭 Telemetry
- Change shape: Config optimization and gate simplification
- Blast radius: Workspace tooling
- Risk class: Low - fixes a build/linker error for testing and improves lint coverage.
- Rollback: Revert the PR.
- Gates run: `cargo xtask gate`, `cargo test --workspace --all-features --no-run`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None.
