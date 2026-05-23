# Agent Source-of-Truth Workflow

Status: active workflow guide.

Use this guide when starting, continuing, or handing off a tokmd lane. It is
operational guidance for humans and agents; the durable ownership rules remain
in `.tokmd-spec/`, `.tokmd-spec/index.toml`, `docs/source-of-truth.md`,
`docs/specs/`, `docs/adr/`, `docs/plans/`, `AGENTS.md`, `.codex/` state where
present, `ci/proof.toml`, and `policy/*.toml`.

For Codex work, `.jules/**` is Jules provenance and ambient suggestion state.
Review it when relevant, but do not treat it as Codex's primary active-lane
controller.

## Before Starting

1. Check the open PR queue.
2. Fetch `tokmd-swarm` and `tokmd` refs, then verify the dual-repo graph before
   starting ordinary swarm work:

   ```bash
   cargo xtask repo-graph \
     --publication public/main \
     --swarm origin/main \
     --expect aligned \
     --json target/repo-graph/agent-start.json
   ```

   If the graph is not aligned, do not start unrelated feature work. Follow the
   publication-import, fast-forward, hotfix, or emergency-repair path in
   `docs/ci/swarm-routing.md` first.
3. Read `docs/NEXT.md` for the current operating mode.
4. Read `.tokmd-spec/index.toml`, then the accepted plan, linked spec, ADR,
   proposal, or policy file named by current repo guidance or PR context.
5. Review `.jules/goals/active.toml` as Jules-local context when it is relevant,
   not as Codex's primary lane selector.
6. Confirm `docs/NEXT.md`, accepted docs/plans/specs/ADRs, and the PR context do
   not contradict each other.

If those artifacts disagree, stop and fix the routing artifact that owns the
truth before opening an implementation branch.

## Choosing The Owning Artifact

- Use a proposal for why, alternatives, and open questions.
- Use a spec for behavior contracts, artifact shapes, compatibility, and proof
  requirements.
- Use an ADR for durable architecture, packaging, governance, or product-boundary
  decisions.
- Use a plan for PR order, dependencies, validation commands, and stop
  conditions.
- Use `.tokmd-spec/` and `.tokmd-spec/index.toml` for repo-native durable rails
  and for links to durable artifacts that still live under `docs/`.
- Use `.jules/goals/active.toml` only for Jules-local machine-readable state and
  suggestions.
- Use `.codex/` for Codex-local tracked state when a Codex workflow needs a
  durable in-repo packet.
- Use `ci/proof.toml` and `policy/*.toml` for machine-checkable rules.
- Use PR bodies for review-local evidence and links, not as the only durable
  source of truth when a repo artifact should exist.

## While Changing Docs

When a change touches source-of-truth artifacts:

1. Keep each artifact in its lane. Do not put implementation sequencing in a
   spec, or durable architecture rationale in an active-goal file.
2. Update checked TOML policy when the claim should be enforced by tooling.
3. Keep `.jules/goals/active.toml` short and Jules-local when it is changed.
4. Do not let `.jules/goals/active.toml` become a Codex controller or run log.
5. Archive a completed or superseded active goal under `.jules/goals/archive/`
   only when the Jules-local machine-readable checkpoint has durable value.
6. Run the documentation artifact checker before opening the PR.

## PR Body Checklist

For non-trivial source-of-truth changes, the PR body should include:

- what changed and why the change removes a concrete reviewer, agent, user,
  proof, or workflow ambiguity;
- linked source-of-truth artifact or active goal;
- changed layer, such as proposal, spec, ADR, plan, active goal, policy, or
  proof scope;
- validation commands and result summary;
- repo-graph evidence for the current swarm or publication step;
- claim boundary that names behavior, schemas, release mutation, proof
  promotion, Codecov defaults, AST behavior, evidencebus runtime, or public CLI
  surfaces that are intentionally unchanged;
- rollback or parking path that preserves the swarm/publication graph;
- stop condition or parking rationale if the work is intentionally incomplete;
- explicit note when product behavior, schemas, proof promotion, Codecov
  defaults, or publish surface are not changed.

## After A Swarm PR Merges

A completed swarm PR should not leave the dual-repo graph unexamined. After a
swarm squash merge, fetch both repositories and record the current relation:

```bash
cargo xtask repo-graph \
  --publication public/main \
  --swarm origin/main \
  --expect swarm-descends-publication \
  --json target/repo-graph/post-swarm-merge.json
```

If `tokmd-swarm/main` is ahead of `tokmd/main`, either perform the publication
checkpoint described in `docs/ci/swarm-routing.md` or leave an explicit
handoff note explaining why the import is deferred. If the publication import
lands, fast-forward `tokmd-swarm/main` to the publication merge commit and rerun
`repo-graph` with `--expect aligned`.

Required publication checks and an aligned repo graph prove the workbench loop.
Publication-only side workflows such as Nix full validation are release-boundary
evidence. When those side workflows are still active, record the run ID or URL,
head SHA, attempt, status, conclusion, and active job or step if available.
Key the note by `headSha` so older commit-scoped Nix full runs are not confused
with the current publication head. Use bounded status snapshots from
`docs/ci/cache-and-cancellation.md#run-status-polling`; do not leave an
unbounded `gh run watch` session running while waiting on long matrix or Nix
jobs. Do not cite a side workflow as passing proof until it completes
successfully.

## Validation

Use the relevant subset:

```bash
cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan
cargo fmt-check
git diff --check
```

Run package, schema, or publish-surface checks only when the changed artifacts
touch those surfaces.

## Stop Conditions

Stop before implementation when:

- `docs/NEXT.md`, accepted docs/plans/specs/ADRs, and the PR context disagree;
- a behavior or artifact shape change has no owning spec;
- an architecture boundary change has no ADR;
- a machine-checkable claim has no policy owner;
- the affected proof plan reports unknown files for source-of-truth changes;
- validation output does not cover the PR's stated contract.
