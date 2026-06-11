## 💡 Summary
Added practical CLI examples for imported proof and doc-artifacts validation to `tokmd cockpit` help markers.

## 🎯 Why
The `Librarian` persona prioritizes missing executable coverage or factual docs drift. `ROADMAP.md` identifies "CLI help examples" as an active goal, specifically asking to "Add practical examples to command help for `analyze`, `diff`, `context`, `handoff`, and `cockpit`". While `tokmd cockpit` had examples for base usages, it lacked practical examples for `--proof-run-summary` and `--doc-artifacts-check` despite being key features of the command. This closes that documentation drift natively inside the code so `cargo xtask docs --update` handles it.

## 🔎 Evidence
- file path: `docs/reference-cli.md` and `crates/tokmd/src/cli/parser/cockpit.rs`
- observed behavior: `tokmd cockpit --help` examples only showed basic usages: `--base origin/main --head HEAD --format comment`.
- command receipt: `cargo xtask docs --update` was run after modification to `cockpit.rs` and the `<!-- HELP: cockpit -->` marker correctly incorporated the new examples.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update the `after_help` string inside `crates/tokmd/src/cli/parser/cockpit.rs` and run `cargo xtask docs --update`.
- why it fits this repo and shard: It implements the `Librarian` persona's objective to add practical examples to command help for `tokmd cockpit` (per `ROADMAP.md`).
- trade-offs: Structure / Velocity / Governance: Fixes docs at the Rust code source rather than directly mutating the markdown file, aligning with the `docs-executable` gate.

### Option B
- what it is: Add a rust doctest to `tokmd_core::cockpit_workflow` directly.
- when to choose it instead: If the goal was to provide an API-level example in Rust code.
- trade-offs: `tokmd_core::workflows::cockpit_workflow` already has a rust doctest, so adding another would be redundant and wouldn't solve the CLI drift issue.

## ✅ Decision
Option A. I modified the `after_help` string in `crates/tokmd/src/cli/parser/cockpit.rs` to include practical examples for proof-run-summary and doc-artifacts-check, then regenerated the markdown docs.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/cli/parser/cockpit.rs`
- `docs/reference-cli.md`

## 🧪 Verification receipts
```text
$ cargo xtask docs --update
Updated /app/docs/reference-cli.md
     Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.63s
     Running `target/debug/xtask docs --update`

$ cargo xtask docs --check
Documentation is up to date.
doc artifacts ok: 2 required doc(s), 54 family file(s), 1 active goal(s), 19 spec-index artifact(s), 0 spec-index lane(s)

$ cargo test --doc -p tokmd
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## 🧭 Telemetry
- Change shape: Docs update
- Blast radius: Docs and CLI help text
- Risk class: Low, documentation changes only.
- Rollback: `git restore docs/reference-cli.md crates/tokmd/src/cli/parser/cockpit.rs`
- Gates run: `cargo xtask docs --update`, `cargo xtask docs --check`, `CI=true cargo test -p tokmd --verbose`, `cargo test --doc -p tokmd`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/run-librarian-01/envelope.json`
- `.jules/runs/run-librarian-01/decision.md`
- `.jules/runs/run-librarian-01/receipts.jsonl`
- `.jules/runs/run-librarian-01/result.json`
- `.jules/runs/run-librarian-01/pr_body.md`

## 🔜 Follow-ups
None.
