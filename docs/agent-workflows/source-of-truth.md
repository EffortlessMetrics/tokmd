# Agent Source-of-Truth Workflow

Status: active workflow guide.

Use this guide when starting, continuing, or handing off a tokmd lane. It is
operational guidance for humans and agents; the durable ownership rules remain
in `docs/source-of-truth.md`, `docs/specs/`, `docs/adr/`, `docs/plans/`,
`.jules/goals/active.toml`, `ci/proof.toml`, and `policy/*.toml`.

## Before Starting

1. Read `docs/reference/SPEC_SYSTEM.md` for the first-read contract and stop
   conditions.
2. Check the open PR queue.
3. Read `docs/NEXT.md` for the current operating mode.
4. Read `.jules/goals/active.toml` for the active program, lane, linked
   artifacts, rules, and stop conditions.
5. Read the linked plan first, then any linked spec, ADR, proposal, or policy
   file named by the active goal.
6. Confirm `docs/NEXT.md` does not contradict `.jules/goals/active.toml` or the
   linked plan.

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
- Use `.jules/goals/active.toml` only for small machine-readable active state.
- Use `ci/proof.toml` and `policy/*.toml` for machine-checkable rules.
- Use PR bodies for review-local evidence and links, not as the only durable
  source of truth when a repo artifact should exist.

## While Changing Docs

When a change touches source-of-truth artifacts:

1. Keep each artifact in its lane. Do not put implementation sequencing in a
   spec, or durable architecture rationale in an active-goal file.
2. Update checked TOML policy when the claim should be enforced by tooling.
3. Keep `.jules/goals/active.toml` short, current, and linked to human-readable
   artifacts.
4. Do not let `.jules/goals/active.toml` become a run log.
5. Archive a completed or superseded active goal under `.jules/goals/archive/`
   only when the machine-readable checkpoint has durable value.
6. Run the documentation artifact checker before opening the PR.

## PR Body Checklist

For non-trivial source-of-truth changes, the PR body should include:

- linked source-of-truth artifact or active goal;
- changed layer, such as proposal, spec, ADR, plan, active goal, policy, or
  proof scope;
- validation commands and result summary;
- stop condition or parking rationale if the work is intentionally incomplete;
- explicit note when product behavior, schemas, proof promotion, Codecov
  defaults, or publish surface are not changed.

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

- `docs/NEXT.md`, `.jules/goals/active.toml`, and the linked plan disagree;
- a behavior or artifact shape change has no owning spec;
- an architecture boundary change has no ADR;
- a machine-checkable claim has no policy owner;
- the affected proof plan reports unknown files for source-of-truth changes;
- validation output does not cover the PR's stated contract.
