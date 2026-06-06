# Real User Path Smoke Run

Use this when your job is:

```text
I want to see the full tokmd review-to-handoff path on a real PR-sized change.
```

This is a smoke transcript, not a generated packet dump. It records one run of
the user path and the human-facing artifacts it produced. Treat the receipts as
evidence for this range only.

## Scenario

- Date: 2026-05-17.
- Range: `a82c5088fc01c11e3739af12bc1c2fab76b1c14a` to
  `386f76cd5459bb60b5dcd196a98f2d1df04a4b3f`.
- Change: GitHub Action quickstart docs plus related links and proof routing.
- Local note: `tokmd` on `PATH` was `1.10.0`, while the workspace binary was
  `1.12.0`, so the smoke run used `cargo run -p tokmd -- ...` for commands
  that need current branch behavior.

## Run First

Plan affected proof:

```bash
cargo xtask affected \
  --base a82c5088fc01c11e3739af12bc1c2fab76b1c14a \
  --head 386f76cd5459bb60b5dcd196a98f2d1df04a4b3f \
  --json-output target/proof/user-path-smoke/affected.json

cargo xtask proof \
  --profile affected \
  --base a82c5088fc01c11e3739af12bc1c2fab76b1c14a \
  --head 386f76cd5459bb60b5dcd196a98f2d1df04a4b3f \
  --plan \
  --plan-json target/proof/user-path-smoke/proof-plan.json \
  --evidence-json target/proof/user-path-smoke/proof-evidence.json
```

Generate and verify the review packet:

```bash
tokmd cockpit \
  --base a82c5088fc01c11e3739af12bc1c2fab76b1c14a \
  --head 386f76cd5459bb60b5dcd196a98f2d1df04a4b3f \
  --review-packet-dir .tokmd/user-path-smoke/review

cargo xtask review-packet-check \
  --dir .tokmd/user-path-smoke/review \
  --json target/tokmd/user-path-smoke/review-packet-check.json
```

Prepare an agent handoff:

```bash
tokmd handoff \
  --preset risk \
  --budget 128k \
  --strategy spread \
  --review-packet-dir .tokmd/user-path-smoke/review \
  --review-packet-check target/tokmd/user-path-smoke/review-packet-check.json \
  --affected target/proof/user-path-smoke/affected.json \
  --proof-plan target/proof/user-path-smoke/proof-plan.json \
  --out-dir .handoff/user-path-smoke
```

## Open First

1. `.tokmd/user-path-smoke/review/review-map.md`
2. `.tokmd/user-path-smoke/review/comment.md`
3. `.tokmd/user-path-smoke/review/evidence.json`
4. `target/tokmd/user-path-smoke/review-packet-check.json`
5. `.handoff/user-path-smoke/work-order.md`

## What Was Clear

- Affected planning found 9 changed files, 6 matched proof scopes, and 0
  unknown files.
- The proof plan listed 32 required commands and no advisory commands for this
  docs/control-plane change.
- `review-map.md` clearly put the source-of-truth plan and `ci/proof.toml`
  ahead of ordinary guide files.
- Review-map reproduction commands used the actual packet directory,
  `.tokmd/user-path-smoke/review`, instead of assuming `.tokmd/review`.
- `review-packet-check` verified 5 packet-local artifacts and 5 hashes.
- `work-order.md` summarized changed surfaces, review evidence, proof
  expectations, missing/unavailable evidence, and agent stop conditions.

## What Was Confusing

- The installed `tokmd` binary on `PATH` can lag the workspace version. When
  smoke-testing unreleased behavior, use the workspace binary or reinstall
  before treating output as current.
- The cockpit packet correctly reported doc-artifacts evidence as missing for
  source-of-truth changes because this smoke run did not generate and import
  `target/docs/doc-artifacts-check.json`.
- The proof plan is easy to mistake for executed proof. It is planned evidence
  until the listed commands run and produce their receipts.

## What Was Fixed

- The review packet now shows packet-directory-specific reproduction commands
  for the smoke path.
- The handoff work order is actionable enough to give an agent the review
  order, proof expectations, and stop conditions without opening every linked
  JSON file first.

## What Was Deferred

- This run did not import doc-artifacts evidence into cockpit. For a complete
  source-of-truth review packet, first run:

  ```bash
  cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json
  ```

  Then pass the receipt to cockpit with the appropriate doc-artifacts input.
- This run planned proof but did not execute all 32 required commands.
- This run did not produce browser, publishing, release, coverage, mutation, or
  Codecov evidence.

## What Not To Infer

- A verified review packet is not a merge verdict.
- A handoff bundle links review and proof artifacts; it does not verify those
  external artifacts itself.
- Missing, unavailable, skipped, stale, or degraded evidence is not passing
  proof.
- Advisory proof, coverage, mutation, browser output, and Codecov upload remain
  advisory unless policy explicitly promotes them.
- This smoke run proves the workflow for the recorded range, not permanent
  readiness for future changes.

## Next Action

Use [User paths](../user-paths.md) to choose the right workflow for your job.
Use [Copy-Ready Workflows](../workflows.md) when you need the command sequence
without this transcript. If a source-of-truth file changed, generate and import
the doc-artifacts receipt before relying on cockpit evidence completeness.
