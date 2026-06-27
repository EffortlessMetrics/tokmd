## 💡 Summary
Fixed documentation version drift to match the 1.14.0 release. Updated multiple markdown files that incorrectly used `1.13.1` in examples.

## 🎯 Why
To prevent user confusion and maintain version consistency across the codebase. Following the release of 1.14.0, documentation examples in GitHub Actions usage and packet workflows needed alignment with the new workspace version.

## 🔎 Evidence
Minimal proof:
- file paths: `docs/action-quickstart.md`, `docs/github-action.md`, `docs/packet-workflows.md`, `docs/evidence-packet.md`
- observed behavior / finding: The documentation examples still pointed to version `1.13.1`.
- command receipt demonstrating it: `grep -Rn "1.13.1" docs/action-quickstart.md docs/github-action.md docs/packet-workflows.md docs/evidence-packet.md`

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `1.13.1` to `1.14.0` in the identified documentation examples.
- why it fits this repo and shard: It is a low-risk release/governance improvement that directly addresses publish-plan/version-consistency drift.
- trade-offs: Structure is improved by ensuring examples use the latest version. No governance or velocity downside.

### Option B
- what it is: Update historical ledgers in `docs/releases/`.
- when to choose it instead: If the ledgers were actually incorrect for the historical release.
- trade-offs: This would introduce factual inaccuracies by invalidating the history of the 1.13.1 release.

## ✅ Decision
Chose Option A. It effectively aligns the current documentation with the latest 1.14.0 release while preserving historical accuracy.

## 🧱 Changes made (SRP)
- `docs/action-quickstart.md`
- `docs/github-action.md`
- `docs/packet-workflows.md`
- `docs/evidence-packet.md`

## 🧪 Verification receipts
```text
cargo xtask version-consistency
cargo xtask docs --check
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Documentation update.
- Blast radius: API / docs (No schema or codebase changes).
- Risk class + why: Low. Modifies documentation examples to use the correct tool version.
- Rollback: Revert the PR.
- Gates run: `cargo xtask version-consistency`, `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/steward_release/envelope.json`
- `.jules/runs/steward_release/decision.md`
- `.jules/runs/steward_release/receipts.jsonl`
- `.jules/runs/steward_release/result.json`
- `.jules/runs/steward_release/pr_body.md`

## 🔜 Follow-ups
None.
