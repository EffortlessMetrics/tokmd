## 💡 Summary
This is a learning PR. The attempt to replace `localeCompare` in the HTML analysis report template was explicitly superseded by PR #1601.

## 🎯 Why
A PR review comment noted: "Superseded by #1601, which replaced the analysis HTML localeCompare path with an explicit Unicode code point comparator and refreshed the current snapshots." We are falling back to a learning PR to record this friction.

## 🔎 Evidence
- Pull request comment: "Superseded by #1601, which replaced the analysis HTML localeCompare path with an explicit Unicode code point comparator and refreshed the current snapshots."

## 🧭 Options considered
### Option A (recommended)
- Record a friction item about the target PR already existing and being closed as superseded.
- **Why it fits this repo and shard**: Respects reality and doesn't duplicate work or try to land a patch that was explicitly obsoleted by another PR (#1601).
- **Trade-offs**:
  - Structure: Leaves the repo unmodified as intended by the maintainer.
  - Velocity: Fast off-ramp.
  - Governance: Complies with the prompt directive to produce a learning PR if a patch is not justified.

### Option B
- Re-create the PR.
- **When to choose it instead**: If the maintainer was incorrect about it being superseded.
- **Trade-offs**: Hallucinates a fix that already landed in another branch, creating noise.

## ✅ Decision
Option A was chosen. A learning PR will be recorded.

## 🧱 Changes made (SRP)
- None.

## 🧪 Verification receipts
```text
cargo test -p tokmd-format --no-default-features
cargo test -p tokmd-format --all-features
npm --prefix web/runner test
cargo fmt -- --check
cargo clippy -p tokmd-format -- -D warnings
```

## 🧭 Telemetry
- Change shape: None.
- Blast radius: None.
- Risk class: None.
- Rollback: None.
- Gates run: None.

## 🗂️ .jules artifacts
- `.jules/runs/compat_targets_matrix/envelope.json`
- `.jules/runs/compat_targets_matrix/decision.md`
- `.jules/runs/compat_targets_matrix/receipts.jsonl`
- `.jules/runs/compat_targets_matrix/result.json`
- `.jules/runs/compat_targets_matrix/pr_body.md`
- Added friction item: `.jules/friction/open/compat_superseded_1601.md`

## 🔜 Follow-ups
None.
