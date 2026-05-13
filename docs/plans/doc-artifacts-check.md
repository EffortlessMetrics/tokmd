# Plan: Documentation Artifact Checker

- Status: active
- Related proposal: none
- Related spec: `docs/specs/doc-artifacts.md`
- Related ADR: `docs/adr/0000-adr-process.md`
- Related issues: none

## Goal

Make tokmd's source-of-truth stack checkable without changing product behavior.

The first implementation target is:

```bash
cargo xtask doc-artifacts --check
```

That command should verify the shape, links, and routing of source-of-truth
documentation artifacts defined by `docs/source-of-truth.md` and
`docs/specs/doc-artifacts.md`.

## Non-goals

- Do not promote proof gates.
- Do not enable default Codecov upload.
- Do not change receipt schemas.
- Do not move existing top-level docs into the new directories.
- Do not make the checker judge prose quality or merge readiness.
- Do not add a new product command; keep this in `xtask`.

## Work Packets

1. Add the doc-artifact contract.

   Land `docs/specs/doc-artifacts.md` and this implementation plan. Keep the
   change docs-only and route it through existing docs/proof-policy checks.

2. Add policy configuration.

   Add `policy/doc-artifacts.toml` with the artifact families, required
   sections, allowed statuses, and active-goal schema name. Keep the first
   policy intentionally small and readable.

3. Add the xtask checker.

   Implement `cargo xtask doc-artifacts --check` with focused tests for:

   - valid current repo artifacts;
   - broken active-goal links;
   - missing required plan/spec sections;
   - invalid ADR filenames;
   - unknown active-goal schema.

4. Wire into docs validation.

   After the checker is stable, call it from `cargo xtask docs --check` or the
   appropriate docs CI lane. Keep this as documentation validation, not a proof
   promotion.

5. Consider a JSON receipt.

   Add `--json <path>` after the text checker has landed and a follow-up
   consumer needs machine-readable checker evidence. The receipt should stay
   visibility-only and must not promote proof gates or Codecov defaults.

6. Upload the JSON receipt from docs CI.

   Have the Docs Check job write `target/docs/doc-artifacts-check.json` and
   upload it as a visibility-only artifact. Keep the existing Docs Check job as
   the validation gate and do not add a product command or proof promotion.

7. Route source-of-truth changes to the checker.

   Ensure `ci/proof.toml` maps proposals, specs, ADRs, plans, templates,
   `.jules/goals/**`, the doc-artifact policy, and checker implementation files
   to `cargo xtask doc-artifacts --check` so affected-proof output does not
   treat source-of-truth changes as generic docs only.

8. Define review-packet evidence behavior.

   Document how a future cockpit review packet should treat
   `target/docs/doc-artifacts-check.json` as documentation-control evidence for
   source-of-truth changes. Keep this docs-only until cockpit has an explicit
   import path.

## Validation

Each implementation PR should run the relevant subset of:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan
cargo fmt-check
git diff --check
```

Run `cargo xtask publish-surface --json --verify-publish` only if a PR touches
package, export, public API, or publish-surface files.

## Stop Conditions

- Stop and update the spec if the checker needs to enforce behavior not covered
  by `docs/specs/doc-artifacts.md`.
- Stop and add an ADR if the checker changes durable architecture, governance,
  or proof-promotion policy.
- Stop before wiring CI if the checker creates noisy failures on current docs.
- Stop before adding new JSON fields or CI upload unless a concrete consumer
  exists.

## Checkpoint History

- 2026-05-13: Source-of-truth routing landed in `docs/source-of-truth.md`.
  This plan starts the follow-up path toward a Rust-owned documentation artifact
  checker.
- 2026-05-13: Draft policy configuration was added in
  `policy/doc-artifacts.toml` so the future checker can read artifact families,
  statuses, required sections, and active-goal link rules from a repo policy
  file instead of hard-coding the source-of-truth contract.
- 2026-05-13: `cargo xtask doc-artifacts --check` landed as the first
  Rust-owned checker for the policy file. It validates the current
  source-of-truth shape and keeps docs-validation wiring as a separate follow-up
  so noisy enforcement can be handled deliberately.
- 2026-05-13: `cargo xtask docs --check` now invokes the doc-artifacts checker
  after generated reference documentation validation, completing the first
  docs-lane wiring without adding a product command or proof-promotion gate.
- 2026-05-13: `cargo xtask doc-artifacts --check --json <path>` now writes a
  `tokmd.doc_artifacts_check.v1` receipt with checked counts and errors for CI,
  review packets, or later evidencebus consumers. Text output remains the
  default, and the receipt is visibility-only.
- 2026-05-13: The CI Docs Check job now writes
  `target/docs/doc-artifacts-check.json`, appends it to the step summary, and
  uploads it as the `doc-artifacts-check` artifact without changing proof
  promotion or Codecov behavior.
- 2026-05-13: `ci/proof.toml` now routes source-of-truth artifact families,
  `.jules/goals/**`, the doc-artifact policy, and checker implementation files
  through the `doc_artifacts_policy` scope so affected proof includes the
  doc-artifacts checker whenever the source-of-truth stack changes.
- 2026-05-13: `docs/review-packet.md` now defines the planned packet treatment
  for `tokmd.doc_artifacts_check.v1` receipts as documentation-control evidence
  without adding a cockpit import path or changing merge behavior.
