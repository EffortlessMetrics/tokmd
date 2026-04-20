## 💡 Summary
Fix a typographical error in `docs/testing.md` where `tokmd sensor cockpit` was incorrectly referenced. It has been corrected to refer to the separate `tokmd sensor` and `tokmd cockpit` commands.

## 🎯 Why
The documentation claimed that the `tokmd sensor cockpit` command is tested by integration tests. However, `sensor` and `cockpit` are both top-level subcommands in `tokmd`, so `tokmd sensor cockpit` is not a valid command. Fixing this eliminates a factual error and prevents confusion.

## 🔎 Evidence
- File: `docs/testing.md`
- Command `cargo run --bin tokmd -- sensor --help` shows that `sensor` has options like `--base` but no subcommands like `cockpit`.
- Command `cargo run --bin tokmd -- cockpit --help` shows that `cockpit` is a top-level subcommand.

## 🧭 Options considered
### Option A (recommended)
- Correct the typo by separating `tokmd sensor` and `tokmd cockpit` commands in `docs/testing.md`.
- **Structure:** Keeps documentation factual and aligned with the actual CLI schema.
- **Velocity:** Small, trivial fix, safely applied.
- **Governance:** Keeps docs truth in sync with executable features.

### Option B
- Ignore the typo or rewrite the entire section.
- **When to choose:** Only if the integration tests themselves needed a rewrite.
- **Trade-offs:** Out of scope for this prompt since the typo is straightforward to resolve.

## ✅ Decision
Option A. The document claimed the `tokmd sensor cockpit` command is tested, but this command syntax is invalid. `tokmd sensor` and `tokmd cockpit` are separate CLI subcommands. Corrected the typo in `docs/testing.md` to reference `tokmd sensor` and `tokmd cockpit` commands accurately.

## 🧱 Changes made (SRP)
- `docs/testing.md`

## 🧪 Verification receipts
```text
{"cmd": "grep -rn \"sensor cockpit\" docs/", "output": "docs/testing.md:82:- Integration tests for `tokmd sensor cockpit` command"}
{"cmd": "sed -i 's/`tokmd sensor cockpit` command/`tokmd sensor` and `tokmd cockpit` commands/g' docs/testing.md", "output": ""}
{"cmd": "cargo xtask docs --check", "output": "Documentation is up to date."}
```

## 🧭 Telemetry
- **Change shape:** Docs typo fix
- **Blast radius:** `docs` only
- **Risk class:** Low (No code changes)
- **Rollback:** Safe to rollback
- **Gates run:** `cargo xtask docs --check`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples_01/envelope.json`
- `.jules/runs/librarian_docs_examples_01/decision.md`
- `.jules/runs/librarian_docs_examples_01/receipts.jsonl`
- `.jules/runs/librarian_docs_examples_01/result.json`
- `.jules/runs/librarian_docs_examples_01/pr_body.md`

## 🔜 Follow-ups
None.
