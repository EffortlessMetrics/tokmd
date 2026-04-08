## 💡 Summary
This is a **learning PR**. I discovered that `crates/tokmd-analysis-content` fails to explicitly enforce deterministic `BTreeMap` sorting via `Vec::sort_by()` before serialization. Because this path is strictly outside the allowed `core-pipeline` shard, I've created a friction item instead of drifting out of scope to patch it.

## 🎯 Why
The Gatekeeper's goal was to protect contract-bearing surfaces and lock in deterministic behavior. Memory explicitly states: "To guarantee deterministic reporting across the tokmd codebase, exported lists generated from BTreeMaps (such as `TodoTagRow` or `ImportEdge` collections) should use explicit `.sort_by()` to define a complete multi-field ordering (e.g., count descending, then alphabetical), rather than relying solely on the implicit alphabetical iteration order of the BTreeMap's keys."

Scanning the allowed `core-pipeline` (types, scan, model, format) showed these paths properly implement the explicit sorting rule on vector maps. However, expanding the search revealed the rule was not applied in `crates/tokmd-analysis-content/src/content.rs` for `TodoTagRow` and `ImportEdge`. Because this is outside the allowed shard, the instructions state: "If the strongest target you find is outside the shard, record it as friction instead of chasing it."

## 🔎 Evidence
- File: `crates/tokmd-analysis-content/src/content.rs`
- Missing `.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.tag.cmp(&b.tag)))` on `TodoTagRow` collection.
- Incomplete sort missing full descending count logic on `ImportEdge` collection.
- Core pipeline passes sorting rules based on `crates/tokmd-model/src/lib.rs` which explicitly sorts all aggregated rows recursively.

## 🧭 Options considered
### Option A (recommended)
- Stop and generate a learning PR instead of forcing an artificial/redundant "fix" inside the `core-pipeline` shard that doesn't actually solve a problem. Log the out-of-scope missing sort as a friction item.
- Fits the `Gatekeeper` persona perfectly: "If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix."
- Trade-offs: Requires a follow-up run on the correct shard to implement the structural fix.

### Option B
- Modify `crates/tokmd-analysis-content/src/content.rs` anyway to apply the sorting rule.
- When to choose it instead: If the rules didn't explicitly forbid wandering outside of the specified shard paths (`core-pipeline`).
- Trade-offs: Breaks the `SRP` and explicit negative constraints.

## ✅ Decision
Option A. It upholds the prompt's hard constraint around the shard boundary and output honesty.

## 🧱 Changes made (SRP)
- Recorded `gatekeeper_determinism_missing_sorts.md` friction item in `.jules/friction/open/`.

## 🧪 Verification receipts
```text
grep -rn "TodoTagRow" crates
grep -rn "ImportEdge" crates
```

## 🧭 Telemetry
- Change shape: Learning PR + Friction Item
- Blast radius: Documentation and Policy
- Risk class: Informational
- Rollback: Remove the generated friction item.
- Gates run: Not applicable to prose changes.

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_determinism/envelope.json`
- `.jules/runs/gatekeeper_determinism/decision.md`
- `.jules/runs/gatekeeper_determinism/receipts.jsonl`
- `.jules/runs/gatekeeper_determinism/result.json`
- `.jules/runs/gatekeeper_determinism/pr_body.md`
- `.jules/friction/open/gatekeeper_determinism_missing_sorts.md`

## 🔜 Follow-ups
- A `Surveyor` or `Specsmith` agent running within the `analysis` shard should process `.jules/friction/open/gatekeeper_determinism_missing_sorts.md`.
