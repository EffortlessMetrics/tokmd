## 💡 Summary
Added missing file-policy allowlist entries for `fixtures/syntax/python`, `fixtures/syntax/typescript`, and `scripts/*.sh`. This resolves strict file-policy checker failures and locks in governance for these new fixture and script files.

## 🎯 Why
The `cargo xtask check-file-policy --strict` gate was failing due to 4 unallowlisted non-Rust files (`fixtures/syntax/python/native_boundary.py`, `fixtures/syntax/typescript/component.tsx`, `fixtures/syntax/typescript/native_boundary.ts`, and `scripts/check-no-bare-self-hosted.sh`). The Gatekeeper persona enforces deterministic policy invariants; all non-Rust files must be mapped to an owner, kind, and reason.

## 🔎 Evidence
- `policy/non-rust-allowlist.toml`
- Running the file policy checker in strict mode failed on the new test fixtures and scripts.
- `cargo xtask check-file-policy --strict` failure output.

## 🧭 Options considered
### Option A (recommended)
- Add explicit `[[allow]]` blocks in `policy/non-rust-allowlist.toml` for the new syntax fixtures and scripts.
- Why it fits this repo and shard: Directly aligns with the `tooling-governance` shard's mandate to maintain deterministic workspace policies.
- Trade-offs: Structure is improved by enforcing invariants. No velocity penalty. Enhances governance.

### Option B
- Wait for the authors of the test fixtures to fix it themselves.
- When to choose it instead: If the files were temporary or mistakenly committed.
- Trade-offs: Leaves the `check-file-policy --strict` gate broken in the meantime, violating the Gatekeeper mandate to lock in invariants.

## ✅ Decision
Option A. Enforcing the file policy allowlist is exactly the kind of deterministic governance fix the Gatekeeper is designed for.

## 🧱 Changes made (SRP)
- `policy/non-rust-allowlist.toml`

## 🧪 Verification receipts
```text
$ cargo xtask check-file-policy --strict
file-policy OK: 87 entries, 1157 non-Rust files covered, 1309 Rust files skipped
```

## 🧭 Telemetry
- Change shape: Policy ledger update
- Blast radius: build / policy (No API or production changes).
- Risk class: Low. Restores CI gate.
- Rollback: Revert the PR.
- Gates run: `cargo xtask check-file-policy --strict`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None.
