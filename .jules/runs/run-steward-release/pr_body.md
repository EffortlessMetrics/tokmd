## 💡 Summary
Removed a stale ignored advisory (`RUSTSEC-2023-0071`) from `deny.toml`. This resolves a warning during `cargo deny check` about an advisory that is no longer encountered in the dependency tree.

## 🎯 Why
Running `cargo deny --all-features check` resulted in a warning: `warning[advisory-not-detected]: advisory was not encountered` pointing to the exemption for `RUSTSEC-2023-0071` (the `rsa` crate brought in via `uselesskey`). The dependency tree no longer requires this exemption, so keeping it adds noise to CI logs and reduces confidence in our security gating. Cleaning this up aligns with our release and governance hygiene goals.

## 🔎 Evidence
- **File:** `deny.toml`
- **Observed behavior:** `cargo deny` outputs a warning for `RUSTSEC-2023-0071` because the crate it applies to is no longer in the dependency graph.
- **Command receipt:** Running `cargo deny --all-features check` produced `warning[advisory-not-detected]: advisory was not encountered` for `RUSTSEC-2023-0071`.

## 🧭 Options considered
### Option A (recommended)
- What it is: Remove the stale advisory exemption from `deny.toml`.
- Why it fits: Aligns with the Steward persona and tooling-governance shard by improving release metadata and gating hygiene.
- Trade-offs:
  - Structure: Cleans up security policy config.
  - Velocity: Reduces warning noise in build logs.
  - Governance: Ensures the advisory exemption list is perfectly accurate to current reality.

### Option B
- What it is: Do nothing and create a learning PR.
- When to choose: If the warning was legitimate and the dependency was still present but hard to patch.
- Trade-offs: Fails to improve the repository hygiene and leaves a known warning.

## ✅ Decision
Option A was chosen to remove the warning noise and improve gating hygiene.

## 🧱 Changes made (SRP)
- Modified `deny.toml` to remove the ignored advisory for `RUSTSEC-2023-0071` and its associated comment.

## 🧪 Verification receipts
```text
$ cargo xtask version-consistency
Checking version consistency against workspace version 1.10.0

  ✓ Cargo crate versions match 1.10.0.
  ✓ Cargo workspace dependency versions match 1.10.0.
  ✓ Node package manifest versions match 1.10.0.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.

$ cargo xtask docs --check
Documentation is up to date.

$ cargo xtask publish --plan --verbose
=== Publish Plan ===
...
To execute this plan:
  cargo xtask publish --yes --verbose

$ cargo deny --all-features check
advisories ok, bans ok, licenses ok, sources ok
```

## 🧭 Telemetry
- Change shape: Metadata/configuration update
- Blast radius: Configuration only (`deny.toml`)
- Risk class: Low risk. This only affects the `cargo deny` linter configuration.
- Rollback: Revert the PR.
- Gates run: `cargo xtask version-consistency`, `cargo xtask docs --check`, `cargo xtask publish --plan --verbose`, `cargo deny --all-features check`.

## 🗂️ .jules artifacts
- `.jules/runs/run-steward-release/envelope.json`
- `.jules/runs/run-steward-release/decision.md`
- `.jules/runs/run-steward-release/receipts.jsonl`
- `.jules/runs/run-steward-release/result.json`
- `.jules/runs/run-steward-release/pr_body.md`

## 🔜 Follow-ups
None.
