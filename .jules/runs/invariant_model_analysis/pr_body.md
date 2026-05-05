## 💡 Summary
Learning PR: Aborted the patch because it was superseded by #1584, which merged the aligned row-sorting extraction with source-local ordering tests and no new public API surface.

## 🎯 Why
During the review cycle, the PR was closed due to another PR (#1584) implementing the same objective without increasing the public API surface. The correct action, per the agent instructions, is to gracefully abort the redundant fix, record the friction item, and submit a learning PR.

## 🔎 Evidence
- Pull Request Comment ID 4380774199: "Superseded by #1584, which merged the aligned row-sorting extraction with source-local ordering tests and no new public API surface."

## 🧭 Options considered
### Option A
- Proceed with the original plan to extract sorting logic into public functions in `tokmd-model` and update `determinism_w66.rs`.
- **Trade-offs**: Unnecessary since a similar change (#1584) was already merged, increasing the API surface pointlessly.

### Option B (recommended)
- Abort the code patch and create a learning PR documenting that the PR was superseded by #1584.
- **Why it fits**: The prompt strictly instructs: "If an intended patch is found to be superseded by another merged PR during execution, gracefully abort the redundant fix and create a 'learning PR'. This involves generating the standard run artifacts and a new friction item (in `.jules/friction/open/`) documenting the workflow edge case."
- **Trade-offs**:
  - *Structure*: Ensures no conflicting/redundant patches are opened.
  - *Velocity*: Fast.
  - *Governance*: Conforms to memory instructions on graceful aborts.

## ✅ Decision
Option B. Aborting the code changes and generating a learning PR.

## 🧱 Changes made (SRP)
- Created `.jules/friction/open/superseded_pr.md`

## 🧪 Verification receipts
```text
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None
- Risk class + why: None
- Rollback: None
- Gates run: None

## 🗂️ .jules artifacts
- `.jules/runs/invariant_model_analysis/envelope.json`
- `.jules/runs/invariant_model_analysis/decision.md`
- `.jules/runs/invariant_model_analysis/receipts.jsonl`
- `.jules/runs/invariant_model_analysis/result.json`
- `.jules/runs/invariant_model_analysis/pr_body.md`
- `.jules/friction/open/superseded_pr.md`

## 🔜 Follow-ups
None.
