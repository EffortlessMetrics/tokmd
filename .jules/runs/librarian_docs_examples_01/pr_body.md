## 💡 Summary
Fixed executable example commands in `docs/debugging.md` that drifted from actual CLI parsing rules. The debugging instructions now use valid positional arguments and current output flags (`--output-dir`, `--artifacts-dir`).

## 🎯 Why
The receipt debugging guide instructed maintainers to run commands using removed flags like `--path` and `--out`. Running these commands exactly as written caused `clap` parsing failures, blocking local troubleshooting workflows and creating friction during evidence review.

## 🔎 Evidence
- `docs/debugging.md` contained `cargo run -p tokmd -- run --path . --out target/tokmd-debug`.
- The current CLI definitions (verified via `cargo run -p tokmd -- run --help`) expect paths as positional inputs `[PATH]...` and outputs via `--output-dir`.
- `cargo run -p tokmd -- cockpit --help` expects `--artifacts-dir` instead of `--out`.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `docs/debugging.md` to reflect the correct CLI schema.
- why it fits this repo and shard: Directly targets example/docs drift for the librarian persona in the tooling-governance shard.
- trade-offs:
  - Structure: Aligns docs with CLI structs.
  - Velocity: Quick, direct documentation fix.
  - Governance: Removes maintainer friction when debugging PR evidence.

### Option B
- what it is: Produce a learning PR proposing a friction item without fixing the text.
- when to choose it instead: If the CLI syntax was in the middle of an active redesign.
- trade-offs: Misses the opportunity to fix clear factual drift in executable examples.

## ✅ Decision
Option A. Fixing executable docs drift directly addresses the Librarian's top ranking goal and prevents developers from copy-pasting failing commands.

## 🧱 Changes made (SRP)
- `docs/debugging.md`: Updated arguments for `tokmd run`, `tokmd analyze`, and `tokmd cockpit` receipt debugging examples.

## 🧪 Verification receipts
```text
cargo xtask docs --check
Documentation is up to date.
doc artifacts ok: 2 required doc(s), 73 family file(s), 1 active goal(s), 26 spec-index artifact(s), 0 spec-index lane(s)

cargo fmt -- --check

cargo clippy -- -D warnings
  Downloaded ...
  Compiling ...
  Finished `dev` profile [unoptimized + debuginfo] target(s)

bash -c 'CI=true cargo test -p tokmd --verbose'
...
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

## 🧭 Telemetry
- Change shape: Docs/Markdown only.
- Blast radius: `docs/`. No API, IO, or concurrency risk.
- Risk class: Low + docs-only change to executable instructions.
- Rollback: Revert the documentation commit.
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test -p tokmd --verbose`.

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples_01/envelope.json`
- `.jules/runs/librarian_docs_examples_01/decision.md`
- `.jules/runs/librarian_docs_examples_01/receipts.jsonl`
- `.jules/runs/librarian_docs_examples_01/result.json`
- `.jules/runs/librarian_docs_examples_01/pr_body.md`

## 🔜 Follow-ups
None.
