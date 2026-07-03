## 💡 Summary
This fixes a minor factual drift in `docs/SCHEMA.md`. The document incorrectly listed `BASELINE_VERSION` as being defined in `crates/tokmd-analysis-types/src/lib.rs`. It is actually defined in `crates/tokmd-analysis-types/src/baseline.rs`.

## 🎯 Why
The `Librarian` persona prioritizes fixing factual docs drift. The README and memory point to `docs/SCHEMA.md` keeping truth with Rust constants. A search revealed that `docs/SCHEMA.md` incorrectly points to `lib.rs` for `BASELINE_VERSION`, while the Rust source defines it in `baseline.rs`. This factual drift violates the "docs/schema/help text mismatch" constraint and needs correction.

## 🔎 Evidence
- File: `docs/SCHEMA.md`
- Observed behavior: Points to incorrect source file.
- Receipt: `grep -rn "pub const BASELINE_VERSION" crates/` shows it is in `crates/tokmd-analysis-types/src/baseline.rs:20`.

## 🧭 Options considered
### Option A (recommended)
- What it is: Update `docs/SCHEMA.md` to correctly point to `crates/tokmd-analysis-types/src/baseline.rs` for `BASELINE_VERSION`.
- Why it fits this repo and shard: Fixes a clear documentation drift regarding schema versioning. It's a quick, factual doc fix that complies with Librarian's constraints.
- Trade-offs: Structure / Velocity / Governance. Minimal risk, corrects truth without changing logic.

### Option B
- What it is: Do nothing and record learning.
- When to choose it instead: If the drift wasn't verifiable or wasn't part of the shard.
- Trade-offs: Misses an opportunity to fix a small but clear piece of factual drift.

## ✅ Decision
Option A. I updated `docs/SCHEMA.md` to fix the file path for `BASELINE_VERSION`.

## 🧱 Changes made (SRP)
- `docs/SCHEMA.md`

## 🧪 Verification receipts
```text
$ rg -n "pub const BASELINE_VERSION" crates/tokmd-analysis-types/src
crates/tokmd-analysis-types/src/baseline.rs:20:pub const BASELINE_VERSION: u32 = 1;
$ cargo xtask doc-artifacts --check
doc artifacts ok
$ cargo xtask docs --check
Documentation is up to date.
$ cargo xtask proof-policy --check
proof policy ok
$ cargo xtask proof --profile affected --base origin/main --head HEAD --run-required --allow-local-required-execution --proof-run-summary target/proof/proof-run-summary-librarian-baseline-path.json
required affected proof passed
$ cargo xtask proof-run-artifacts-check --proof-run-summary target/proof/proof-run-summary-librarian-baseline-path.json
Proof run artifacts OK: 6 executed required command(s), guard local_explicit_required_opt_in_enabled
```

## 🧭 Telemetry
- Change shape: Docs update
- Blast radius: Docs only
- Risk class: Low
- Rollback: `git restore docs/SCHEMA.md`
- Gates run: `cargo xtask doc-artifacts --check`, `cargo xtask docs --check`, `cargo xtask proof-policy --check`, `cargo fmt-check`, affected required proof, proof-run artifact check

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples/envelope.json`
- `.jules/runs/librarian_docs_examples/decision.md`
- `.jules/runs/librarian_docs_examples/receipts.jsonl`
- `.jules/runs/librarian_docs_examples/result.json`
- `.jules/runs/librarian_docs_examples/pr_body.md`

## 🔜 Follow-ups
None.
