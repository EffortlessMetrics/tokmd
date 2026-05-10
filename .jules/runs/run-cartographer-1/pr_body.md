## 💡 Summary
Updated `ROADMAP.md` and `docs/implementation-plan.md` to reflect that the Tree-sitter AST shadow foundation (Phase 7 / ADR-0008) has begun and landed behind a feature flag (`ast`). Also removed hallucinatory Adze AST integration references from `ROADMAP.md` and `CHANGELOG.md`.

## 🎯 Why
The roadmap and implementation plans designated Phase 7 / `v3.0.0` (Tree-sitter AST integration) as pure long-term ideas. However, the repository has already shipped ADR-0008 and a partial feature-gated `ast` shadow implementation in `crates/tokmd-analysis`. This causes a mismatch between the high-level roadmap docs and the actively shipped truth in the codebase. Furthermore, "Adze AST integration" was listed as `v4.0.0` but lacks any factual grounding in the codebase, so it is removed as drift theater.

## 🔎 Evidence
- file path(s): `ROADMAP.md`, `docs/implementation-plan.md`, `crates/tokmd-analysis/Cargo.toml`
- observed behavior / finding: `crates/tokmd-analysis` actively contains `tree-sitter` and `tree-sitter-rust` dependencies behind the `ast` capability feature. `ROADMAP.md` still labeled the entire concept as a "🔭 Long-term" `v3.0.0` milestone, despite the work already actively being integrated behind feature flags.
- command receipt:
  ```text
  grep -rin "Tree-sitter" .
  # Shows both the `ast` crate additions and the lagging `ROADMAP.md` status.
  grep -rin "adze" .
  # Shows Adze only in `ROADMAP.md` and `CHANGELOG.md` with no ADR or code.
  ```

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `ROADMAP.md` and `docs/implementation-plan.md` to flag `v3.0.0` Tree-sitter integration as `🚧 Active (Shadow)` since foundation work has landed. Remove hallucinated `Adze` references.
- why it fits this repo and shard: It resolves factual drift between the shipped code and the design docs, meeting the Cartographer persona's explicit target: "roadmap/design/requirements drift from shipped reality".
- trade-offs:
  - Structure: Aligns high-level planning docs with the actual crate dependency tree.
  - Velocity: Eliminates confusion for contributors reading `ROADMAP.md`.
  - Governance: Stops "strategy theater" by stripping the non-existent `Adze` references.

### Option B
- what it is: Revert the `ast` shadow code and delete ADR-0008 to make reality match the outdated docs.
- when to choose it instead: Only if the code was merged accidentally and genuinely needs removing.
- trade-offs: Creates massive code churn and removes actively functioning capabilities; violates the builder style.

## ✅ Decision
Choose Option A. It correctly acknowledges the recently landed AST foundation (ADR-0008) in the design and roadmap docs, achieving the anti-drift constraint.

## 🧱 Changes made (SRP)
- `ROADMAP.md`: Updated `v3.0.0` status to Active (Shadow), updated the `J. Tree-sitter AST Parsing` checklist to reflect landed items, and removed `Adze`.
- `docs/implementation-plan.md`: Updated Phase 7 goal to acknowledge active shadow mode foundation work.
- `CHANGELOG.md`: Removed the inaccurate Adze roadmap note.

## 🧪 Verification receipts
```text
cargo xtask docs --check
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Documentation update (drift fix)
- Blast radius: docs
- Risk class: Low (no production code changed)
- Rollback: `git checkout ROADMAP.md docs/implementation-plan.md CHANGELOG.md`
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/run-cartographer-1/envelope.json`
- `.jules/runs/run-cartographer-1/decision.md`
- `.jules/runs/run-cartographer-1/receipts.jsonl`
- `.jules/runs/run-cartographer-1/result.json`
- `.jules/runs/run-cartographer-1/pr_body.md`

## 🔜 Follow-ups
None.
