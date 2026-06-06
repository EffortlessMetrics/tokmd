## 💡 Summary
Removed the `git` feature flag from `tokmd-cockpit`, making `tokmd-git` a hard dependency. This removes dozens of `#[cfg(feature = "git")]` macros from the cockpit crate, significantly simplifying the code and dependency structure.

## 🎯 Why
`tokmd-cockpit` is built specifically for computing PR metrics and aggregating code review artifacts. Without `git`, its functionality is largely empty stubs, meaning there is no practical value to compiling `tokmd-cockpit` without `git` support. The artificial feature boundary just added noise to the source files and complexity to the workspace dependency graph.

## 🔎 Evidence
- `crates/tokmd-cockpit/Cargo.toml` defined `git = ["dep:tokmd-git"]`.
- `crates/tokmd-cockpit/src/` contained dozens of `#[cfg(feature = "git")]` statements gating almost all meaningful metrics logic.
- `grep -rnw "crates/tokmd-cockpit" -e "feature.*git"` confirmed the heavy proliferation of the feature flag.

## 🧭 Options considered
### Option A (recommended)
- Remove the `git` feature from `tokmd-cockpit`, making `tokmd-git` a regular required dependency.
- Removes `#[cfg(feature = "git")]` throughout `tokmd-cockpit`.
- Updates `tokmd` and `tokmd-core` to remove the feature forwarding.
- **Trade-offs:**
  - Structure: Simplifies Cockpit code immensely. Makes feature boundaries clearer.
  - Velocity: Future Cockpit work won't require stubbing out non-git paths.
  - Governance: Aligns the architectural seam (Cockpit is inherently built on Git).

### Option B
- Keep the `git` feature in `tokmd-cockpit` but try to clean up usages.
- **Trade-offs:** Does not fix the core architectural issue. Leaves an artificial feature boundary where none adds value.

## ✅ Decision
Option A. I removed the `git` feature, made `tokmd-git` a regular dependency, and stripped out the `#[cfg]` gating from `tokmd-cockpit`.

## 🧱 Changes made (SRP)
- `crates/tokmd-cockpit/Cargo.toml`: Removed `git` feature, made `tokmd-git` a standard dependency.
- `crates/tokmd/Cargo.toml`: Removed `tokmd-cockpit/git` feature forwarding.
- `crates/tokmd-core/Cargo.toml`: Removed `tokmd-cockpit` from `cockpit` feature dependencies.
- `crates/tokmd-cockpit/src/**/*.rs`: Removed `#[cfg(feature = "git")]` and changed `#[cfg(all(test, feature = "git"))]` to `#[cfg(test)]`.
- `crates/tokmd-cockpit/tests/**/*.rs`: Removed `#[cfg(feature = "git")]`.

## 🧪 Verification receipts
```text
{"timestamp": "2024-06-06T12:00:00Z", "command": "cargo check --workspace", "output": "success"}
{"timestamp": "2024-06-06T12:00:15Z", "command": "cargo test -p tokmd-cockpit", "output": "success"}
{"timestamp": "2024-06-06T12:01:30Z", "command": "cargo test -p tokmd", "output": "success"}
{"timestamp": "2024-06-06T12:05:00Z", "command": "cargo run -p xtask -- proof-policy", "output": "success"}
```

## 🧭 Telemetry
- Change shape: Structural refactor / boundary hygiene.
- Blast radius: `tokmd-cockpit`, `tokmd`, `tokmd-core`. Dependencies updated, internal feature flags removed. No public API behavior changes.
- Risk class: Low. Compile-time feature flags only. Covered by existing tests.
- Rollback: `git checkout crates/tokmd/Cargo.toml crates/tokmd-core/Cargo.toml crates/tokmd-cockpit`
- Gates run: `cargo check --workspace`, `cargo test -p tokmd-cockpit`, `cargo test -p tokmd`, `cargo run -p xtask -- proof-policy`

## 🗂️ .jules artifacts
- `.jules/runs/surveyor-001/envelope.json`
- `.jules/runs/surveyor-001/decision.md`
- `.jules/runs/surveyor-001/receipts.jsonl`
- `.jules/runs/surveyor-001/result.json`
- `.jules/runs/surveyor-001/pr_body.md`

## 🔜 Follow-ups
None.
