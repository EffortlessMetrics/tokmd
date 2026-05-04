## 💡 Summary
Cleaned up the `deny.toml` configuration by removing a stale ignore for `RUSTSEC-2023-0071` to silence a `warning[advisory-not-detected]` during CI gating.

## 🎯 Why
The `deny.toml` file had an ignore for `RUSTSEC-2023-0071` (from a transitive `rsa` test dependency). Since that dependency chain is no longer pulling in the vulnerable version, `cargo deny` correctly flagged the ignore rule as stale with a `warning[advisory-not-detected]`. This causes unnecessary noise during the `cargo deny` validation gate, so we're removing it to keep release metadata clean and our CI strict.

## 🔎 Evidence
- **File path:** `deny.toml`
- **Observed behavior:** Running `cargo deny --all-features check` emitted a `warning[advisory-not-detected]` indicating that the `RUSTSEC-2023-0071` advisory was not encountered in the dependency tree.

## 🧭 Options considered
### Option A (recommended)
- Remove the stale ignore block for `RUSTSEC-2023-0071` from `deny.toml`.
- **Why it fits:** Direct, low-risk fix that restores the `cargo deny` validation gate to a fully clean state.
- **Trade-offs:**
  - *Structure:* Cleans up outdated configuration.
  - *Velocity:* Marginally speeds up CI logic.
  - *Governance:* Aligns with strict governance standards by eliminating noisy false positives in security/license tools.

### Option B
- Ignore the warning.
- **When to choose it instead:** If the dependency was temporarily dropped but expected to return soon.
- **Trade-offs:** Leaves technical debt in the release gates, reducing the signal-to-noise ratio over time.

## ✅ Decision
Option A. It's a clear, honest patch that directly addresses an observable governance warning.

## 🧱 Changes made (SRP)
- `deny.toml`: Removed the ignore object for `RUSTSEC-2023-0071`.

## 🧪 Verification receipts
```text
$ cargo deny --all-features check
advisories ok, bans ok, licenses ok, sources ok
```

## 🧭 Telemetry
- **Change shape:** Configuration deletion.
- **Blast radius:** Low. Only affects the `cargo deny` advisory checking logic during CI.
- **Risk class:** Trivial. Does not affect runtime behavior or compilation.
- **Rollback:** `git revert`.
- **Gates run:** `cargo deny --all-features check`

## 🗂️ .jules artifacts
- `.jules/runs/steward_release/envelope.json`
- `.jules/runs/steward_release/decision.md`
- `.jules/runs/steward_release/receipts.jsonl`
- `.jules/runs/steward_release/result.json`
- `.jules/runs/steward_release/pr_body.md`

## 🔜 Follow-ups
None.
