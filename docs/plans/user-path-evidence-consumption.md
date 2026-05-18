# Plan: User Path Evidence Consumption

- Status: complete
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
   - Status: complete.
   - Fulfilled by `docs/ci/proof-observation-artifacts.md`,
     `docs/user-paths.md`, `docs/artifacts.md`, and `docs/workflows.md`.
   - These docs keep required proof, advisory proof, scoped coverage, mutation,
     coverage, and promotion-readiness boundaries explicit.
6. Add copy-ready workflow sequences when the path chooser still leaves users
   assembling commands from multiple pages.
   - Status: complete.
   - Compose existing `tokmd` and `xtask` commands; do not add new CLI
     behavior.
7. Close or split #2299.
   - Status: complete.
   - #2299 was mined into narrow keeper slices and closed as superseded:
     #2337, #2338, #2339, #2340, and #2341.
   - Any remaining coverage work must be restacked from current `main`, name one
     owner surface, report zero unknown affected files, and avoid broad
     generated residue.

## Closeout Decision

The lane is complete. The repo now has a job-to-artifact map, small physical
artifact trees, copy-ready workflows, a more actionable handoff work order,
clearer cockpit review-map priority wording, proof evidence reading order, a
browser trial path, publishing evidence guidance, and a clean disposition for
the broad generated coverage PR.

No new product command, proof promotion, Codecov default, AST product behavior,
evidencebus runtime, or release mutation was added. Future product-readiness
work should start from a fresh consumer or artifact gap instead of extending
this lane by inertia.

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
- 2026-05-16: Added copy-ready workflows for inspection, PR review, proof
  planning, proof observation summaries, agent handoff, browser-to-native
  review, and publishing evidence.
- 2026-05-16: Closed #2299 as superseded after mining keeper slices into
  #2337, #2338, #2339, #2340, and #2341.
- 2026-05-16: Closed the lane. The remaining work is not more control-plane
  compression; the next lane should start only from a fresh user, artifact, or
  workflow gap.
