# Spec: Packet GHCR Container Runtime

- Status: draft
- Schema family, if any: reuses `tokmd.evidence-packet/v1`
  (`docs/evidence-packet.schema.json`); adds no new schema
- Related ADRs:
  `docs/adr/0003-publish-surface-taxonomy.md`,
  `docs/adr/0005-release-train-and-rc-semantics.md`
- Related proof scopes: `tokmd_cli`, `project_truth_docs`, `release_metadata`,
  `doc_artifacts_policy`

## Contract

The `EffortlessMetrics/tokmd` GitHub Action exposes a `runtime` input that
selects how the Action obtains the `tokmd` binary it runs. Today `runtime`
accepts `binary` (the default, implemented) and `container` (reserved, rejected
with a hard error). This spec owns the normative contract for the
`runtime: container` path so the eventual implementation has a fixed target,
claim boundary, and rollout gate before code lands.

`runtime: container` means the Action obtains `tokmd` from a pinned OCI image
pulled from the **publication** registry
(`ghcr.io/effortlessmetrics/tokmd`) instead of downloading a prebuilt release
binary. Once implemented, every Action `mode` (`module`, `export`, `gate`,
`cockpit`, `sensor`, `baseline`, `packet`, and the default flow) must produce
byte-identical artifacts under `runtime: container` and `runtime: binary` for
the same inputs and the same `tokmd` version. The container is a runtime
substitution, not a behavior change.

`runtime: container` is a **secondary** runtime. The prebuilt-binary runtime
remains the default and the supported PR adoption path. The container runtime
must not be advertised as a supported default, and must not be selectable as a
silent fallback, until the verification gate in this spec passes for the
requested image tag.

This spec does **not** own:

- the swarm workbench image `ghcr.io/effortlessmetrics/tokmd-swarm`
  (`docs/specs/swarm-ghcr-image.md`);
- publication release mutation, GHCR push, tag aliasing, or visibility settings
  (`docs/specs/publishing-evidence.md`, `.github/workflows/release.yml`);
- the evidence packet shape, status semantics, or verifier behavior
  (`docs/specs/evidence-packet-workflow.md`).

It owns only how the Action selects, validates, and runs the publication
container as a `tokmd` runtime, and the gate that lets the container runtime be
called supported for a tag.

## Inputs

The container runtime path consumes these Action inputs:

| Input | Status | Default | Used for |
| --- | --- | --- | --- |
| `runtime` | implemented (`binary`); reserved (`container`) | `binary` | Selects binary download vs container pull. |
| `version` | implemented | `latest` | Resolves the image tag for the container runtime (see tag resolution). |
| `image` | implemented | `ghcr.io/effortlessmetrics/tokmd` | Container image reference (without tag) when `runtime: container`. The Action resolves `<image>:<normalized-version>`, accepts only verification-gated tags, and anonymously pulls and runs that image against the mounted workspace. |
| existing per-mode inputs | implemented | — | Unchanged; the runtime does not alter mode behavior. |

Input rules (implemented):

- When `runtime` is not `binary` or `container`, the Action must fail with a
  clear error naming the received value. (Implemented.)
- When `runtime: container` and `version` is a concrete version, the resolved
  image reference is `<image>:<normalized-version>` where the version is
  normalized to the published tag form (for example `1.14.0`, matching the
  publication GHCR tag vocabulary in `docs/specs/swarm-ghcr-image.md`).
  (Implemented.)
- When `runtime: container`, the Action accepts only tags whose full
  verification gate (steps 1-7 below) has passed for that exact tag. Any other
  tag is a hard error pointing at this spec; the Action does not pull it and
  does not silently fall back to the binary runtime. (Implemented.)
- When `runtime: container` and `version` is `latest` (or any mutable
  major/minor alias), the Action rejects it with a hard error rather than
  pulling a mutable tag, so the recorded `tokmd-version` output stays
  reproducible. Callers must pin a verified concrete patch tag. The default
  `version` of `latest` therefore requires an explicit pinned `version` for the
  container runtime. (Implemented; resolves the `latest` open question by
  rejection rather than auto-resolution.)
- `runtime: container` requires a Linux runner with Docker available; on other
  runners or without Docker the Action fails with a clear error. (Implemented.)
- The `image` input must reference the publication registry
  `ghcr.io/effortlessmetrics/tokmd` by default. A non-default `image` is an
  explicit operator override and must not be silently rewritten.
- Container runtime selection must not depend on hidden local state, credentials
  baked into the Action, or operator memory.

## Outputs

