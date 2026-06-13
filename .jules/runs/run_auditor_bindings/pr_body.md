## 💡 Summary
Tightened dependency declarations by removing an explicitly requested `rt-multi-thread` feature on `tokio` in `tokmd-node`. The feature is redundant because `napi`'s `async` feature already pulls it in transitively.

## 🎯 Why
The `tokmd-node` crate previously requested `rt-multi-thread` on `tokio`. This is redundant because `napi` with the `async` feature automatically pulls in `tokio_rt`, which requires a multi-threaded runtime. Keeping dependencies tight and avoiding duplicate declarations reduces manifest clutter and potential feature unification confusion.

## 🔎 Evidence
- File: `crates/tokmd-node/Cargo.toml`
- Memory confirmed: "In `napi-rs` bindings (e.g., `tokmd-node`), the `napi` dependency's `async` feature automatically pulls in `tokio` with the `rt-multi-thread` feature via `tokio_rt`. Explicitly requesting `features = ["rt-multi-thread"]` on a direct `tokio` dependency is redundant and can be safely tightened to `tokio = "1"`."

## 🧭 Options considered
### Option A (recommended)
- What it is: Remove `features = ["rt-multi-thread"]` from `tokio` in `tokmd-node`'s `Cargo.toml`.
- Why it fits this repo and shard: Directly aligns with the Auditor persona's goal to remove duplicate or redundant dependency declarations in the bindings shard.
- Trade-offs: Structure is improved, no velocity loss.

### Option B
- What it is: Investigate removing `tempfile` from `dev-dependencies`.
- When to choose it instead: If the redundancy in `tokio` was incorrect or unavailable.
- Trade-offs: Higher effort to verify test usages.

## ✅ Decision
Proceeded with Option A to remove the redundant feature declaration, as it is a safe, high-signal cleanup.

## 🧱 Changes made (SRP)
- `crates/tokmd-node/Cargo.toml`: Changed `tokio` dependency to `"1"`.

## 🧪 Verification receipts
```text
$ cargo build -p tokmd-node --verbose
(built successfully)

$ CI=true cargo test -p tokmd-node --verbose
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

## 🧭 Telemetry
- Change shape: Manifest hygiene / feature tightening
- Blast radius: None (internal to Node bindings, purely a manifest change with identical build outcome)
- Risk class: Low - build and tests confirm identical behavior.
- Rollback: Revert the `Cargo.toml` change.
- Gates run: `deps-hygiene` fallbacks.

## 🗂️ .jules artifacts
- `.jules/runs/run_auditor_bindings/envelope.json`
- `.jules/runs/run_auditor_bindings/decision.md`
- `.jules/runs/run_auditor_bindings/receipts.jsonl`
- `.jules/runs/run_auditor_bindings/result.json`
- `.jules/runs/run_auditor_bindings/pr_body.md`

## 🔜 Follow-ups
None at this time.
