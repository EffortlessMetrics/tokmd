# Proposal: Release Readiness Receipt

- Status: proposed
- Owner: release/distribution readiness
- Related issues:
- Related specs: `docs/specs/publishing-evidence.md`
- Related ADRs: `docs/adr/0001-production-package-publishability.md`, `docs/adr/0003-publish-surface-taxonomy.md`, `docs/adr/0005-release-train-and-rc-semantics.md`

## Problem

Release-facing users now have several strong artifacts:

- `cargo xtask publish-surface --json --verify-publish`
- `cargo xtask version-consistency`
- `cargo xtask affected --json-output ...`
- `cargo xtask proof --profile affected --plan ...`

The adoption question is whether those artifacts are sufficient for a
maintainer to check release readiness, or whether tokmd needs a wrapper receipt
such as:

```bash
cargo xtask release-readiness --json target/release/readiness.json
```

A wrapper could make release evidence easier to consume, but it would also add
another artifact family and another command before a consumer has proven that
the existing artifacts are too hard to use.

## Goals

- Decide whether to add a release-readiness wrapper receipt now.
- Preserve release evidence as pre-release evidence, not release approval.
- Keep the command path clear for outside maintainers.
- Name the consumer gap that would justify a future wrapper.
- Avoid adding machinery when existing artifacts already answer the question.

## Non-goals

- Do not publish crates.
- Do not create tags, GitHub releases, release aliases, or Docker images.
- Do not change release workflow behavior or package membership.
- Do not change public `tokmd` CLI behavior.
- Do not promote advisory proof, scoped coverage, mutation, or Codecov upload.
- Do not add a merge or release verdict.

## Options

Option A: no wrapper yet.

Keep the current release evidence path as composed artifacts:

```text
publish-surface JSON/stdout
version-consistency output
affected.json
proof-plan.json
proof-evidence.json
```

Use `docs/release-readiness.md` as the command-first guide and
`docs/publishing-evidence.md` as the explanation layer. This preserves the
current artifact ownership and avoids a second source of truth.

Option B: add `cargo xtask release-readiness --json`.

Add a wrapper that reads or invokes existing evidence sources and emits a
single `tokmd.release_readiness.v1` receipt with availability, status, and
warnings for package surface, version consistency, affected proof, and proof
planning.

This would make release automation easier to consume, but it would need a new
schema, tests, docs, routing, and verifier expectations. It also risks turning
pre-release evidence into an implied release verdict unless carefully worded.

Option C: add a docs-only release evidence checklist.

Keep the current commands but add a manually maintained checklist. This is
less useful than the current quickstart because it is not machine-readable and
could drift from the existing artifacts.

## Recommendation

Choose Option A for now: do not add a release-readiness wrapper receipt.

The current adoption guides already give the user a command-first path:

```text
publish-surface
  -> version-consistency
  -> affected routing, when release-facing files changed
  -> proof plan, when scoped evidence is needed
```

Each artifact has a clear owner and verifier path. A wrapper should wait until
a concrete consumer cannot use those artifacts directly, such as a release
workflow, GitHub Action, or downstream evidence collector that needs one stable
JSON envelope.

If that consumer appears, the future wrapper should be an `xtask` receipt, not
a public `tokmd` command. It should read existing artifacts when possible,
avoid release mutation, and report missing inputs as missing rather than
passing.

## Open Questions

- Will the GitHub Action need a single release-readiness JSON artifact for
  hosted adoption, or are uploaded publish-surface and proof artifacts enough?
- Should a future wrapper invoke checks directly, or only summarize artifacts
  produced by earlier commands?
- What verifier would own a `tokmd.release_readiness.v1` receipt if the wrapper
  is added later?
- Which downstream consumer, if any, needs one envelope instead of the current
  release evidence artifacts?
