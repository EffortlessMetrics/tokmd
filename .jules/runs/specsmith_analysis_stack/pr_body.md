## 💡 Summary
Learning PR documenting that the prompted baseline fallback fix for `ComplexityBaseline::from_analysis` is already implemented on `main`. No code changes made.

## 🎯 Why
The original prompt/memory instructed Specsmith to fix an issue where structural metrics (`total_files`, `total_code_lines`) were incorrectly zeroed when extracting from `receipt.derived` if `receipt.complexity` was `None`. However, this was already resolved on the upstream branch. Proceeding with a Learning PR prevents duplicate/no-op branches from being merged while honoring the prompt pipeline.

## 🔎 Evidence
- **Finding**: A PR review comment confirmed that the fallback logic fix was already merged on `main`.
- **Proof**: `git reset --hard HEAD` left the working directory identical to `origin/main`.

## 🧭 Options considered
### Option B (recommended)
- Pivot to a Learning PR.
- **When to choose it**: When the intended logic target from memory or prompt has already been solved in the upstream branch.
- **Trade-offs**: Requires giving up on landing a code patch, but honors the truth of the repository state and preserves the prompt-to-PR pipeline flow as a learning outcome.

### Option A
- Re-implement the fallback fix from scratch.
- **Why it fits**: It doesn't.
- **Trade-offs**: Creates unnecessary duplication since the fix already exists on `main`.

## ✅ Decision
**Option B**. The baseline fallback fix is already present on current main. We will document the friction and close out as a learning PR instead of forcing a fake fix.

## 🧱 Changes made (SRP)
- Created `.jules/friction/open/specsmith_baseline_fallback_obsolete.md`

## 🧪 Verification receipts
```text
git reset --hard HEAD
cargo test -p tokmd-analysis-types
cargo fmt -- --check
cargo clippy -p tokmd-analysis-types -- -D warnings
cargo xtask gate --check
```

## 🧭 Telemetry
- **Change shape**: Learning PR + Friction Item
- **Blast radius**: None.
- **Risk class**: None.
- **Rollback**: Trivial revert.
- **Gates run**: `cargo xtask gate --check`, `cargo test -p tokmd-analysis-types`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_analysis_stack/envelope.json`
- `.jules/runs/specsmith_analysis_stack/decision.md`
- `.jules/runs/specsmith_analysis_stack/receipts.jsonl`
- `.jules/runs/specsmith_analysis_stack/result.json`
- `.jules/runs/specsmith_analysis_stack/pr_body.md`
- Added `.jules/friction/open/specsmith_baseline_fallback_obsolete.md` noting the obsolete prompt.

## 🔜 Follow-ups
Review Jules memory prompt to ensure it is not repeatedly driving tasks that are already merged on main.
