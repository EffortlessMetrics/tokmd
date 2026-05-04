## 💡 Summary
Replaced `String.prototype.localeCompare` with strict Unicode comparisons in the web runner's input ordering logic. This ensures deterministic sorting behavior across all environments.

## 🎯 Why
In JS/Node environments, `String.prototype.localeCompare()` is platform-dependent and causes determinism drift. Using strict Unicode code-unit comparisons ensures determinism and matches Rust's native lexicographical `String::cmp` and `BTreeMap` sorting behavior, fixing a compatibility issue between JS environments and Rust core.

## 🔎 Evidence
- File: `web/runner/ingest.js`
- `localeCompare` uses platform locale settings, meaning path sorting order would drift depending on where the runner executes.
```text
{"command": "grep -rn \"localeCompare\" web/runner crates/tokmd-node crates/tokmd-wasm crates/tokmd-python", "output": "web/runner/ingest.js:534:            return priority === 0 ? leftPath.localeCompare(rightPath) : priority;"}
```

## 🧭 Options considered
### Option A (recommended)
- Replace `localeCompare` with strict Unicode comparison (`leftPath < rightPath ? -1 : leftPath > rightPath ? 1 : 0`).
- Why it fits this repo and shard: The `bindings-targets` shard is responsible for cross-platform alignment, and removing determinism drift directly improves target compatibility.
- Trade-offs:
  - Structure: Guarantees 1:1 input sorting parity with Rust's native `String::cmp`.
  - Velocity: Quick, focused inline patch.
  - Governance: Standardizes on strict Unicode sorting.

### Option B
- Write an extensive WASM bridge method just to handle lexicographical sorting.
- When to choose it instead: If the path normalization and sorting logic was exceedingly complex or needed direct Rust regex evaluation.
- Trade-offs: Massively higher complexity and bundle overhead for a very simple comparison operation.

## ✅ Decision
Option A. It's the most direct, performant, and reliable way to ensure determinism for string sorting in JS to match Rust.

## 🧱 Changes made (SRP)
- `web/runner/ingest.js`: Replaced `localeCompare` with strict `<`/`>` comparison in the `.sort()` callback for input path entries.

## 🧪 Verification receipts
```text
{"command": "cd web/runner && npm test", "output": "Tests passed, 1 skip (built tokmd-wasm bundle not present)"}
```

## 🧭 Telemetry
- Change shape: Internal logic fix
- Blast radius: API (JS runner input sorting)
- Risk class: Low - fixes a non-deterministic sort to a deterministic one, well covered by runner test suite.
- Rollback: Revert the PR
- Gates run: `compat-matrix` fallback tests (`npm test` in the JS surface)

## 🗂️ .jules artifacts
- `.jules/runs/compat_targets_matrix/envelope.json`
- `.jules/runs/compat_targets_matrix/decision.md`
- `.jules/runs/compat_targets_matrix/receipts.jsonl`
- `.jules/runs/compat_targets_matrix/result.json`
- `.jules/runs/compat_targets_matrix/pr_body.md`

## 🔜 Follow-ups
None at this time.
