## 💡 Summary
Updated the `tokmd` CLI reference documentation in `docs/reference-cli.md` to accurately reflect the `--children` flag options for different subcommands.

## 🎯 Why
The global CLI reference table documented `--children` with only `collapse` and `separate` options. However, `tokmd export` and `tokmd module` subcommands actually accept `separate` and `parents-only`. This created a mismatch between the documentation and the actual CLI schema.

## 🔎 Evidence
- `tokmd module --help` output shows: `--children <CHILDREN> ... [default: separate] Possible values: - separate ... - parents-only ...`
- The `docs/reference-cli.md` file previously listed only `collapse` and `separate` in the global arguments table and the `tokmd lang` table.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `docs/reference-cli.md` to clarify that `--children` options vary by subcommand, and update the `lang` specific table to reflect its valid options (`collapse`, `separate`).
- why it fits this repo and shard: This directly addresses docs/schema drift within the `tooling-governance` shard.
- trade-offs:
  - Structure: Improves accuracy of CLI documentation.
  - Velocity: Fast to verify and land.
  - Governance: Ensures the reference-cli docs match the actual CLI clap schema.

### Option B
- what it is: Update the CLI to use a single `ChildrenMode` enum across all subcommands.
- when to choose it instead: If the goal was to unify CLI flag behavior rather than just documenting the current state.
- trade-offs: Would require runtime behavior changes, violating the Librarian persona rules to not touch runtime behavior changes unless explicitly required.

## ✅ Decision
Option A. The `docs/reference-cli.md` has clear drift around the `--children` flag variants depending on the subcommand (`lang` vs `export`/`module`). I updated the tables to clarify the options.

## 🧱 Changes made (SRP)
- `docs/reference-cli.md`: Updated the global arguments table to note that `--children` options vary by subcommand (adding `parents-only` to the list of examples). Updated the `tokmd lang` specific table to explicitly list its supported modes (`collapse`, `separate`).

## 🧪 Verification receipts
```text
cargo xtask docs --check
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test -p xtask
```

## 🧭 Telemetry
- Change shape: Documentation update
- Blast radius: docs
- Risk class: Low - documentation changes only
- Rollback: Revert the PR
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test -p xtask`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples/envelope.json`
- `.jules/runs/librarian_docs_examples/decision.md`
- `.jules/runs/librarian_docs_examples/receipts.jsonl`
- `.jules/runs/librarian_docs_examples/result.json`
- `.jules/runs/librarian_docs_examples/pr_body.md`

## 🔜 Follow-ups
None.
