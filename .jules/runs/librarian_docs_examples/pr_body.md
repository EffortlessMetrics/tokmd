## 💡 Summary
Fixed documentation drift in `docs/reference-cli.md` where the manually maintained argument tables were missing recently added CLI options. The tables for `tokmd diff`, `tokmd module`, `tokmd export`, and `tokmd cockpit` have been updated to reflect the true `--help` outputs.

## 🎯 Why
`cargo xtask docs --check` only verifies that the embedded `<!-- HELP: cmd -->` output blocks match the CLI exactly. However, the manually written argument tables immediately preceding these blocks had silently drifted out of sync, missing flags like `--exclude`, `--profile`, and `--no-progress`. This fixes the drift so users don't see conflicting options.

## 🔎 Evidence
- `docs/reference-cli.md`
- `tokmd diff --help` output shows `--exclude <PATTERN>` but the manual table did not include it.
- Replaced the outdated tables with the true flag structures.

## 🧭 Options considered
### Option A (recommended)
- Update the manual markdown tables to mirror the true `--help` output while keeping their descriptive value.
- Fits the `tooling-governance` shard and the `Librarian` persona by explicitly addressing factual documentation drift.
- Trade-offs: Requires continued maintenance, but preserves the high-quality layout.

### Option B
- Delete the manual tables entirely and rely solely on the generated `--help` outputs in the docs.
- When to choose: If drift happens constantly and maintenance becomes too high.
- Trade-offs: Lowers the quality of the reading experience.

## ✅ Decision
Option A was chosen to fix the factual drift while preserving the high-quality, readable format of the reference CLI documentation.

## 🧱 Changes made (SRP)
- `docs/reference-cli.md`: Updated argument tables for `diff`, `module`, `export`, and `cockpit` to match the exact CLI help text.

## 🧪 Verification receipts
```text
cargo xtask docs --check
Documentation is up to date.
```

## 🧭 Telemetry
- Change shape: Docs update
- Blast radius: API (None), IO (None), docs (Medium), schema (None)
- Risk class: Low
- Rollback: `git checkout docs/reference-cli.md`
- Gates run: `cargo xtask docs --check`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples/envelope.json`
- `.jules/runs/librarian_docs_examples/decision.md`
- `.jules/runs/librarian_docs_examples/receipts.jsonl`
- `.jules/runs/librarian_docs_examples/result.json`
- `.jules/runs/librarian_docs_examples/pr_body.md`

## 🔜 Follow-ups
None.
