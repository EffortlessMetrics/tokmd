## 💡 Summary
Removed the deprecated `unmaintained = "all"` key from `deny.toml` to restore compatibility with `cargo-deny@0.19.0` used in CI.

## 🎯 Why
The CI workflow uses `cargo-deny@0.19.0`, which breaks on the `unmaintained = "all"` configuration key, causing the `deny` job to fail. Removing it resolves the failure and keeps release hygiene checks passing.

## 🔎 Evidence
- `deny.toml`
- Observed behavior: `error[deprecated]: this key has been removed` when running `cargo deny --all-features check`.
- Check passes after removing the key.

## 🧭 Options considered
### Option A (recommended)
- Remove the deprecated `unmaintained = "all"` key from `deny.toml`.
- It fits this repo by keeping the CI tools updated and fixing a broken configuration.
- Trade-offs: Structure / Velocity / Governance - Low risk, keeps us aligned with upstream tool changes.

### Option B
- Pin `cargo-deny` in CI to an older version (e.g. 0.16.3).
- Choose this if we heavily depend on a deprecated feature that hasn't been replaced.
- Trade-offs: Leaves us stranded on an old version of a security-checking tool.

## ✅ Decision
Option A. It's better to stay current with tooling and adapt the config rather than pinning to an outdated tool version, especially for security-related tools like `cargo-deny`.

## 🧱 Changes made (SRP)
- `deny.toml`

## 🧪 Verification receipts
```text
$ cargo deny --all-features check
error[deprecated]: this key has been removed, see https://github.com/EmbarkStudios/cargo-deny/pull/611 for migration information
   ┌─ /app/deny.toml:18:1
   │
18 │ unmaintained = "all"
   │ ━━━━━━━━━━━━

$ sed -i '/unmaintained = "all"/d' deny.toml
$ cargo deny --all-features check
advisories ok, bans ok, licenses ok, sources ok
```

## 🧭 Telemetry
- Change shape: Config fix
- Blast radius: CI gates
- Risk class + why: Low, only modifies cargo-deny configuration file.
- Rollback: Revert commit
- Gates run: `cargo deny --all-features check`

## 🗂️ .jules artifacts
- `.jules/runs/steward_release/envelope.json`
- `.jules/runs/steward_release/decision.md`
- `.jules/runs/steward_release/receipts.jsonl`
- `.jules/runs/steward_release/result.json`
- `.jules/runs/steward_release/pr_body.md`

## 🔜 Follow-ups
None.
