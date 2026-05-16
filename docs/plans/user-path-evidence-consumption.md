# Plan: User Path Evidence Consumption

- Status: active
- Related proposal:
- Related spec:
- Related ADR:
- Related issues:

## Goal

Make the existing `tokmd` evidence surfaces easier to consume from real user
jobs:

```text
inspect repo
review PR
prepare agent handoff
read CI proof evidence
try browser mode
publish/release safely
```

This lane starts from the platform audit result: the core proof, cockpit,
browser, publishing, handoff, and AST-shadow foundations are already present,
and no new implementation lane should start without a concrete consumer or
artifact gap. The work here is compression. A user or coding agent should be
able to answer:

```text
what do I run?
what artifact did it create?
what do I open first?
what does it mean?
what does it not mean?
what is the next action?
```

## Non-goals

- Do not add a public `tokmd review` command.
- Do not promote fast proof, scoped coverage, mutation, or Codecov upload.
- Do not add AST-backed default output or browser AST capability claims.
- Do not implement evidencebus runtime integration.
- Do not reopen architecture cleanup without a fresh owner-module pressure
  point.
- Do not merge broad generated coverage PRs such as #2299 without restacking
  them into narrow reviewed keeper slices.

## Work Packets

1. Add the user-path chooser.
   - Status: complete.
   - Add `docs/user-paths.md` as the command-to-artifact map for inspect,
     review, handoff, CI proof, browser, and publishing jobs.
   - Link it from README, Start Here, and the docs index without duplicating
     all command details.
2. Add small sample artifact trees.
   - Status: complete.
   - Add concise examples for review packets, handoff bundles, proof status,
     browser receipts, and publishing evidence.
   - Do not check in large generated dumps.
3. Improve cockpit review-map readability when fresh evidence shows the user
   guide is still not enough.
   - Status: complete.
   - Keep schema compatibility or version deliberately.
   - Preserve cockpit as evidence, not a merge verdict.
4. Improve handoff `work-order.md` readability when the current linked-evidence
   summary still leaves agent tasks ambiguous.
   - Status: complete.
   - Summarize external evidence handles; do not make handoff verify those
     external receipts.
5. Add a proof evidence reading-order guide if the user-path chooser and
   artifact glossary are not enough for CI owners.
   - Status: pending.
   - Keep required proof, advisory proof, scoped coverage, mutation, coverage,
     and promotion-readiness boundaries explicit.
6. Close or split #2299.
   - Status: pending.
   - Keep #2299 parked unless a slice is restacked on current `main`, has zero
     unknown affected files, and drops placeholder-pinning or misleading test
     names.

## Validation

For docs-only PRs in this lane, run:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-user-path-evidence-consumption.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-user-path-evidence-consumption.json --evidence-json target/proof/proof-evidence-user-path-evidence-consumption.json
cargo fmt-check
git diff --check
```

If a later packet changes cockpit, handoff, or proof behavior, add the focused
tests named by the affected proof plan and the relevant packet verifier.

## Stop Conditions

- Stop if a workflow needs behavior the current CLI cannot express; write a
  proposal or spec before implementation.
- Stop before adding a new public command when existing `cockpit`, `handoff`,
  `context`, `gate`, or `xtask` surfaces cover the job.
- Stop before promoting advisory proof or enabling default Codecov upload.
- Stop before treating browser mode as native-equivalent.
- Stop if affected planning reports unknown files.
- Stop if a generated coverage PR is still broad, stale, or pins placeholder
  behavior.

## Checkpoint History

- 2026-05-16: Started after the code-intelligence platform audit refresh
  selected no automatic implementation lane. The first packet defines the lane
  and adds the user-path evidence chooser.
- 2026-05-16: Added small sample artifact trees for review packets, handoff
  bundles, proof status, browser receipts, and publishing evidence.
- 2026-05-16: Made `work-order.md` more directly actionable with changed
  surfaces, review evidence, proof expectations, missing-evidence, and stop
  condition sections.
- 2026-05-16: Made `review-map.md` more explicit about review-first signals
  and the packet's non-verdict boundary.
