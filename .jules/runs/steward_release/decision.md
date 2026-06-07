# Decision

## Option A (recommended)
Implement `std::fmt::Display` for `ExclusionReason` in `xtask/src/tasks/publish.rs` and update `print_plan` to use `Display` instead of `Debug` for exclusion reasons. Currently, `cargo xtask publish --plan --verbose` outputs raw Debug formatting (e.g. `NotPublishable`), which looks like an unpolished internal detail for release governance tooling.

* Fits repo and shard: Directly targets the `xtask` workspace tooling for release metadata presentation (a stated primary path and focus).
* Trade-offs:
  * Structure: Marginally increases boilerplate by adding a standard `Display` impl.
  * Velocity: Negligible cost.
  * Governance: Improves the polish and clarity of the publish plan drift output (a top-priority target) without altering behavioral semantics.

## Option B
Find another release/governance issue, perhaps in `CHANGELOG.md` or `.github/workflows/release.yml`.
* When to choose: If the current `publish --plan` output format isn't considered "drift" or "RC-hardening" enough.
* Trade-offs: Might require larger, riskier changes, potentially missing the "low-risk, high-confidence" requirement.

## Decision
Choose Option A. It's a low-risk, zero-behavior-change improvement to the governance release tooling output surface, perfectly aligning with the "Stabilizer" style and the specific request to focus on `publish-plan` presentation drift.
