## 💡 Summary
This is a learning PR. The previously attempted performance patch (swapping `BTreeMap` for `rustc_hash::FxHashMap` in `tokmd-analysis` hotpaths) was closed due to wrong-repo intake. The work was successfully proven locally but must be ported to the `EffortlessMetrics/tokmd-swarm` repository to align with the current topology.

## 🎯 Why
A PR reviewer closed the implementation PR because `tokmd` imports `tokmd-swarm` via merge commits, meaning direct implementation in the `tokmd` repository is the wrong topological layer for core library patches. This run documents that friction to prevent future wasted work.

## 🔎 Evidence
- Pull Request Comment ID: 4684603396
- Comment text: "Closing as wrong-repo intake for the current topology. Normal implementation lands in EffortlessMetrics/tokmd-swarm and is imported into EffortlessMetrics/tokmd by merge commit."

## 🧭 Options considered
### Option A
- Force the code patch despite the wrong repo closure.
- Trade-offs: Directly violates reviewer instructions and repository topology rules.

### Option B (recommended)
- Abort the patch and produce a Learning PR.
- When to choose it instead: Always, when explicitly instructed by human review that the current environment topology is wrong.
- Trade-offs: Delays the performance win, but adheres to strict repo governance.

## ✅ Decision
Choosing Option B. I have reverted the codebase changes to `crates/tokmd-analysis` and am submitting the run packet and friction item as a learning PR instead.

## 🧱 Changes made (SRP)
- Created `.jules/friction/open/wrong_repo_intake_friction.md` to document the topological constraint.
- Updated `.jules/runs/bolt_analysis_stack_builder/` artifacts to reflect a learning PR outcome.

## 🧪 Verification receipts
```text
git checkout crates/tokmd-analysis
git clean -fd crates/tokmd-analysis
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: Internal documentation only.
- Risk class: None.
- Rollback: Revert the PR.
- Gates run: None required for learning PR.

## 🗂️ .jules artifacts
- `.jules/runs/bolt_analysis_stack_builder/envelope.json`
- `.jules/runs/bolt_analysis_stack_builder/decision.md`
- `.jules/runs/bolt_analysis_stack_builder/receipts.jsonl`
- `.jules/runs/bolt_analysis_stack_builder/result.json`
- `.jules/runs/bolt_analysis_stack_builder/pr_body.md`
- `.jules/friction/open/wrong_repo_intake_friction.md`

## 🔜 Follow-ups
- Port the `FxHashMap` optimization to a narrow `tokmd-swarm` PR.
