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
| `image` | **present in `action.yml`; container path still gated** | `ghcr.io/effortlessmetrics/tokmd` | Container image reference (without tag) when `runtime: container`. The Action resolves `<image>:<normalized-version>` and reports it in the `runtime: container` error, but does not pull until the verification gate passes. |
| existing per-mode inputs | implemented | — | Unchanged; the runtime does not alter mode behavior. |

Input rules for the planned implementation:

- When `runtime` is not `binary` or `container`, the Action must fail with a
  clear error naming the received value. (Already implemented.)
- When `runtime: container` and `version` is a concrete version, the resolved
  image reference is `<image>:<normalized-version>` where the version is
  normalized to the published tag form (for example `1.14.0`, matching the
  publication GHCR tag vocabulary in `docs/specs/swarm-ghcr-image.md`).
- When `runtime: container` and `version` is `latest`, the Action must resolve a
  concrete published tag rather than pulling a mutable `latest`-style tag, so
  the recorded `tokmd-version` output is reproducible. The exact resolution
  mechanism is an open question below.
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

Adding the `image` input and enabling the `container` branch in `action.yml`
must keep the existing `runtime: container` rejection behavior until the
verification gate passes, then replace the hard error with the implemented pull
path. Any change to the `runtime` input vocabulary, the `image` input, the
container invocation contract, or the verification gate must update this spec
and `docs/packet-workflows.md` in the same change.

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
| `1.14.0` (and `1.14` / `1` / `latest`) | verified-public (2026-06-26) | not run | **pending** |

On **2026-06-26**, anonymous registry-API verification (no Docker on the
verification host) confirmed gate steps 1-4 and the registry-level portion of
step 5 for `1.14.0` and its `1.14` / `1` / `latest` aliases: an unauthenticated
pull token resolved every tag to HTTP 200 with one shared OCI image index
digest `sha256:bd214464…b914b096`, and the `linux/amd64` config blob was
fetched anonymously (public package; `image.revision` matches the `v1.14.0`
release tag commit, `image.version` is `1.14.0`). Receipt:
`target/publishing/ghcr-visibility-1.14.0.md`; ledger copy in
`docs/releases/1.14-ledger.md`.

Gate steps 6 (container `tokmd --version`) and 7 (mounted-repository `complete`
packet smoke) were **not run** because they require a Docker exec unavailable on
the verification host. The OCI `image.version` label is image-declared metadata,
not a runtime exec, so it does not discharge step 6. The container runtime for
`1.14.0` therefore stays **pending** under this gate: `action.yml` keeps the
`runtime: container` hard error until a Docker-capable host completes steps 6-7
for the tag. Public visibility being verified does not by itself make the
container runtime supported.

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

- How `runtime: container` with `version: latest` should resolve a concrete,
  reproducible published tag instead of pulling a mutable tag.
- Whether the `image` input should accept a full `repo:tag` reference or only a
  repo reference with the tag derived from `version`.
- Whether the Action should run the container via `docker run` with a workspace
  mount or via a `container:`-based composite step, and how that interacts with
  the existing composite `shell: bash` steps.
- Whether a container-vs-binary artifact-equivalence smoke should become a
  required release-facing lane or remain a manual maintainer receipt.
- Whether `ub-review` downstream consumption should prefer the container runtime
  once verified, or continue defaulting to the binary runtime.
