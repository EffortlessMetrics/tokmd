---
# PR Glass Cockpit

Make review boring. Make truth cheap.

## 💡 Summary
Unified the wording for the `--children` argument across subcommands in the CLI help for improved consistency.

## 🎯 Why (user/dev pain)
Previously, the `--children` argument had slightly varying descriptions depending on whether it was viewed in `tokmd module` ("Whether to include embedded languages..."), `tokmd export`, or the default `tokmd lang` command ("How to handle embedded languages..."). Unifying the text clarifies intent across contexts.

## 🔎 Evidence (before/after)
- **Before**: `tokmd module --help` displayed: `Whether to include embedded languages (tokei "children" / blobs) in module totals [default: separate].`
- **After**: `tokmd module --help` displays: `How to handle embedded languages (tokei "children" / blobs) in module totals [default: separate].`
- File paths: `crates/tokmd-config/src/lib.rs`

## 🧭 Options considered
### Option A (recommended)
- Unify around the more descriptive phrase: "How to handle embedded languages...".
- Why it fits this repo: Consistently explains the action ("How to handle") rather than implying a yes/no inclusion ("Whether to include") when multiple enums (`separate`, `parents-only`) exist.
- Trade-offs: Requires a minor snaptest update.

### Option B
- No-op. Leave the discrepancies in the CLI.

## ✅ Decision
Option A. It's a small DX improvement with minimal risk.

## 🧱 Changes made (SRP)
- `crates/tokmd-config/src/lib.rs`: Updated docstrings for `CliModuleArgs` and `CliExportArgs`.
- `crates/tokmd/tests/snapshots/cli_snapshot_golden__help.snap`: Updated auto-generated CLI help.

## 🧪 Verification receipts
- `cargo build --verbose`: PASS (Finished `dev` profile)
- `cargo test -p tokmd-config`: PASS (test result: ok. 15 passed; 0 failed)
- `INSTA_UPDATE=always cargo test -p tokmd --test cli_snapshot_golden`: PASS (test result: ok. 12 passed; 0 failed)
- `cargo fmt -- --check && cargo clippy -- -D warnings`: PASS (Finished `dev` profile)

## 🧭 Telemetry
- Change shape: Small documentation / snapshot update
- Blast radius: CLI documentation
- Risk class: Low (no runtime logic changed)
- Rollback: `git revert`
- Merge-confidence gates: `build`, `test`, `fmt`, `clippy`

## 🗂️ .jules updates
- Bootstrapped `.jules/policy/scheduled_tasks.json`, `.jules/runbooks/*`, and `.jules/palette/*`.
- Added run envelope: `.jules/palette/envelopes/run-001.json`
- Initialized `.jules/palette/ledger.json`

## 📝 Notes (freeform)
N/A

## 🔜 Follow-ups
N/A
---
