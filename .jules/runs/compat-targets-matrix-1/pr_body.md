## 💡 Summary
This is a learning PR. The `bindings-targets` shard passes all feature and target matrix compatibility checks, but a workspace-wide `--no-default-features` check uncovered a compilation error in the adjacent `tokmd` CLI crate.

## 🎯 Why
The `tokmd` crate unconditionally references `tokmd_git::git_cmd` without properly gating it under the `#[cfg(feature = "git")]` attribute. Since the issue lives outside the assigned `bindings-targets` shard, it has been logged as a friction item to preserve the single-responsibility boundary of the shard.

## 🔎 Evidence
File: `crates/tokmd/src/commands/check_ignore.rs`
Observed behavior:
When compiled without default features, the build fails with:
```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `tokmd_git`
   --> crates/tokmd/src/commands/baseline.rs:189:18
    |
189 |     let output = tokmd_git::git_cmd()
```

## 🧭 Options considered
### Option A (recommended)
- Submit a learning PR and record the friction item for the out-of-shard compilation error.
- Fits the repo guidelines which instruct agents not to chase issues outside of their assigned paths, optimizing for bounded, safe exploration.
- Trade-offs: Structure is preserved by not muddying the PR with out-of-scope files; Governance rules are respected; Velocity might slightly reduce on this specific fix but remains focused.

### Option B
- Forcibly fix the `tokmd` CLI crate in this run.
- Should be chosen only if the explicit instruction allowed arbitrary scope creep.
- Trade-offs: Breaks the `bindings-targets` shard isolation and conflates binding matrices with CLI architecture.

## ✅ Decision
Option A. I am respecting the shard isolation constraint and filing the issue as a friction item.

## 🧱 Changes made (SRP)
- Added `.jules/friction/open/tokmd-no-default-features-git.md` to document the out-of-shard failure.

## 🧪 Verification receipts
```text
cargo check -p tokmd-wasm --no-default-features (Success)
cargo check -p tokmd-wasm --all-features (Success)
cargo test --target wasm32-unknown-unknown -p tokmd-wasm --no-default-features (Success)
cargo check -p tokmd --no-default-features (Failed: error[E0433]: failed to resolve: use of unresolved module or unlinked crate `tokmd_git`)
```

## 🧭 Telemetry
- Change shape: Documentation / Friction Logging
- Blast radius: None (documentation only)
- Risk class: Low - no code changes applied.
- Rollback: Revert the added friction file.
- Gates run: `cargo check` and `cargo test` across features and targets.

## 🗂️ .jules artifacts
- `.jules/runs/compat-targets-matrix-1/envelope.json`
- `.jules/runs/compat-targets-matrix-1/decision.md`
- `.jules/runs/compat-targets-matrix-1/receipts.jsonl`
- `.jules/runs/compat-targets-matrix-1/result.json`
- `.jules/runs/compat-targets-matrix-1/pr_body.md`
- Friction item: `.jules/friction/open/tokmd-no-default-features-git.md`

## 🔜 Follow-ups
- A subsequent agent assigned to `tokmd` or `core-cli` should resolve the `tokmd` `--no-default-features` compilation failure documented in `.jules/friction/open/tokmd-no-default-features-git.md`.
