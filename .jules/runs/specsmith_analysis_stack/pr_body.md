## 💡 Summary
This learning PR documents an attempt to extract BDD integration tests into `tokmd-analysis` which failed topology verification and was closed as wrong-repo intake. This change belongs in the `tokmd-swarm` repository.

## 🎯 Why
The Specsmith assignment aimed to improve scenario coverage for the analysis stack by separating BDD-style tests from unit tests. However, the attempt was rejected because this logic originates in `EffortlessMetrics/tokmd-swarm` and must be ported there first.

## 🔎 Evidence
- Original target: `crates/tokmd-analysis/tests/bdd.rs`
- Feedback received: "Closing as wrong-repo intake for the current topology. Normal implementation lands in EffortlessMetrics/tokmd-swarm and is imported into EffortlessMetrics/tokmd by merge commit."

## 🧭 Options considered
### Option A (recommended)
- **What it is:** Abandon the PR in this repo, record the learning, and plan to port the effort to `tokmd-swarm`.
- **Why it fits this repo and shard:** Adheres to the multi-repo topology contract and stops hallucinated or duplicate work.
- **Trade-offs:** Short-term velocity loss for long-term consistency.

### Option B
- **What it is:** Keep pushing the PR in this repo.
- **When to choose it instead:** Never, as it breaks repository boundaries.
- **Trade-offs:** Rejection and sync drift.

## ✅ Decision
Option A. I am rolling back the local changes, closing this effort as a learning PR, and documenting the wrong-repo intake.

## 🧱 Changes made (SRP)
- Recorded a learning PR about repository boundaries and PR intake.
- Reverted all attempted codebase modifications (the extraction of `tests/bdd.rs` and the changes to `ci/proof.toml` and `tests/analysis_deep_w64.rs`).

## 🧪 Verification receipts
```text
$ git reset --hard origin/main
HEAD is now at ...
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None (documentation only)
- Risk class: Low
- Rollback: N/A
- Gates run: None

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_analysis_stack/envelope.json`
- `.jules/runs/specsmith_analysis_stack/decision.md`
- `.jules/runs/specsmith_analysis_stack/receipts.jsonl`
- `.jules/runs/specsmith_analysis_stack/result.json`
- `.jules/runs/specsmith_analysis_stack/pr_body.md`
- `.jules/friction/open/wrong-repo-intake.md`

## 🔜 Follow-ups
Port the BDD test extraction logic to a narrow PR in `EffortlessMetrics/tokmd-swarm`.
