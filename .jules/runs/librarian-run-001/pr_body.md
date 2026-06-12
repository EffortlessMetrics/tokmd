## 💡 Summary
Fixed silent documentation drift in `docs/reference-cli.md` by explicitly registering the missing `syntax` and `evidence-packet` command help markers.

## 🎯 Why
The `tokmd` CLI currently includes `syntax` and `evidence-packet` commands, and their help markers exist in `docs/reference-cli.md`. However, they were not registered in the `markers` array in `xtask/src/tasks/docs.rs`. This omission caused `cargo xtask docs --check` to ignore them, leading to silent reference documentation drift when those CLI contracts changed.

## 🔎 Evidence
- `docs/reference-cli.md` contains `<!-- HELP: syntax -->` and `<!-- HELP: evidence-packet -->` markers.
- `tokmd --help` exposes both `syntax` and `evidence-packet` commands.
- `xtask/src/tasks/docs.rs` was missing these commands in the `markers` array.
- Running `cargo xtask docs --check` before the fix falsely passed without validating these sections.

## 🧭 Options considered
### Option A (recommended)
- Add `("syntax", "syntax")` and `("evidence-packet", "evidence-packet")` to the `markers` array in `xtask/src/tasks/docs.rs`.
- Fits the repo and shard by directly addressing the tooling governance mechanism that generates reference docs.
- Trade-offs: Structure/Governance - ensures CLI and reference-cli.md stay in sync for these commands, preventing silent drift. Velocity - low effort, high impact.

### Option B
- Wait for someone else to notice the drift when those commands change.
- When to choose: If we were outside the `tooling-governance` shard.
- Trade-offs: Fails the mission.

## ✅ Decision
Option A. It ensures that the `syntax` and `evidence-packet` commands are checked for drift by `cargo xtask docs --check` and automatically updated by `cargo xtask docs --update`, matching the rest of the CLI.

## 🧱 Changes made (SRP)
- `xtask/src/tasks/docs.rs`: Registered `syntax` and `evidence-packet` in the `markers` array.
- `docs/reference-cli.md`: Updated via `cargo xtask docs --update`.

## 🧪 Verification receipts
```text
cargo xtask docs --update
cargo xtask docs --check
cargo test --doc
```

## 🧭 Telemetry
- Change shape: Pure tooling fix
- Blast radius: `docs` and `xtask` tooling surfaces only. No runtime API impact.
- Risk class + why: Low risk. Fixes a documentation generation omission.
- Rollback: Revert the PR.
- Gates run: `docs-executable`

## 🗂️ .jules artifacts
- `.jules/runs/librarian-run-001/envelope.json`
- `.jules/runs/librarian-run-001/decision.md`
- `.jules/runs/librarian-run-001/receipts.jsonl`
- `.jules/runs/librarian-run-001/result.json`
- `.jules/runs/librarian-run-001/pr_body.md`

## 🔜 Follow-ups
None.
