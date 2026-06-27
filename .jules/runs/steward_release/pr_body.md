## 💡 Summary
Added missing globs for fixtures, scripts, and policy artifacts to the non-Rust allowlist. This resolves 18 file-policy findings and unbreaks `cargo xtask check-file-policy --strict`.

## 🎯 Why
The `cargo xtask check-file-policy --strict` governance check was failing because newly added test fixtures, bash scripts, and JSON policy receipts were not covered by the `policy/non-rust-allowlist.toml`. This check ensures that all files committed to the repository are intentionally tracked and documented.

## 🔎 Evidence
- file path(s): `policy/non-rust-allowlist.toml`
- observed behavior / finding: `cargo xtask check-file-policy --strict` reported 18 findings for untracked files in `fixtures/`, `policy/`, and `scripts/`.
- receipt: `cargo xtask check-file-policy --strict` outputting `Error: file-policy: 18 finding(s) (strict)`

## 🧭 Options considered
### Option A (recommended)
- what it is: Add the missing globs to `policy/non-rust-allowlist.toml`.
- why it fits this repo and shard: It restores a failing governance check (file policy tracking). Fits perfectly within the `tooling-governance` shard.
- trade-offs: Structure / Velocity / Governance: Fixes broken governance checks in `xtask` related to release readiness/safety. High confidence, low risk.

### Option B
- what it is: Produce a Learning PR without fixing the file policy.
- when to choose it instead: If the files were actually malicious or accidental and needed removal rather than tracking.
- trade-offs: Missed opportunity to fix the file policy check, which is an important governance tool.

## ✅ Decision
Chose Option A to add the missing globs and restore `cargo xtask check-file-policy --strict` to a passing state.

## 🧱 Changes made (SRP)
- `policy/non-rust-allowlist.toml`: Added allowlist blocks for `fixtures/**`, `policy/*.json`, and `scripts/**`.

## 🧪 Verification receipts
```text
file-policy OK: 86 entries, 1192 non-Rust files covered, 1330 Rust files skipped
     Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.26s
     Running `target/debug/xtask check-file-policy --strict`
```

## 🧭 Telemetry
- Change shape: Metadata addition
- Blast radius: Configuration only (no API, IO, or concurrency changes)
- Risk class + why: Low. Restores a governance check without affecting product behavior.
- Rollback: Revert the PR.
- Gates run: `cargo xtask check-file-policy --strict`, `cargo xtask publish --plan --verbose`, `cargo xtask version-consistency`, `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/steward_release/envelope.json`
- `.jules/runs/steward_release/decision.md`
- `.jules/runs/steward_release/receipts.jsonl`
- `.jules/runs/steward_release/result.json`
- `.jules/runs/steward_release/pr_body.md`

## 🔜 Follow-ups
None.
