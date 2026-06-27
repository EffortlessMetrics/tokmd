## 💡 Summary
Added missing allowlist entries for `fixtures/**` and `scripts/**` to the non-Rust file policy. This resolves 4 strict findings and restores a clean run for `cargo xtask check-file-policy --strict`.

## 🎯 Why
The strict file policy checker is a deterministic governance gate that ensures all non-Rust files in the repository have an explicit owner, surface, and justification. Several files in `fixtures/syntax/` and `scripts/` were recently added or drifted without corresponding policy entries, causing the strict check to fail and blocking determinism gates.

## 🔎 Evidence
```text
file-policy findings (4):
  - file fixtures/syntax/python/native_boundary.py does not match any non-Rust allowlist glob
  - file fixtures/syntax/typescript/component.tsx does not match any non-Rust allowlist glob
  - file fixtures/syntax/typescript/native_boundary.ts does not match any non-Rust allowlist glob
  - file scripts/check-no-bare-self-hosted.sh does not match any non-Rust allowlist glob
check-file-policy failed
```

## 🧭 Options considered
### Option A (recommended)
- Add deterministic `fixtures/**` and `scripts/**` globs to the file policy `policy/non-rust-allowlist.toml`.
- Fits the tooling-governance shard by maintaining strict file ownership policies.
- Trade-offs: Structure/Governance +1, restores a passing build with no runtime footprint.

### Option B
- Ignore the strict flag.
- When to choose: Never for a deterministic gatekeeper persona.
- Trade-offs: Degrades trust in the automated governance.

## ✅ Decision
Option A was chosen to fix the broken file policy checker by aligning the allowed globs with the existing files in `fixtures/` and `scripts/`.

## 🧱 Changes made (SRP)
- `policy/non-rust-allowlist.toml`: Appended allowlist entries for `fixtures/**` and `scripts/**`.

## 🧪 Verification receipts
```text
$ cargo xtask check-file-policy --strict
file-policy OK: 85 entries, 1157 non-Rust files covered, 1309 Rust files skipped
```

## 🧭 Telemetry
- Change shape: Metadata addition
- Blast radius: Configuration / Policy / GitHub Actions (no runtime impact)
- Risk class: Low, only expands file policy allowlist.
- Rollback: Revert the TOML changes.
- Gates run: `cargo xtask check-file-policy --strict`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None at this time.
