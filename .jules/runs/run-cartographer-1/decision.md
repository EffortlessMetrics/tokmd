## Drift Identified

The roadmap doc (`ROADMAP.md`), the implementation plan (`docs/implementation-plan.md`), and the changelog designate `Tree-sitter AST integration` as a long-term goal (e.g., `v3.0.0`) and `Adze AST integration` as `v4.0.0`. However, ADR-0008 ("AST foundation and shadow rollout") was accepted and implemented recently in the `tokmd-analysis` crate, utilizing `tree-sitter` and `tree-sitter-rust` as feature-gated dependencies, despite `ROADMAP.md` completely neglecting to update its "Status Summary" table and current goals to reflect this newly landed Phase 7/v3.0 foundation work, or acknowledge it in recent milestones (even though the `ast` feature is now actively used).

Moreover, `ROADMAP.md` still lists "Tree-sitter AST integration" and "Adze AST integration" as pure long-term ideas under `v3.0.0` and `v4.0.0`, despite Phase 7 work already being underway, leading to factual drift between the shipped code (where AST capabilities are now compiled and tested via feature flags) and the high-level roadmap docs.

## Options Considered

### Option A (Recommended)
- Update `ROADMAP.md` and `docs/implementation-plan.md` to reflect that the Tree-sitter AST shadow foundation (Phase 7 / ADR-0008) has *begun* and landed behind a feature flag (`ast`), updating the status and milestone logs to match reality.
- Remove references to `Adze` AST integration entirely if it's a hallucinated or outdated component that never shipped, or explicitly mark Tree-sitter shadow mode as the actively shipping implementation.
- This resolves the drift and provides accurate guidance for contributors looking to utilize the new AST feature gate.

### Option B
- Revert the `ast` code in `crates/tokmd-analysis` and delete ADR-0008.
- This creates massive code churn and removes actively functioning and merged capabilities, which violates the builder style of improving code rather than regressing it.

## Decision
Choose Option A. It's the standard anti-drift move: the code shipped a partial, feature-gated shadow implementation of Tree-sitter (ADR-0008), so the roadmap and implementation plans must be updated to acknowledge that Phase 7 / v3.0 work has started and the AST foundation is now partially complete and in-tree.
