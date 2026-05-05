## 💡 Summary
Replaced platform-dependent `localeCompare` sorting in the generated HTML analysis report template with explicit strict Unicode code unit comparison. This guarantees deterministic lexicographical sorting behavior across different browsers and matches Rust's native `String::cmp`.

## 🎯 Why
In JS/Node/Browser environments, `String.prototype.localeCompare()` is platform and locale-dependent, causing sorting drift for strings in HTML reports compared to Rust's native deterministic order. Memory constraints require ensuring strict Unicode code-unit comparisons (`a < b ? -1 : a > b ? 1 : 0`) to lock in determinism.

## 🔎 Evidence
- `crates/tokmd-format/src/analysis/templates/report.html` used `aVal.localeCompare(bVal)` for column string sorting.
- Testing in Node environment confirms `localeCompare` yields different results from direct unicode code point comparison, creating indeterminism drift.

## 🧭 Options considered
### Option A (recommended)
- Replace `localeCompare` in `report.html` with explicit strict Unicode code unit comparison (`a < b ? -1 : a > b ? 1 : 0`).
- **Why it fits this repo and shard**: Solves the deterministic drift for target-specific (JS/browser) rendering behavior as explicitly outlined by the Compat persona constraints.
- **Trade-offs**:
  - Structure: Aligns string sorting behavior across Rust and JS boundaries.
  - Velocity: Quick and effective fix.
  - Governance: Zero new dependencies and compliant with determinism rules.

### Option B
- Pass pre-sorted or index-mapped arrays from Rust down to JS.
- **When to choose it instead**: If UI table sorting needed to support complex locale-specific collations beyond basic lexicographical comparisons.
- **Trade-offs**: High structural complexity and bloated template bindings.

## ✅ Decision
Option A was chosen as it effectively resolves the indeterminism drift natively within JS, aligning with Rust lexicographical behavior with minimal changes to the existing template logic.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/analysis/templates/report.html`: Replaced `localeCompare` sorting logic with `a < b ? -1 : a > b ? 1 : 0`.
- `crates/tokmd-format/tests/**`: Updated `.snap` fixtures to reflect the updated `report.html` template.

## 🧪 Verification receipts
```text
INSTA_UPDATE=always cargo test -p tokmd-format
cargo test -p tokmd-format
cargo test -p tokmd-format --no-default-features
cargo test -p tokmd-format --all-features
npm --prefix web/runner test
cargo fmt -- --check
cargo clippy -p tokmd-format -- -D warnings
```

## 🧭 Telemetry
- Change shape: Implementation code and snapshot tests.
- Blast radius: Output / compatibility (Browser UI tables sorting behavior).
- Risk class: Low - fixes UI indeterminism without altering data models.
- Rollback: Revert the template changes.
- Gates run: `cargo test`, `cargo fmt`, `cargo clippy`, `npm test`

## 🗂️ .jules artifacts
- `.jules/runs/compat_targets_matrix/envelope.json`
- `.jules/runs/compat_targets_matrix/decision.md`
- `.jules/runs/compat_targets_matrix/receipts.jsonl`
- `.jules/runs/compat_targets_matrix/result.json`
- `.jules/runs/compat_targets_matrix/pr_body.md`

## 🔜 Follow-ups
None.
