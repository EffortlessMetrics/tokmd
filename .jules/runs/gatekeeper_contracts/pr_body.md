## 💡 Summary
Appends missing file globs for `fixtures`, `policy/*-baseline-receipt.json`, and `scripts/*.sh` to `policy/non-rust-allowlist.toml` to fix the `file-policy` checker errors.

## 🎯 Why
The `file-policy` gate ensures deterministic tracking and ownership of non-Rust files in the repository. The strict check (`cargo xtask check-file-policy --strict`) was failing because 18 files, primarily fixtures, policy receipts, and build scripts, were not covered by any allowlist glob, leading to policy semantic drift.

## 🔎 Evidence
File path(s): `policy/non-rust-allowlist.toml`
Observed behavior:
```
Error: file-policy: 18 finding(s) (strict)
```
Command run: `cargo xtask check-file-policy --strict`

## 🧭 Options considered
### Option A
- Fix `bindings-parity --check` returning an error message when run without arguments, since update mode is not implemented.
- when to choose it instead: This is an ergonomic improvement for the CLI.
- trade-offs: Doesn't directly address a broken contract or strict gate failure, unlike the file policy check.

### Option B (recommended)
- Fix file-policy findings by adding them to `non-rust-allowlist.toml`.
- why it fits this repo and shard: It directly fixes a broken strict gate check that is supposed to run deterministically.
- trade-offs: Structure / Velocity / Governance: Improves Governance by ensuring strict adherence to the file policy without compromising velocity.

## ✅ Decision
Chosen Option B. It aligns perfectly with the Gatekeeper persona's mission to "Protect contract-bearing surfaces and lock in deterministic behavior", specifically targeting "policy/gate semantic drift".

## 🧱 Changes made (SRP)
- `policy/non-rust-allowlist.toml` - Added allowlist entries for `fixtures/**`, `policy/*-baseline-receipt.json`, and `scripts/*.sh`.

## 🧪 Verification receipts
```text
cargo xtask check-file-policy --strict
file-policy OK: 86 entries, 1186 non-Rust files covered, 1327 Rust files skipped
```

## 🧭 Telemetry
- Change shape: config-update
- Blast radius: build / policy
- Risk class: Low
- Rollback: Revert PR
- Gates run: `cargo xtask check-file-policy --strict`, `cargo test -p xtask`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None.
