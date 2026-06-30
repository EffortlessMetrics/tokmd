## 💡 Summary
Aligned the `--children` flag defaults documentation in `docs/reference-cli.md` with the actual CLI behavior.

## 🎯 Why
The documentation previously stated that the `--children` flag defaults to `collapse` for all commands. However, `tokmd module` and `tokmd export` default to `separate`, while only `tokmd lang` defaults to `collapse`. This factual drift could mislead users configuring their scans and cause unexpected embedded language handling.

## 🔎 Evidence
- **File path**: `docs/reference-cli.md`
- **Observed behavior**: The docs incorrectly stated a universal `collapse` default. Running `tokmd module --help` and `tokmd export --help` shows `[default: separate]`, while `tokmd lang --help` shows `[default: collapse]`.

## 🧭 Options considered
### Option A (recommended)
- Update `docs/reference-cli.md` to explicitly note that the default is `collapse` for `lang` and `separate` for `module` and `export`.
- fits this repo and shard as it improves factual docs quality without risky runtime changes.
- trade-offs: Structure / Velocity / Governance - Zero runtime risk, immediately improves docs accuracy.

### Option B
- Modify the CLI to use `collapse` as the default for all subcommands.
- Choose when unified defaults are prioritized over existing subcommand-specific accounting logic.
- trade-offs: Would be a breaking change to existing `module` and `export` behavior and violates the Librarian persona's constraints against changing runtime behavior.

## ✅ Decision
Chose Option A to correct the factual documentation drift without introducing runtime regressions.

## 🧱 Changes made (SRP)
- `docs/reference-cli.md`: Updated the `--children` flag documentation table entry to reflect subcommand-specific defaults.

## 🧪 Verification receipts
```text
cargo xtask docs --update
cargo xtask docs --check
cargo xtask check-file-policy --strict
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test --verbose
```

## 🧭 Telemetry
- Change shape: Docs update
- Blast radius: Docs only
- Risk class: Low
- Rollback: Revert docs commit
- Gates run: `cargo xtask docs --check`, `cargo xtask check-file-policy --strict`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test --verbose`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples/envelope.json`
- `.jules/runs/librarian_docs_examples/receipts.jsonl`
- `.jules/runs/librarian_docs_examples/decision.md`
- `.jules/runs/librarian_docs_examples/result.json`
- `.jules/runs/librarian_docs_examples/pr_body.md`

## 🔜 Follow-ups
None.
