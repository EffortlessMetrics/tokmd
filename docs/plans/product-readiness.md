# Plan: Product Readiness User Paths

- Status: active
- Related proposal:
- Related spec:
- Related ADR:
- Related issues:

## Goal

Make the main tokmd jobs obvious for users who do not care about the internal
control plane:

```text
1. Tell me what this repo is.
2. Tell me what changed.
3. Help me review this PR.
4. Give CI stable evidence and gates.
5. Give my coding agent the right context and proof expectations.
```

The first pass should simplify and connect existing surfaces. It should not add
new product commands or promote advisory proof.

## Non-goals

- Do not add a separate `tokmd review` command before it has a distinct
  artifact contract from cockpit.
- Do not promote fast proof, scoped coverage, mutation, or Codecov upload.
- Do not make README or tutorial examples depend on unpublished behavior.
- Do not turn onboarding docs into architecture inventory.
- Do not implement evidencebus export in this lane.

## Work Packets

1. Add a cockpit review-packet quickstart.
   - Show the local commands for generating doc-artifacts evidence when
     available, running `tokmd cockpit`, and checking the review packet.
   - Explain which packet files a reviewer should open first.
2. Simplify README first-run paths around the five user jobs.
   - Keep the command inventory available, but lead with the smallest useful
     commands for inspection, PR review, CI evidence, and agent handoff.
3. Refresh tutorial and recipes around job-to-be-done flows.
   - Prefer short workflows that produce a visible receipt before explaining
     the underlying proof or schema machinery.
4. Tighten browser/native capability guidance.
   - Keep browser mode as a no-install artifact generator with explicit
     native-only boundaries.
5. Keep handoff docs aligned with the shipped link artifacts and `work-order.md`.
   - The guide should tell agents how to consume links, not imply handoff
     verifies external receipts.

## Validation

For docs-only PRs in this lane, run:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan
cargo fmt-check
git diff --check
```

For PRs that change examples tied to generated CLI reference output, also run
the relevant generator/checker listed by `cargo xtask docs --check`.

## Stop Conditions

- Stop if a user-facing workflow needs behavior the CLI does not currently
  support.
- Stop before adding new proof gates, Codecov defaults, or merge verdicts.
- Stop before adding a new command when existing `cockpit`, `handoff`, `gate`,
  or `context` surfaces can express the workflow.
- Stop if affected planning reports unknown files for docs, source-of-truth, or
  proof-policy changes.

## Checkpoint History

- 2026-05-13: Created after the agent-handoff readiness lane completed. The
  next product-readiness slice should make existing review, CI, browser, and
  agent workflows easier to start without expanding the control plane.