The container runtime path produces the same Action outputs as the binary
runtime. It introduces no new packet artifact and no new manifest field.

| Surface | Requirement under `runtime: container` |
| --- | --- |
| Packet artifacts (`sensors/tokmd/*`) | Identical layout, basenames, and status semantics as the binary runtime. |
| `tokmd-version` output | The version reported by the container `tokmd --version`, matching the pinned image tag. |
| Other mode outputs | Unchanged paths and contents relative to the binary runtime for the same inputs. |
| Workspace mounting | The container must run against the checked-out workspace as the working directory so repo-relative scoped paths resolve identically. |

The container image (owned by the publication release pipeline, not this spec)
is expected to provide:

- `tokmd` on `PATH` with `ENTRYPOINT ["tokmd"]`;
- `git` for base/head ref resolution;
- CA certificates;
- a sensible working directory for a mounted repository;
- OCI `source`, `description`, `license`, and `version` labels.

## Compatibility

This spec does not change:

- the default `runtime: binary` behavior or the prebuilt-binary download path;
- the evidence packet schema, status rules, or verifier semantics;
- public `tokmd` CLI behavior or receipt schema versions;
- publication release workflow behavior, GHCR push, tags, or visibility;
- swarm workbench GHCR ownership or claim boundary;
- branch-protection gates, proof promotion, Codecov defaults, or AST defaults.

The `container` branch in `action.yml` is now implemented: for a
verification-gated tag it anonymously pulls `<image>:<normalized-version>` and
runs it against the mounted workspace; for any non-gated or mutable tag it keeps
the hard error pointing at this spec. Any change to the `runtime` input
vocabulary, the `image` input, the container invocation contract, the supported
(gate-verified) tag set, or the verification gate must update this spec and
`docs/packet-workflows.md` in the same change.

## Verification Gate

The container runtime must not be documented or defaulted as supported for a
tag until all of the following pass for that exact tag from an anonymous
(unauthenticated) context, consistent with the publication GHCR evidence rules
in `docs/specs/publishing-evidence.md`:

1. the image was pushed by the publication release pipeline;
2. the expected tag exists;
3. the package is public;
4. anonymous `docker manifest inspect ghcr.io/effortlessmetrics/tokmd:<tag>`
   succeeds (does not return `denied`);
5. anonymous `docker pull` of the tag succeeds;
6. the container reports the expected `tokmd --version`;
7. the container generates a `complete` evidence packet against a mounted
   sample repository using the same scoped-path and base/head contract as the
   binary runtime.

If any anonymous check returns `denied` for a tag, that tag's container runtime
stays **pending**. Do not rewrite tags, rerun release mutation, advertise
container runtime support, or enable a silent container fallback. Fix package
visibility or linkage first, then rerun the full checklist. Push success is not
consumer visibility.

Maintainer outcomes are recorded as publication GHCR visibility receipts under
`target/publishing/ghcr-visibility-<version>.md` and copied into the matching
release ledger, per `docs/specs/publishing-evidence.md`.

### Verification Status

| Tag | Visibility (steps 1-5) | Runtime exec (steps 6-7) | Container runtime |
| --- | --- | --- | --- |
| `1.14.0` (concrete patch tag) | verified-public (2026-06-26) | verified (2026-06-26) | **gate-passed and wired**; accepted by `action.yml` `runtime: container` |
| `1.14` / `1` / `latest` (mutable aliases) | verified-public (2026-06-26) | n/a | **rejected** by `action.yml`; mutable tags are not accepted for the container runtime |

On **2026-06-26**, anonymous registry-API verification (no Docker on the
verification host) confirmed gate steps 1-4 and the registry-level portion of
step 5 for `1.14.0` and its `1.14` / `1` / `latest` aliases: an unauthenticated
pull token resolved every tag to HTTP 200 with one shared OCI image index
digest `sha256:bd214464…b914b096`, and the `linux/amd64` config blob was
fetched anonymously (public package; `image.revision` matches the `v1.14.0`
release tag commit, `image.version` is `1.14.0`). Receipt:
`target/publishing/ghcr-visibility-1.14.0.md`; ledger copy in
`docs/releases/1.14-ledger.md`.

Gate steps 6 (container `tokmd --version`) and 7 (mounted-repository
`complete` packet smoke) require a Docker-capable host. The OCI `image.version`
label is image-declared metadata, not a runtime exec, so it does not discharge
step 6 on its own.

