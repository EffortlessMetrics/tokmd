# Spec: Dual-Repo Repository Topology

- Status: active
- Schema family, if any: `tokmd.repo_graph.v1`
- Related ADRs: n/a
- Related specs: `docs/specs/swarm-ghcr-image.md`,
  `docs/specs/publishing-evidence.md`
- Related proof scopes: `project_truth_docs`, `proof_control_plane`

## Contract

`EffortlessMetrics/tokmd-swarm` and `EffortlessMetrics/tokmd` must operate as
one Git commit graph with two repository roles.

`tokmd-swarm` is the active development workbench. Normal feature, docs, test,
and maintenance changes branch from `tokmd-swarm/main`, open PRs against
`tokmd-swarm/main`, and land by squash merge after local proof and hosted
workbench checks.

`tokmd` is the publication repository. It owns release, publish, signing, tag,
Docker, `v1` alias, and release-record mutation. Routine swarm work is imported
into `tokmd` by a deliberate merge-commit PR, not by squash merge or orphan
content sync.

After each publication import, `tokmd-swarm/main` must fast-forward to the exact
publication merge commit. This keeps GitHub ahead/behind counts meaningful and
preserves the squashed swarm commits as second-parent history behind each
publication merge.

## Inputs

The topology contract is evaluated from these inputs:

- the current `EffortlessMetrics/tokmd:main` commit;
- the current `EffortlessMetrics/tokmd-swarm:main` commit;
- the merge base between those two refs;
- publication PR merge method;
- swarm PR merge method;
- repository-guarded workflow behavior.

The operational runbook for these inputs is `docs/ci/swarm-routing.md`.

## Outputs

The primary machine-readable topology receipt is produced by:

```bash
cargo xtask repo-graph \
  --publication public/main \
  --swarm origin/main \
  --expect aligned \
  --json target/repo-graph/alignment.json
```

The receipt uses schema `tokmd.repo_graph.v1` and must report the relation,
publication head, swarm head, merge base, ahead counts, and recommended next
action for the requested expectation.

The steady-state aligned output means:

```text
relation = aligned
publication_ahead = 0
swarm_ahead = 0
```

During normal operation, temporary non-aligned relations are allowed only when
they match the current step:

- `swarm-ahead` before a deliberate publication import;
- `publication-ahead` after a publication import and before swarm fast-forward;
- aligned after the publication merge commit has been pushed back to swarm.

## Compatibility

This contract replaces orphan-import content sync as the normal operating
model. Orphan content sync may be used only as an explicit emergency repair and
must not become the routine publication path.

Swarm-aware files may live in shared history when their behavior is guarded by
repository conditions. Publication-only workflows must not run release, publish,
signing, tag, Docker, alias, Nix package, or full-publication validation
behavior in `tokmd-swarm`. This includes CI package gates such as
`Nix PR Package Gate`, not only separate Nix full-validation workflows.
Swarm-routed workbench workflows must not become required publication release
gates unless a future ADR and policy update explicitly move that boundary.

GHCR registry ownership is split by repository role:

| Image | Repository | Status |
| --- | --- | --- |
| `ghcr.io/effortlessmetrics/tokmd` | publication `tokmd` | Supported public secondary runtime when release ledgers record `verified-public` (see `docs/specs/publishing-evidence.md`). |
| `ghcr.io/effortlessmetrics/tokmd-swarm` | workbench `tokmd-swarm` | Publish workflow on swarm `main`; **verified-public** for `:main` (2026-06-24, bootstrap amd64-only). Workbench/experimental tier. See `docs/specs/swarm-ghcr-image.md`. |

Publication release workflows must not push the swarm image name. Swarm publish
workflows must not push publication semver tags or mutate publication release
records.

This spec does not change product receipts, public CLI behavior, release
workflow behavior, proof promotion, Codecov defaults, AST behavior, or
evidencebus runtime behavior.

## Proof Requirements

For a routine swarm PR, the proof is:

```bash
cargo xtask repo-graph \
  --publication public/main \
  --swarm origin/main \
  --expect aligned \
  --json target/repo-graph/agent-start.json
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan
git diff --check
```

For a publication import, prove the sequence with:

```bash
cargo xtask repo-graph \
  --publication public/main \
  --swarm origin/main \
  --expect swarm-ahead \
  --json target/repo-graph/pre-publication.json
```

Then merge the publication PR in `tokmd` with a merge commit, fast-forward
`tokmd-swarm/main` to the publication merge commit, and prove:

```bash
cargo xtask repo-graph \
  --publication public/main \
  --swarm origin/main \
  --expect aligned \
  --json target/repo-graph/post-fast-forward.json
```

For changes to this spec or the topology runbook, also run:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
```

## Open Questions

- When to add `linux/arm64` for swarm GHCR after bootstrap amd64-only publish
  (see `docs/specs/swarm-ghcr-image.md`).
- Whether future publication-import automation should write a dedicated
  closeout receipt that bundles the pre-publication, merge-commit, and
  post-fast-forward `repo-graph` receipts.
- Whether branch-protection settings should be mirrored into a checked policy
  file once GitHub settings drift becomes a recurring source of topology risk.
