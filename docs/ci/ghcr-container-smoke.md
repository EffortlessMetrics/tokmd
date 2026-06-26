# GHCR Container Smoke

`GHCR Container Smoke` (`.github/workflows/ghcr-container-smoke.yml`) is a manual
verification lane that discharges gate steps 6-7 of the
[packet GHCR runtime spec](../specs/packet-ghcr-runtime.md#verification-gate) for
a stable tag of the **publication** image
`ghcr.io/effortlessmetrics/tokmd`:

6. the container reports the expected `tokmd --version`;
7. the container generates a `complete` evidence packet against a mounted sample
   repository.

It is a `workflow_dispatch`-only lane. It only **pulls** and **runs** an already
published image. It performs no release mutation, no push, and no tag or alias
change. This lane is the runtime-exec evidence (gate steps 6-7) that a tag must
pass before it is added to the `runtime: container` supported-tag set in
`action.yml`; the lane itself does not modify `action.yml`.

## When To Run

Run it after a stable release when the publication GHCR image is expected to be
public, and you want runtime-exec evidence (not just registry visibility) before
calling the container runtime supported for that tag. Public-visibility gate
steps 1-5 are verified separately and recorded in
[publishing evidence](../publishing-evidence.md#post-release-ghcr-visibility-checks)
and the matching release ledger
(for example [1.14 ledger](../releases/1.14-ledger.md)).

## How To Run

### From GitHub (workflow_dispatch)

```bash
gh workflow run "GHCR Container Smoke" \
  --repo EffortlessMetrics/tokmd-swarm \
  --field version=1.14.0 \
  --field image=ghcr.io/effortlessmetrics/tokmd \
  --field preset=bun-ub
```

The dispatch trigger is only registered once the workflow file is on the
default branch, so merge the workflow before the first dispatch.

On success the lane writes a runtime-exec receipt to
`target/publishing/ghcr-visibility-<version>-runtime-exec.md`, appends it to the
job summary, and uploads it as the `ghcr-container-smoke-receipt-<version>`
artifact. Copy the receipt outcome into the release ledger and update the spec
verification status table once a green run exists.

### Locally (equivalent steps)

The lane mirrors the manual recipe from
[publishing evidence](../publishing-evidence.md#post-release-ghcr-visibility-checks).
A logged-out Docker client is required so the pull is genuinely anonymous:

```bash
VERSION=1.14.0
IMAGE=ghcr.io/effortlessmetrics/tokmd
DOCKER_CONFIG="$(mktemp -d)"

# gate step 5 (pull) + step 6 (version)
docker --config "${DOCKER_CONFIG}" pull "${IMAGE}:${VERSION}"
docker --config "${DOCKER_CONFIG}" run --rm "${IMAGE}:${VERSION}" --version

# gate step 7: complete packet against a mounted git fixture
FIXTURE="$(mktemp -d)"
mkdir -p "${FIXTURE}/src"
printf 'pub fn add(a: i32, b: i32) -> i32 { a + b }\n' > "${FIXTURE}/src/lib.rs"
git -C "${FIXTURE}" init -q
git -C "${FIXTURE}" -c user.email=ci@tokmd.dev -c user.name=tokmd-ci add .
git -C "${FIXTURE}" -c user.email=ci@tokmd.dev -c user.name=tokmd-ci commit -q -m fixture

docker --config "${DOCKER_CONFIG}" run --rm \
  --user "$(id -u):$(id -g)" -e HOME=/repo \
  -v "${FIXTURE}:/repo" -w /repo \
  "${IMAGE}:${VERSION}" \
  packet generate --preset bun-ub --base HEAD --head HEAD --no-syntax src

jq -r '.schema, .status' "${FIXTURE}/sensors/tokmd/manifest.json"
rm -rf "${DOCKER_CONFIG}" "${FIXTURE}"
```

The smoke passes `--no-syntax` so the packet contract is `complete` without
depending on the image carrying the optional `ast` syntax feature. Syntax
evidence remains advisory; its absence degrades a packet to `partial`, not
`failed`.

## Claim Boundary

A green run proves only that, for the exact tag verified:

- the publication image is anonymously pullable;
- the container `tokmd` runs and reports the expected version;
- the container produces a `complete` evidence packet contract from a mounted
  repository.

It does **not** enable or verify the `runtime: container` Action path, does not
prove anything about any other tag, and does not prove UB presence/absence,
memory safety, reachability, or release readiness. Each new stable tag re-enters
the gate before its container runtime is called supported. The lane is
advisory and `workflow_dispatch`-only; it is registered in
[`policy/ci-lane-whitelist.toml`](../../policy/ci-lane-whitelist.toml) as
`ghcr_container_smoke`.
