## 💡 Summary
Updated `policy/non-rust-allowlist.toml` to register missing test fixtures, scripts, and policy receipts. This ensures `cargo xtask check-file-policy --strict` passes on a fresh clone.

## 🎯 Why
On a fresh checkout, running `cargo xtask check-file-policy --strict` reported 18 findings for non-Rust files that did not match any allowlist glob (such as `fixtures/bindings-parity/manifest.json`, `policy/no-panic-baseline-receipt.json`, and `scripts/check-no-bare-self-hosted.sh`). This broken gate invariant creates friction for local verification and violates the deterministic file policy contract.

## 🔎 Evidence
- file path(s): `policy/non-rust-allowlist.toml`
- observed behavior / finding: `cargo xtask check-file-policy --strict` failed with `Error: file-policy: 18 finding(s) (strict)`
- command receipt demonstrating it:
```text
Error: file-policy: 18 finding(s) (strict)
```

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `policy/non-rust-allowlist.toml` to cover all valid untracked test fixtures, policies, and scripts.
- why it fits this repo and shard: The `tooling-governance` shard governs repository invariants and file policies. Fixing a broken file-policy gate tightens deterministic behavior.
- trade-offs: Improves Governance and Velocity with minimal Structure overhead.

### Option B
- what it is: Delete the untracked fixtures.
- when to choose it instead: If the fixtures were truly unintended garbage.
- trade-offs: This would break the tests that rely on them.

## ✅ Decision
Option A. The fixtures are real and required for tests, they were simply missing from the policy allowlist, causing a broken file-policy gate out of the box.

## 🧱 Changes made (SRP)
- `policy/non-rust-allowlist.toml`: Added 10 new `[[allow]]` blocks covering test fixtures, policy receipts, and CI scripts.

## 🧪 Verification receipts
```text
$ cargo xtask check-file-policy --strict
file-policy OK: 93 entries, 1185 non-Rust files covered, 1327 Rust files skipped
```

## 🧭 Telemetry
- Change shape: Config update
- Blast radius: `policy/non-rust-allowlist.toml` (Governance)
- Risk class: Low (Tightening an existing invariant)
- Rollback: Revert the TOML additions.
- Gates run: `cargo xtask check-file-policy --strict`, `CI=true cargo test -p xtask`, `cargo xtask docs --check`, `cargo xtask boundaries-check`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None.
