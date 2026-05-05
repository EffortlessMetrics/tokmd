## 💡 Summary
Added the missing `--profile` CLI flag to the "Global Arguments" section of `docs/reference-cli.md`. This resolves factual drift between the parser definition (`Cli` struct) and the human-readable schema.

## 🎯 Why
The `--profile` flag exists as a global argument in `crates/tokmd/src/cli/parser.rs` and shows up in the CLI's `--help` output. However, it was completely missing from the handwritten global arguments table in `docs/reference-cli.md`, creating a mismatch between the documented API and the actual executable contract.

## 🔎 Evidence
- `docs/reference-cli.md`: Missing `--profile` in the "Global Arguments" table.
- `crates/tokmd/src/cli/parser.rs`: Defines `--profile` as a global argument.
- `cargo run --bin tokmd -- --help` outputs `--profile <PROFILE>`.

## 🧭 Options considered
### Option A (recommended)
- Add the `--profile` row to the markdown table in `docs/reference-cli.md`.
- Why it fits this repo and shard: It directly fixes factual documentation drift for contract-bearing output surfaces without making sweeping refactors.
- Trade-offs: Structure is minimal, velocity is high, governance guarantees correct documentation.

### Option B
- Refactor `xtask docs --check` to dynamically map and check all global flag Rust struct definitions against the markdown file.
- When to choose it instead: If global arguments change rapidly and manual matching becomes untenable.
- Trade-offs: Requires a heavier build logic and parse capability inside xtask.

## ✅ Decision
Selected Option A to immediately lock in the missing documentation and satisfy the single prompt-to-PR objective without adding complex AST parsing overhead to the xtask suite.

## 🧱 Changes made (SRP)
- `docs/reference-cli.md` - Added `--profile <PROFILE>` and its aliases to the "Global Arguments" table.

## 🧪 Verification receipts
```text
$ sed -i 's/| `--no-progress` | Disable progress spinners (useful for CI\/non-TTY). |/| `--no-progress` | Disable progress spinners (useful for CI\/non-TTY). |\n| `--profile <PROFILE>` | Configuration profile to use (e.g., "llm_safe", "ci"). Aliases: `--view`. |/' docs/reference-cli.md
$ cargo xtask docs --check
Documentation is up to date.
```

## 🧭 Telemetry
- Change shape: Documentation Add
- Blast radius: CLI documentation surface only
- Risk class: Safe / Trivial. The change only impacts documentation and does not touch functional logic.
- Rollback: Revert the file addition.
- Gates run: `cargo xtask docs --check` and `cargo test -p xtask`.

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
Consider adding a more robust xtask static check to ensure Rust structs (like `GlobalArgs`) directly drive the `Global Arguments` table section without drifting out of sync.
