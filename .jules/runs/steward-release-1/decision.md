## Problem
The `cargo xtask check-file-policy --strict` gate check is failing. There are non-Rust files in the repository that are not covered by the allowlist in `policy/non-rust-allowlist.toml`. Specifically:
- `fixtures/syntax/python/native_boundary.py`
- `fixtures/syntax/typescript/component.tsx`
- `fixtures/syntax/typescript/native_boundary.ts`
- `scripts/check-no-bare-self-hosted.sh`

## Options considered
### Option A (recommended)
- **What it is:** Add the missing entries to `policy/non-rust-allowlist.toml` for `fixtures/syntax/**` and `scripts/check-no-bare-self-hosted.sh`.
- **Why it fits this repo and shard:** This is a low-risk, high-confidence tooling/governance fix that aligns exactly with the Steward persona's mandate to resolve release-surface checks and ensure metadata alignment.
- **Trade-offs:**
  - Structure: Improves exactitude of policy governance.
  - Velocity: Quick and zero-risk to runtime behavior.
  - Governance: Directly resolves the strict governance check drift.

### Option B
- **What it is:** Remove the failing files if they are unused.
- **When to choose it instead:** If the files were accidentally committed and serve no real purpose.
- **Trade-offs:** We don't have proof they are unused (the syntax fixtures are likely needed for tree-sitter AST tests). Riskier than just allowlisting them.

## Decision
Option A. It's safe, corrects the policy drift, and turns the repo green.
