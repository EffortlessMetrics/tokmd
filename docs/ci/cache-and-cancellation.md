# CI cache + cancellation policy

## Cancellation

PR-facing workflows define a `concurrency` group with the shape:

```yaml
concurrency:
  group: <name>-${{ github.workflow }}-${{ github.event.pull_request.number || github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' && github.event.action == 'synchronize' }}
```

**Why the conditional?** With plain `cancel-in-progress: true`, label
add/remove on a PR cancels in-flight runs. That throws away work and
makes lane routing (PR 08, PR 12) painful — adding the `wasm` label on a
PR with `Wasm Compile & Test` already running just kills the run instead
of letting the new run start with the new label set.

The conditional cancels only when GitHub fires `synchronize` (a new
commit pushed). All other PR events — `labeled`, `unlabeled`, `opened`,
`reopened` — leave existing runs alone, and their replacement runs start
fresh with the new state.

Publication-only Nix workflows are different. They do not run on PR events.
`nix-full.yml` is commit-scoped for successful `CI` workflow runs on
`tokmd/main`, so a newer publication merge does not cancel an older Nix full
validation for a prior commit. These long side validations have explicit job
timeouts and are release/publication evidence, not the normal swarm workbench
gate. Treat an in-progress Nix full run as a release-readiness caveat, not as a
reason to block a green `tokmd-swarm` PR, a merge-commit publication import, or
the follow-up fast-forward back to swarm.

When checking current publication state, key Nix full runs by `headSha`, not
just by workflow name. Multiple in-progress `Nix Full Validation` runs can be
valid at the same time when they cover different publication commits. Report the
run for the current `tokmd/main` head separately from older commit-scoped runs,
and avoid treating an older in-progress run as evidence about the current
publication merge.

## Cache save policy

Every `Swatinem/rust-cache@v2` use sets:

```yaml
- uses: Swatinem/rust-cache@v2
  with:
    save-if: ${{ github.ref == 'refs/heads/main' }}
```

PRs **restore** caches but never **save** them. `main` is the only ref
that writes the canonical cache.

This avoids per-PR cache churn: every fork/branch was previously
producing its own cache entries that competed for the GitHub Actions
cache budget (10GB per repo by default), evicting useful caches in
seconds. Save-on-main means new PRs get a warm cache from main and
return without writing.

## Affected workflows

| Workflow | Cancel | Cache save policy |
|----------|--------|-------------------|
| `ci.yml` | sync-only | `save-if: main` on every cache use |
| `coverage.yml` | sync-only | `save-if: main` |
| `cockpit.yml` | sync-only | `save-if: main` |
| `proof-executor.yml` | sync-only | `save-if: main` |
| `proof-observation-collection.yml` | sync-only | `save-if: main` |
| `nix-full.yml` | commit-scoped side validation | n/a |
| `nix-macos.yml` | ref-scoped side validation | n/a |
