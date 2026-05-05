## 💡 Summary
Record learning outcome for a superseded patch. The original intended change to expand `isRunMessage` constraints inside `web/runner/messages.js` to accept `paths` and `scan` configurations was superseded by PR #1594.

## 🎯 Why
During execution of the patch fixing validation drift between the Rust core and browser runner capabilities for multi-surface execution targets, the changes were found to overlap with and be superseded by a separate recently-merged PR (#1594). In compliance with the workflow expectations for superseded work, the active product patch was gracefully aborted and converted into this learning PR to document the edge case.

## 🔎 Evidence
- Pull Request Comment indicating supersession by #1594.

## 🧭 Options considered
### Option A (recommended)
- Revert the current changes and file a learning PR with the documented friction.
- Why it fits: Aligns strictly with expectations for handling superseded, redundant fixes.

### Option B
- Ignore the comment and continue modifying the code.
- Why to choose it: Only if the user comment explicitly instructs to reopen and disregard the overlap.
- Trade-offs: Directly contradicts instructions and risks merge conflicts.

## ✅ Decision
Option A was chosen to gracefully handle the superseded workflow state by documenting the friction and creating a learning PR.

## 🧱 Changes made (SRP)
- `.jules/friction/open/bridge_bindings_wasm_superseded.md`

## 🧪 Verification receipts
```text
No product verification receipts required. Learning PR submission.
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None
- Risk class: None
- Rollback: None
- Gates run: None

## 🗂️ .jules artifacts
- `.jules/runs/bridge_bindings_wasm/envelope.json`
- `.jules/runs/bridge_bindings_wasm/decision.md`
- `.jules/runs/bridge_bindings_wasm/receipts.jsonl`
- `.jules/runs/bridge_bindings_wasm/result.json`
- `.jules/runs/bridge_bindings_wasm/pr_body.md`
- `.jules/friction/open/bridge_bindings_wasm_superseded.md`

## 🔜 Follow-ups
None.
