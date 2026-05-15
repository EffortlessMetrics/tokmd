# Plan: Publishing Evidence Readiness

- Status: complete
- Related proposal:
- Related spec: `docs/specs/publishing-evidence.md`
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
   - Status: complete.
   - Decision: existing `publish-surface --json --verify-publish` output is the
     first machine-readable publishing evidence artifact. A separate wrapper
     receipt is deferred until a consumer needs one.
   - Map release metadata, version consistency, publish-surface, CI lane
     whitelist, release workflow, and package closure checks to their current
     evidence.
2. Add a user-facing publishing evidence guide.
   - Status: complete.
   - Explain what to run before release work, what artifact to open first, and
     what checks do not prove.
   - Keep this as documentation unless the guide exposes a genuine product gap.
3. Add artifact-glossary entries for release-facing evidence.
   - Status: complete.
   - Include `publish-surface` JSON output, version consistency output, release
     metadata scope, and release workflow artifacts if they are current.
4. Decide whether a Rust-owned publishing evidence receipt is needed.
   - Status: complete.
   - Decision: no new wrapper receipt is needed yet.
   - Close the lane as docs-only guidance plus existing proof routing. Reopen
     only if a concrete consumer cannot use the current publish-surface,
     version-consistency, CI lane, release workflow, and affected-proof
     evidence set.

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
- 2026-05-15: Added the publishing evidence spec. It keeps
  `publish-surface --json --verify-publish` as the first machine-readable
  artifact, maps current release metadata, CI lane, workflow, and proof-routing
  evidence, and defers any wrapper receipt until a consumer proves the need.
- 2026-05-15: Added the user-facing publishing evidence guide. It gives
  maintainers and agents the command order, artifact reading order, meanings,
  non-meanings, and release-mutation boundary before any behavior change.
- 2026-05-15: Added release-facing artifact glossary entries for
  publish-surface JSON, version consistency output, `release_metadata` proof
  scope routing, CI release lane policy, and intentional release workflow
  artifacts.
- 2026-05-15: Closed the lane. The current evidence set is useful without a
  wrapper receipt: `publish-surface --json --verify-publish` owns package
  surface and closure facts, version consistency owns metadata alignment,
  affected/proof planning owns release-file routing, CI lane policy owns
  release-check obligations, and release workflow artifacts remain the
  intentional mutation evidence. A new receipt should start from a fresh
  proposal naming the consumer and gap.
