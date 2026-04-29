## 💡 Summary
Cleaned up stale governance metadata from `deny.toml`. Removed the `Unicode-DFS-2016` license and the `ring` license clarification, as these are no longer present in our workspace lockfile, resolving `license-not-encountered` warnings during `cargo deny` gate checks.

## 🎯 Why
When `cargo deny --all-features check` runs, it emits `license-not-encountered` warnings if allowed licenses or clarification blocks in `deny.toml` are no longer used by any dependency in the workspace lockfile. Keeping `deny.toml` strictly aligned with the actual lockfile reduces warning fatigue and ensures release gates fail only on actual regressions.

## 🔎 Evidence
- File path: `deny.toml`
- Observed behavior: `cargo deny check licenses` threw `warning[license-not-encountered]: license was not encountered` for `Unicode-DFS-2016`.
- Receipt:
```text
warning[license-not-encountered]: license was not encountered
   ┌─ /app/deny.toml:41:6
   │
41 │     "Unicode-DFS-2016",
   │      ━━━━━━━━━━━━━━━━ unmatched license allowance
```

## 🧭 Options considered
### Option A (recommended)
- what it is: Remove unused `Unicode-DFS-2016` from `licenses.allow` and remove the stale `licenses.clarify` block for `ring` in `deny.toml`.
- why it fits this repo and shard: It resolves `cargo deny` warnings about unencountered licenses, which are part of our governance gates. The `tooling-governance` shard explicitly covers `deny.toml` since it's governance configuration.
- trade-offs: Structure is cleaner; velocity is fast; improves governance by avoiding warning fatigue.

### Option B
- what it is: Ignore the warnings.
- when to choose it instead: If the dependencies were temporarily removed but expected to be re-added shortly.
- trade-offs: We keep noisy warnings in our release gates.

## ✅ Decision
Option A was chosen. It directly aligns with the `tooling-governance` shard guidelines to improve release hygiene by resolving actionable `cargo deny` warnings and removing stale state.

## 🧱 Changes made (SRP)
- `deny.toml`: Removed `"Unicode-DFS-2016"` from `[licenses.allow]`.
- `deny.toml`: Removed the stale `[[licenses.clarify]]` block for the `ring` crate.

## 🧪 Verification receipts
```text
$ cargo deny check licenses
licenses ok

$ cargo xtask docs --check
Documentation is up to date.

$ cargo xtask version-consistency
Checking version consistency against workspace version 1.9.0
  ✓ Cargo crate versions match 1.9.0.
  ✓ Cargo workspace dependency versions match 1.9.0.
  ✓ Node package manifest versions match 1.9.0.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.
```

## 🧭 Telemetry
- Change shape: Metadata cleanup
- Blast radius: Configuration only (no API, IO, docs, or schema impact)
- Risk class: Low risk (just removing unused permissions)
- Rollback: Revert the PR
- Gates run: `cargo deny check licenses`, `cargo xtask docs --check`, `cargo xtask version-consistency`

## 🗂️ .jules artifacts
- `.jules/runs/steward_1/envelope.json`
- `.jules/runs/steward_1/decision.md`
- `.jules/runs/steward_1/receipts.jsonl`
- `.jules/runs/steward_1/result.json`
- `.jules/runs/steward_1/pr_body.md`

## 🔜 Follow-ups
None.
