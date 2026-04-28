## 💡 Summary
Added missing CLI help markers to `docs/reference-cli.md` and synced the document using `cargo xtask docs --update`. This fixes factual drift between the reference documentation and the actual CLI implementation.

## 🎯 Why
The `docs/reference-cli.md` file was missing `<!-- HELP: <command> -->` tags for several commands: `diff`, `context`, `check-ignore`, `tools`, `baseline`, `handoff`. Because the tags were missing, `cargo xtask docs --check` failed and the documentation was outdated, which violates the `docs-executable` gate profile requiring docs and schemas to match CLI behavior.

## 🔎 Evidence
The `cargo xtask docs --check` command failed initially:
```text
Error: Documentation drift detected in /app/docs/reference-cli.md. Run `cargo xtask docs --update` to fix.
```

## 🧭 Options considered
### Option A (recommended)
- Programmatically add the missing `<!-- HELP: <cmd> -->` tags and run `cargo xtask docs --update`.
- Fits this repo and shard by ensuring documentation syncs mechanically with code.
- Trade-offs: Structure is minimal, Velocity is high, Governance is enforced via automated tooling.

### Option B
- Manually write out the help text into the markdown document.
- Slower, error-prone, and likely to drift again immediately.
- Trade-offs: Negative velocity and poor governance.

## ✅ Decision
Chose Option A. By placing the correct marker pairs and regenerating the file, the workspace tool automatically injects the latest text and passes the `--check` gate.

## 🧱 Changes made (SRP)
- `docs/reference-cli.md`

## 🧪 Verification receipts
```text
$ cargo xtask docs --check
Error: Documentation drift detected in /app/docs/reference-cli.md. Run `cargo xtask docs --update` to fix.

$ cargo xtask docs --update
Updated /app/docs/reference-cli.md

$ cargo xtask docs --check
Documentation is up to date.

$ cargo fmt -- --check
$ cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Docs update
- Blast radius: `docs/reference-cli.md` only. No API, IO, or schema impact.
- Risk class: Low + why: Only touches markdown text documentation via established xtask mechanisms.
- Rollback: Revert markdown changes.
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy`

## 🗂️ .jules artifacts
- `.jules/runs/c346971f-d18f-42fe-b66b-eed1583c8dc0/envelope.json`
- `.jules/runs/c346971f-d18f-42fe-b66b-eed1583c8dc0/decision.md`
- `.jules/runs/c346971f-d18f-42fe-b66b-eed1583c8dc0/receipts.jsonl`
- `.jules/runs/c346971f-d18f-42fe-b66b-eed1583c8dc0/result.json`
- `.jules/runs/c346971f-d18f-42fe-b66b-eed1583c8dc0/pr_body.md`

## 🔜 Follow-ups
None.
