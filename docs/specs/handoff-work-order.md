# Spec: Handoff Work Order

- Status: active
- Schema family, if any: handoff manifest schema version 5; no separate
  `work-order.md` JSON schema
- Related ADRs:
- Related proof scopes: `tokmd_context_handoff`, `project_truth_docs`,
  `doc_artifacts_policy`

## Contract

`.handoff/work-order.md` is the agent-readable task map inside a
`tokmd handoff` bundle. It turns the bundle manifest, selected files, optional
review links, and optional proof links into a consumption guide for coding-agent
work.

The work order is not the authoritative bundle index. `manifest.json` remains
the authoritative artifact inventory, schema-versioned handoff receipt, and
hash source for packet-local artifacts.

The work order is not a proof verifier, merge verdict, release approval, or
policy promotion surface. It may summarize adjacent review and proof receipts
for triage, but the linked review-packet verifier, proof-route receipt,
affected-proof receipt, and proof-plan receipt remain the evidence sources to
inspect before claiming proof.

## Inputs

The work order is generated from explicit handoff inputs:

- the handoff manifest and selected bundle metadata;
- included files, excluded paths, smart exclusions, and token-budget state;
- optional `--review-packet-dir` and `--review-packet-check` inputs;
- optional `--proof-route`, `--affected`, and `--proof-plan` inputs;
- best-effort readable summaries of linked review/proof receipts when supplied.

Input paths recorded in the handoff bundle must be repo-relative or
operator-supplied paths represented as link handles. The work order must not
discover hidden proof state, call GitHub APIs, fetch CI artifacts, execute proof
commands, or mutate external review/proof receipts while rendering.

## Outputs

The work order is a Markdown artifact listed in `manifest.json` as
`work-order.md`. It should give agents a stable reading order and enough
guardrails to avoid treating context as proof.

The generated work order should include these semantic sections or equivalent
content:

- a short statement that the handoff is a deterministic source/context bundle;
- selected-file and bundle-scope summary;
- changed-surface or relevant-source summary when available;
- review evidence handles when review packet links are supplied;
- proof-route, affected-proof, and proof-plan handles when proof links are
  supplied;
- proof expectations when affected/proof-plan links are supplied;
- route ownership, unmatched route files, and skipped-by-policy lane counts
  when a proof-route link is supplied;
- missing, stale, degraded, skipped, or unavailable evidence as work to resolve;
- agent stop conditions and boundaries;
- pointers back to packet-local link artifacts when they exist.

The work order may summarize readable linked receipts, but it must keep those
summaries compact. It should point to source artifacts instead of copying raw
receipt bodies or command output.

## Compatibility

This spec does not change the public `tokmd handoff` CLI, handoff manifest
schema version, work-order filename, link-artifact filenames, review-packet
behavior, proof-plan behavior, Codecov defaults, AST defaults, release
surfaces, or branch-protection gates.

Existing consumers can continue to read:

- `.handoff/manifest.json` as the authoritative artifact index;
- `.handoff/work-order.md` as the human agent brief;
- `.handoff/review-links.json` for review packet handles;
- `.handoff/proof-links.json` for proof-route, affected-proof, and proof-plan
  handles;
- linked review and proof receipts as their own evidence sources.

Future changes that alter required work-order sections, link-artifact
semantics, or manifest artifact requirements should update this spec and the
handoff tests in the same PR.

## Proof Requirements

For spec-only changes:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-handoff-work-order-spec.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-handoff-work-order-spec.json --evidence-json target/proof/proof-evidence-handoff-work-order-spec.json
git diff --check
```

Implementation changes to the work-order renderer should also run focused
handoff tests that prove:

- `work-order.md` is listed in `manifest.json`;
- packet-local hashes remain valid after work-order rendering;
- plain handoffs include the reading order and guardrails;
- linked review/proof handoffs include link handles without copying external
  receipts;
- linked proof-route receipts are summarized as routing and skip-policy
  evidence, not execution proof;
- missing linked receipts are rendered as missing work, not passing proof;
- advisory proof, Codecov upload, review verdicts, release mutation, and AST
  defaults remain unchanged.

## Open Questions

- Whether a future downstream tool needs a separate structured
  `agent-work-order.json` artifact, or whether the current manifest plus
  Markdown work order is enough.
- Whether work-order section headings should become machine-checked once a
  second independent consumer depends on exact headings.
