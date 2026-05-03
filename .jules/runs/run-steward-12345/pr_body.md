## 💡 Summary
Removed the `RUSTSEC-2023-0071` exclusion from `deny.toml` since the offending dependency (an outdated `rsa` crate) is no longer present in our workspace or test dependencies.

## 🎯 Why
Keeping stale security exemptions clutters the metadata and masks potential future issues. The `cargo deny` check confirms this advisory is no longer encountered, so the ignore rule should be removed to maintain strict security hygiene.

## 🔎 Evidence
- `deny.toml`
- Running `cargo deny --all-features check` reported a warning: `warning[advisory-not-detected]: advisory was not encountered` for `RUSTSEC-2023-0071`.

## 🧭 Options considered
### Option A (recommended)
- Remove the stale `RUSTSEC-2023-0071` ignore rule from `deny.toml`.
- Why it fits: This is a low-risk, high-confidence metadata hygiene improvement that perfectly matches the Steward persona.
- Trade-offs:
  - Structure: Cleans up the security profile.
  - Velocity: Zero runtime cost.
  - Governance: Tightens the security gates.

### Option B
- Add missing `--profile` CLI support to `cargo xtask gate`.
- When to choose: If we were focusing on CLI UX for CI automation rather than core repository metadata.
- Trade-offs: Larger blast radius and touches the CLI interface, which is slightly higher risk than a purely metadata-level fix.

## ✅ Decision
Option A was chosen because it's a zero-risk, easily verifiable metadata cleanup that aligns squarely with the Steward's mandate to improve release/governance surfaces.

## 🧱 Changes made (SRP)
- `deny.toml`: Removed the `RUSTSEC-2023-0071` ignore block.

## 🧪 Verification receipts
```text
cargo deny --all-features check
advisories ok, bans ok, licenses ok, sources ok
```

## 🧭 Telemetry
- Change shape: Deletion
- Blast radius: metadata/CI (dependencies)
- Risk class: Low
- Rollback: git revert
- Gates run: `cargo deny --all-features check`, `cargo test --workspace`

## 🗂️ .jules artifacts
- `.jules/runs/run-steward-12345/envelope.json`
- `.jules/runs/run-steward-12345/decision.md`
- `.jules/runs/run-steward-12345/receipts.jsonl`
- `.jules/runs/run-steward-12345/result.json`
- `.jules/runs/run-steward-12345/pr_body.md`

## 🔜 Follow-ups
None
