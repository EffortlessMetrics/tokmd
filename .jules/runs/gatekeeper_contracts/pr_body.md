## 💡 Summary
Locked in missing non-Rust file paths in the policy allowlist, including xtask test artifacts.

## 🎯 Why
`cargo xtask check-file-policy` was surfacing unallowlisted syntax fixtures, CI scripts, and xtask artifacts, threatening the explicit determinism contract.

## 🔎 Evidence
`cargo xtask check-file-policy`

## 🧭 Options considered
### Option A (recommended)
- Explicitly allow the testing fixtures, scripts, and xtask build artifacts.
- Protects structural governance.
- Trade-offs: Structure (stronger), Velocity (faster local runs), Governance (strict).

### Option B
- Delete the files.
- Choose if cruft.
- Trade-offs: We lose testing fixtures and security tools.

## ✅ Decision
Option A. The files provide value but were missed in the policy ledger.

## 🧱 Changes made (SRP)
- `policy/non-rust-allowlist.toml`

## 🧪 Verification receipts
```text
$ cargo test -p xtask
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.17s

$ cargo xtask check-file-policy --strict
file-policy OK: 86 entries, 1168 non-Rust files covered, 1309 Rust files skipped
```

## 🧭 Telemetry
- Change shape: schema addition
- Blast radius: governance checks only
- Risk class: trivial
- Rollback: safe
- Gates run: `cargo xtask check-file-policy`, `cargo test -p xtask`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None.
