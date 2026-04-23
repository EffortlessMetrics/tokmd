## 💡 Summary
Removed the unused `pyo3-build-config` build dependency from the `tokmd-python` crate manifest.

## 🎯 Why
The `tokmd-python` bindings crate does not have a `build.rs` script, rendering the `pyo3-build-config` in the `[build-dependencies]` block entirely unused. Removing it improves dependency hygiene and eliminates a minor download/compilation step in that crate's context.

## 🔎 Evidence
- `crates/tokmd-python/Cargo.toml` previously had a `[build-dependencies]` section.
- `ls -la crates/tokmd-python/build.rs` returns `ls: cannot access 'crates/tokmd-python/build.rs': No such file or directory`.

## 🧭 Options considered
### Option A (recommended)
- Remove `pyo3-build-config` and the `[build-dependencies]` section from `crates/tokmd-python/Cargo.toml`.
- **Why it fits:** It's a high-signal dependency hygiene improvement for the bindings-targets shard, matching Auditor's primary goal.
- **Trade-offs:** Structure: Improves manifest accuracy. Velocity: Slightly reduces dependency overhead. Governance: Aligns with unused dependency removal expectations.

### Option B
- Document the build dependency or leave it for future use.
- **When to choose:** If there was an active, imminent PR intending to add a build script.
- **Trade-offs:** Retains unnecessary manifest bloat and unneeded dependencies in the resolution tree.

## ✅ Decision
Option A was chosen because it's a verifiable, boring, and clean removal of an unneeded build dependency as no `build.rs` exists.

## 🧱 Changes made (SRP)
- `crates/tokmd-python/Cargo.toml`: Removed the `[build-dependencies]` section and the `pyo3-build-config` declaration.

## 🧪 Verification receipts
```text
$ cargo build -p tokmd-python
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 15s

$ cargo test -p tokmd-python
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo clippy -p tokmd-python -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 37.79s
```

## 🧭 Telemetry
- Change shape: Removal
- Blast radius: `tokmd-python` crate manifest only. No API or compatibility changes.
- Risk class: Low - build logic verification prevents compilation errors.
- Rollback: Revert the commit.
- Gates run: `cargo build -p tokmd-python`, `cargo test -p tokmd-python`, `cargo clippy`, `cargo fmt`, `cargo deny` (noted missing).

## 🗂️ .jules artifacts
- `.jules/runs/auditor-bindings/envelope.json`
- `.jules/runs/auditor-bindings/decision.md`
- `.jules/runs/auditor-bindings/receipts.jsonl`
- `.jules/runs/auditor-bindings/result.json`
- `.jules/runs/auditor-bindings/pr_body.md`

## 🔜 Follow-ups
None required for this specific crate, though other binding crates could be evaluated similarly if more time allowed.

## 🗂️ Friction items added
- `.jules/friction/open/cargo_deny_missing.md`: noted missing `cargo-deny` tool in CI environment.
