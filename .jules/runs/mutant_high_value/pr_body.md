## 💡 Summary
Updated `clean_path` in `tokmd-format::redact` to correctly resolve parent directory segments (`..`), handling empty segments from double slashes and respecting absolute paths. This ensures path determinism and prevents directory structure leakage via logical bypass paths.

## 🎯 Why
The previous implementation of `clean_path` only normalized separators and stripped `.` segments but did not resolve `..` segments. This allowed identical physical paths like `src/main.rs` and `src/../src/main.rs` or `foo//../main.rs` to produce entirely different hashes. This breaks the determinism contract, bypassing redaction rules and risking directory structure leakage. Initial attempts to fix this missed edge cases with absolute paths and double slashes which the revised implementation solves correctly.

## 🔎 Evidence
- **File:** `crates/tokmd-format/src/redact/mod.rs`
- **Finding:** A `short_hash("src/../src/main.rs")` output differed from `short_hash("src/main.rs")`.
- **Receipt:** Tests added to `crates/tokmd-format/src/redact/mod.rs` failed prior to this change.

## 🧭 Options considered
### Option A (recommended)
- Implement a robust stack-based loop inside `clean_path` to handle `..` natively, accounting for double slashes and absolute paths.
- **Why it fits:** Keeps path handling strictly isolated within string manipulation avoiding OS-specific logic from `std::path::Path`. This guarantees identical outcomes across Unix/Windows for identical paths.
- **Trade-offs:**
  - Structure: Path normalization stays within the target file.
  - Velocity: Extremely fast execution with predictable cross-platform string logic.
  - Governance: Satisfies `contracts-determinism` fallback validation profile and prevents path traversal edge cases safely.

### Option B
- Use standard `std::path::Path::components()`.
- **Why it fits:** Relies on standardized algorithms.
- **Trade-offs:** OS-specific path parsing can lead to hash drift across platforms for identical file names, which breaks cross-platform determinism contracts.

## ✅ Decision
Option A was chosen. Handling string manipulation directly ensures consistency across Unix and Windows and aligns exactly with `tokmd`'s determinism requirements, whilst addressing all edge cases raised during review.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/redact/mod.rs`
  - Added parent directory resolution logic to `clean_path` resolving `..` securely while ignoring double slashes.
  - Handled absolute paths ensuring leading `/` is preserved during evaluation.
  - Appended explicit regression tests verifying paths handle `..` appropriately, alongside absolute path tests.

## 🧪 Verification receipts
```text
cargo test -p tokmd-format
CI=true cargo test -p tokmd-format --verbose
cargo build
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Implementation Fix + Proof Tests.
- Blast radius: Output Formatting & Path Hashing Logic.
- Risk class: Low, path hashes remain correct and predictable but are now fully resilient against `..` exploits.
- Rollback: Revert the PR safely.
- Gates run: `cargo build --verbose`, `CI=true cargo test -p tokmd-format --verbose`, `cargo clippy`, `cargo fmt`.

## 🗂️ .jules artifacts
- `.jules/runs/mutant_high_value/envelope.json`
- `.jules/runs/mutant_high_value/decision.md`
- `.jules/runs/mutant_high_value/receipts.jsonl`
- `.jules/runs/mutant_high_value/result.json`
- `.jules/runs/mutant_high_value/pr_body.md`

## 🔜 Follow-ups
None.