Steps 6-7 are run by the `GHCR Container Smoke` lane
(`.github/workflows/ghcr-container-smoke.yml`), a `workflow_dispatch`-only smoke
that anonymously pulls the published image, runs `tokmd --version`, and
generates a `complete` packet against a mounted git fixture. See
[GHCR container smoke runbook](../ci/ghcr-container-smoke.md). The lane only
pulls and runs the already-published image; it records gate evidence for adding
a tag to `action.yml`'s supported set. It does not perform release mutation.

On **2026-06-26**, that lane ran on `ubuntu-latest`
([run 28262553040](https://github.com/EffortlessMetrics/tokmd-swarm/actions/runs/28262553040))
and discharged gate steps 6-7 for `1.14.0` from an anonymous context: anonymous
`docker pull` resolved the image to digest `sha256:bd214464…b914b096` (matching
the verified-public visibility digest), the container reported `tokmd 1.14.0`
(step 6), and a mounted-repository `tokmd packet generate --no-syntax` produced a
`status: complete` `tokmd.evidence-packet/v1` manifest with
`tokmd_version: 1.14.0` (step 7). All seven gate steps now pass for `1.14.0`, so
the container runtime is gate-verified for that tag. Receipt:
`target/publishing/ghcr-visibility-1.14.0-runtime-exec.md` (under the gitignored
`target/` tree, uploaded as the `ghcr-container-smoke-receipt-1.14.0` artifact);
summary copied into `docs/releases/1.14-ledger.md`.

PR B then wired the `runtime: container` path in `action.yml`: it replaces the
old hard error with an anonymous `docker pull` of the verification-gated tag plus
a mounted-workspace `docker run` wrapper (matching the GHCR Container Smoke mount
pattern). The Action accepts only `1.14.0` today; mutable tags (`latest`, `1.14`,
`1`) and any non-gated tag remain hard errors pointing at this spec. The
container-runtime test in `.github/workflows/test-action.yml` exercises both the
supported-tag success path and the unverified/mutable rejection paths. Each new
stable tag must re-enter the verification gate and be added to the Action's
supported-tag set before its container runtime is called supported.

## Claim Boundary

When implemented and verified for a tag, the container runtime proves only that:

- the pinned publication image for that tag is anonymously pullable;
- the container `tokmd` runs and reports the expected version;
- the container produces the same evidence packet contract as the binary
  runtime for the recorded inputs.

The container runtime does **not** prove:

- undefined behavior presence or absence in any scanned repository;
- public reachability, memory safety, or absence of panic paths;
- evidence packet completeness beyond the recorded `manifest.json` status;
- CI proof, fuzzing, Miri, mutation, coverage, or release readiness;
- merge readiness or human-review completion;
- container runtime health for any tag other than the verified tag;
- swarm workbench GHCR support claims.

A verified container runtime for one tag says nothing about the next tag. Each
new stable tag re-enters the verification gate before its container runtime is
called supported.

## Proof Requirements

For this spec-only change:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-packet-ghcr-runtime-spec.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-packet-ghcr-runtime-spec.json --evidence-json target/proof/proof-evidence-packet-ghcr-runtime-spec.json
git diff --check
```

When the `runtime: container` path is implemented, the implementing PR should
also prove:

- the `runtime` input still rejects unknown values with a clear error;
- `runtime: container` resolves a concrete, reproducible image tag;
- container-runtime artifacts match binary-runtime artifacts for the same
  inputs and version on at least one mode (packet) with a recorded receipt;
- the verification gate checklist outcome is recorded as a publication GHCR
  visibility receipt before container runtime support is advertised.

Proof establishes spec routing, required-section shape, and link integrity for
the contract. Proof does not establish container runtime availability, GHCR
public visibility, image build behavior, or end-to-end packet generation from a
container.

## Open Questions

- Resolved by PR B: `runtime: container` with `version: latest` (or any mutable
  alias) is rejected rather than auto-resolved, so the recorded `tokmd-version`
  stays reproducible. A future change could add concrete-tag resolution for
  `latest` if a reproducible mechanism is agreed.
- Resolved by PR B: the `image` input is a repo reference (without tag); the tag
  is always derived from `version`. A full `repo:tag` reference is not accepted.
- Resolved by PR B: the Action runs the container via `docker run` with a
  workspace bind mount (`-v $GITHUB_WORKSPACE:$GITHUB_WORKSPACE -w
  $GITHUB_WORKSPACE`) through a PATH wrapper, so the existing composite
  `shell: bash` steps invoke `tokmd` unchanged.
- Whether a container-vs-binary artifact-equivalence smoke should become a
  required release-facing lane or remain a manual maintainer receipt.
- Whether `ub-review` downstream consumption should prefer the container runtime
  once verified, or continue defaulting to the binary runtime.
