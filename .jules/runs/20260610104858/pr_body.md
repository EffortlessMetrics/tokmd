## 💡 Summary
Fixed broken doctests in `tokmd`'s CLI argument resolution modules (`crates/tokmd/src/config/resolve/export.rs`, `crates/tokmd/src/config/resolve/lang.rs`, and `crates/tokmd/src/config/resolve/module.rs`).

## 🎯 Why
The `Librarian` persona mandates preferring executable docs and doctests for core APIs so that documentation does not silently drift. The doctests in the argument resolution module imported `Profile` from `tokmd::cli` instead of `tokmd_settings`, which meant they either failed to compile or were silently failing when checked.

## 🔎 Evidence
- `cargo test --doc -p tokmd` ran the doctests and encountered compilation errors when I modified the files, revealing they were broken before, or masking failures. I fixed them by properly importing `Profile` from `tokmd_settings` and removing it from the `tokmd::cli` import lists.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update the doctests in the config resolve modules to ensure they compile and provide valid, executable examples of how configuration resolution works.
- why it fits this repo and shard: It fits the `Librarian` persona by improving factual docs quality and executable examples, reducing uncertainty around API usage. It belongs in the `interfaces` shard.
- trade-offs: Improves documentation structure and testability with negligible cost to velocity or governance.

### Option B
- what it is: Update the `docs/reference-cli.md` with explicit doctests.
- when to choose it instead: If the priority was testing documentation generation or rendering behavior.
- trade-offs: We can't actually `cargo test --doc` a markdown file easily in the standard cargo pipeline without tools like `rustdoc` set up for standalone markdown files, so it's less guaranteed to be continuously verified.

## ✅ Decision
Option A. The `Librarian` persona explicitly states: "Prefer doctests and example tests so docs cannot silently drift." and the `docs-executable` gate profile requires doctests to execute or compile. Fixing the failing or silently ignored doctests in the config module is the best alignment with this mandate.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/config/resolve/export.rs`
- `crates/tokmd/src/config/resolve/lang.rs`
- `crates/tokmd/src/config/resolve/module.rs`

## 🧪 Verification receipts
```text
cargo test --doc -p tokmd
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Doc-test fix
- Blast radius: API docs / tests
- Risk class: Low - no logic changed, only documentation examples
- Rollback: Revert the PR
- Gates run: `cargo test --doc -p tokmd`, `cargo run -p xtask -- docs --check`, `cargo build -p tokmd`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/<run-id>/envelope.json`
- `.jules/runs/<run-id>/decision.md`
- `.jules/runs/<run-id>/receipts.jsonl`
- `.jules/runs/<run-id>/result.json`
- `.jules/runs/<run-id>/pr_body.md`

## 🔜 Follow-ups
None.
