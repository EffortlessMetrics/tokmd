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

   Add `--json <path>` only after the text checker has landed and a follow-up
   consumer needs machine-readable checker evidence.

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
- Stop before adding JSON output unless a concrete consumer exists.

## Checkpoint History

- 2026-05-13: Source-of-truth routing landed in `docs/source-of-truth.md`.
  This plan starts the follow-up path toward a Rust-owned documentation artifact
  checker.
- 2026-05-13: Draft policy configuration was added in
  `policy/doc-artifacts.toml` so the future checker can read artifact families,
  statuses, required sections, and active-goal link rules from a repo policy
  file instead of hard-coding the source-of-truth contract.
