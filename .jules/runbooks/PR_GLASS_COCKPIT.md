---
# PR Glass Cockpit

Make review boring. Make truth cheap.

## 💡 Summary
1–4 sentences. What changed.

## 🎯 Why (perf bottleneck)
What was wasteful and where it showed up (runtime/allocations/CPU/IO/compile time).

## 📊 Proof (before/after)
Prefer one:
- benchmark output (cargo bench / criterion / existing harness)
- runtime timing using repo-provided fixtures/examples
- structural proof (work eliminated) + why it matters
If unmeasured, say so and explain why.

## 🧭 Options considered
### Option A (recommended)
- What it is
- Why it fits this repo
- Trade-offs: Structure / Velocity / Governance

### Option B
- What it is
- When to choose it instead
- Trade-offs

## ✅ Decision
State the decision and why.

## 🧱 Changes made (SRP)
Bullets with file paths.

## 🧪 Verification receipts
Copy from the run envelope. Commands + results.

## 🧭 Telemetry
- Change shape
- Blast radius (API / IO / format stability / concurrency)
- Risk class + why
- Rollback
- Merge-confidence gates (what ran)

## 🗂️ .jules updates
What changed in .jules and why.

## 📝 Notes (freeform)
Optional. Extra context for future runs or reviewers.

## 🔜 Follow-ups
If anything remains, create friction items and link them.
---
