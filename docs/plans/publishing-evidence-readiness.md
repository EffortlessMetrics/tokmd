# Plan: Publishing Evidence Readiness

- Status: active
- Related proposal:
- Related spec:
- Related ADR: `docs/adr/0001-production-package-publishability.md`, `docs/adr/0003-publish-surface-taxonomy.md`, `docs/adr/0005-release-train-and-rc-semantics.md`
- Related issues:

## Goal

Make tokmd's release and publishing facts easier to consume without changing
publishing behavior.

Today the repo already has strong release-facing checks:

```text
cargo xtask publish-surface --json --verify-publish
cargo xtask version-consistency
cargo xtask ci-plan --github-output
ci/proof.toml release_metadata scope
policy/ci-lane-whitelist.toml release lanes
.github/workflows/release.yml
```

The gap is product compression. A maintainer, CI job, or coding agent should be
able to tell what publishing evidence exists, what it proves, what it does not
prove, and which command reproduces it without reading release workflow YAML.

## Non-goals

- Do not publish crates, tag releases, move `v1`, create GitHub releases, or
  push Docker images.
- Do not change release workflow behavior.
- Do not change package membership, crate publishability, dependency closure,
  or public API surface.
- Do not change public `tokmd` CLI behavior or receipt schemas.
- Do not promote advisory proof, scoped coverage, mutation, or Codecov upload.
- Do not make publishing evidence a merge verdict.

## Work Packets

1. Define the publishing evidence artifact contract.
   - Status: active.
   - Decide whether existing `publish-surface --json` output is enough for the
     first artifact, or whether a separate wrapper receipt is needed later.
   - Map release metadata, version consistency, publish-surface, CI lane
     whitelist, release workflow, and package closure checks to their current
     evidence.
2. Add a user-facing publishing evidence guide.
   - Status: pending.
   - Explain what to run before release work, what artifact to open first, and
     what checks do not prove.
   - Keep this as documentation unless the guide exposes a genuine product gap.
3. Add artifact-glossary entries for release-facing evidence.
   - Status: pending.
   - Include `publish-surface` JSON output, version consistency output, release
     metadata scope, and release workflow artifacts if they are current.
4. Decide whether a Rust-owned publishing evidence receipt is needed.
   - Status: pending.
   - If needed, write a proposal/spec before implementation.
   - If not needed, close the lane with docs-only guidance and proof routing.

## Validation

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-publishing-evidence-readiness.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-publishing-evidence-readiness.json --evidence-json target/proof/proof-evidence-publishing-evidence-readiness.json
cargo xtask publish-surface --json --verify-publish
cargo fmt-check
git diff --check
```

Run required affected proof if the affected plan selects it. Run
`cargo xtask version-consistency` if release metadata, package manifests,
release workflow, or version docs change.

## Stop Conditions

- Stop if the lane requires publishing, tagging, or mutating release state.
- Stop if a proposed artifact would change public receipt schemas without a
  spec or ADR.
- Stop if affected planning reports unknown files.
- Stop if the work would promote advisory proof or Codecov defaults.
- Stop if the guide cannot explain current behavior without changing release
  automation; split that into a separate implementation plan.

## Checkpoint History

- 2026-05-15: Started from the code-intelligence platform audit. The audit
  found publishing facts verified but less user-facing than proof, cockpit, and
  handoff facts.
