## 💡 Summary
Added usage examples as Rust doctests for the four main workflow APIs in `tokmd-core`. This helps library consumers understand how to embed the `lang`, `module`, `export`, `diff`, and `analyze` engines directly.

## 🎯 Why / Threat model
The core library facade lacked executable documentation for its highest-traffic methods. Writing them as doctests ensures they can never silently drift out of sync with the actual API surface.

## 🔎 Finding (evidence)
- `crates/tokmd-core/src/lib.rs` lacked `/// ```rust` blocks for its public `*_workflow` functions.

## 🧭 Options considered
### Option A (recommended)
- Add standard `#[test]` unit tests that happen to be readable.
- This is fine, but doesn't show up in `rustdoc` or IDE hover cards.

### Option B
- Write inline `/// ```rust` doctests on the public functions.
- Why it fits: The `tokmd-core` crate is a library intended for embedding, and users will look directly at the Rustdoc for these functions.
- Trade-offs: Doctests run sequentially by default, but these are small and fast.

## ✅ Decision
Option B. We want the examples visible directly on the trait/function definitions.

## 🧱 Changes made (SRP)
- Added doctests to `module_workflow` in `crates/tokmd-core/src/lib.rs`.
- Added doctests to `export_workflow` in `crates/tokmd-core/src/lib.rs`.
- Added doctests to `diff_workflow` in `crates/tokmd-core/src/lib.rs`.
- Added doctests to `analyze_workflow` in `crates/tokmd-core/src/lib.rs`.

## 🧪 Verification receipts
```
cargo test -p tokmd-core --doc --all-features
```

## 🧭 Telemetry
- Change shape: Documentation additions.
- Blast radius: Rustdoc and `cargo test --doc` execution.
- Risk class: Very low. Only comments were touched.
- Rollback: Revert the PR.
- Merge-confidence gates: `cargo build`, `cargo fmt`, `cargo clippy`, `cargo test -p tokmd-core`.

## 🗂️ .jules updates
- Wrote run envelope to `.jules/docs/envelopes/`.
- Appended run ID to `.jules/docs/ledger.json`.

## 📝 Notes (freeform)
All doctests are self-contained and use the `current_dir()` defaults to avoid needing test fixtures.
