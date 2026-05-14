# Plan: Artifact Glossary

- Status: complete
- Related proposal:
- Related spec:
- Related ADR:
- Related issues:

## Goal

Give users and agents one dictionary for the artifacts they see across repo
inspection, PR review, CI proof, documentation-control, browser, and handoff
workflows.

The glossary should answer:

```text
what writes this artifact
where it usually lives
what it means
what it does not mean
what verifies or reproduces it
```

## Non-goals

- Do not change any receipt schema or product behavior.
- Do not add a new command.
- Do not promote proof gates, scoped coverage, mutation, or Codecov upload.
- Do not implement evidencebus export.
- Do not replace formal schema docs with the glossary.

## Work Packets

1. Add `docs/artifacts.md`.
   - Status: complete.
   - Cover review packet, proof/CI, documentation-control, handoff, browser,
     and core repo/change artifacts.
2. Link the glossary from user-path entry points.
   - Status: complete.
   - Add links from README and `docs/start-here.md` so users can find the
     dictionary from the first-run path.
3. Record the lane checkpoint.
   - Status: complete.
   - Keep `docs/NEXT.md` and `.jules/goals/active.toml` aligned without
     reopening the completed first product-readiness pass.

## Validation

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-artifact-glossary.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-artifact-glossary.json --evidence-json target/proof/proof-evidence-artifact-glossary.json
cargo fmt-check
git diff --check
```

## Stop Conditions

- Stop if the glossary requires behavior the CLI does not currently support.
- Stop if a listed artifact cannot be traced to a current command, workflow, or
  doc contract.
- Stop if affected planning reports unknown files for this docs-only lane.

## Checkpoint History

- 2026-05-14: Added the artifact glossary as a one-PR product-readiness
  compression lane after the first user-path pass completed.
