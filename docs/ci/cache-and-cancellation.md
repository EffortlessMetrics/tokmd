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
makes label and risk-pack routing painful — adding the `wasm` label on a
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
publication merge. When a failed Nix full run is rerun, also record the run
`attempt`, current `status`, and `conclusion`; GitHub keeps the rerun under the
same run ID, so an in-progress later attempt is different evidence from the
earlier failed attempt.


## Codex CI-efficiency compatibility invariants

When drafting CI-efficiency PRs, treat this section as hard compatibility policy.

### 1) Concurrency semantics are lane-specific

Do not apply one concurrency recipe across every workflow. A CI-efficiency PR
must preserve the cancellation model for the lane it edits:

| Workflow class | Expected cancellation model |
| --- | --- |
| Core PR workflows | Cancel superseded `synchronize` runs, but do not cancel label/open/reopen runs. |
| `ci.yml` route + gate | The routed Rust Small frontdoor was retired in phase 3 (#299); its routing folded into the `Route CI runner` job of `ci.yml`. Cancel superseded `route`/`Tokmd Rust Result` runs for new PR commits, and keep `Tokmd Rust Result` as the required check. The historical routing contract is preserved in `docs/ci/routed-ci-policy.md`. |
| Publication side validation | Treat runs as commit-scoped evidence keyed by `headSha`; do not collapse older publication evidence into the newest commit. |

Do not submit generic efficiency edits that flip workflows to plain
`cancel-in-progress: true`, plain `false`, or a new group key without explaining
which lane class is changing and why the evidence semantics still hold.

### 2) Change classification before lane selection

Do not treat every changed path as Rust input. Control-plane and metadata edits
must route to light validation paths unless mixed with real Rust/build/test
changes.

Paths that are docs/control-plane light by default:

- `docs/**`, `*.md`, `README*`, `CHANGELOG*`, `SECURITY*`, `CONTRIBUTING*`
- `policy/**`, `plans/**`, `badges/**`, `AGENTS.md`
- `.github/CODEOWNERS`, `.github/dependabot.yml`, PR templates
- `.codex/campaigns/**`, `docs/tracking/**`, `ci/hardware/**` receipts
- `.rails/**`, `.uselesskey/**`

Workflow edits are special:

- `.github/workflows/**` must not be routed as docs-light.
- Route workflow-only changes to minimal hosted workflow/YAML validation,
  not full Rust CI unless required.

### 3) Default PR routing policy

Default PR CI proposals should classify first, then choose the cheapest truthful
lane the current required-check policy can support:

- docs/control-plane-only → avoid Rust compile only after the aggregate required
  check still has truthful replacement evidence
- workflow-only → hosted workflow validation only
- Rust/build/test changes → routed Rust-small
- hardware/GPU/receipt-only → syntax/receipt validation only
- unknown or mixed → Rust-small (not full CI)

Full CI requires explicit intent (labels, manual dispatch, merge queue, release,
main push, or schedule according to workflow policy).

### 4) Hosted fallback guardrails

Do not silently replace a self-hosted Rust-small route with a full
GitHub-hosted equivalent.

- Fork PRs may use a tiny hosted safe lane.
- Missing runner readiness, transient token failures, or no idle runner must not
  automatically trigger a 75–120 minute hosted lane.
- Require explicit labels/inputs for expensive hosted fallback (for example
  `full-ci`, `allow-github-hosted`, `ci-budget-ack`).

### 5) Artifact cost policy

Do not upload large receipts/JUnit/log artifacts on every default PR run unless
merge policy requires them.

- Prefer upload-on-failure.
- Keep retention short (typically 3–7 days).
- Keep policy-required receipts small, and skip uploads for docs/control-plane
  only paths whenever possible.

### 6) Required validation for CI-only PRs

Every CI-efficiency PR must include evidence for:

- `git diff --check`
- YAML parse check for each edited workflow
- classification dry-run or shell-unit coverage for:
  - docs-only
  - `.rails/**`
  - `.uselesskey/**`
  - workflow-file change
  - Rust-file change
  - mixed docs + Rust
- explicit confirmation that each edited workflow preserved its lane-specific
  concurrency semantics, unless the change intentionally updates that policy

### Reviewer reject checklist

Reject CI-efficiency PRs unless all answers are "yes":

1. Edited workflows preserve their documented concurrency semantics.
2. Metadata/control-plane-only edits avoid unnecessary Rust CI, or the PR
   explains why the current required aggregate still runs it.
3. Workflow edits are kept out of docs-light routing.
4. No silent expensive hosted fallback was introduced.
5. The change reduces actual billable work instead of shifting cost.

## Run status polling

For agent-run CI triage, prefer bounded status snapshots over long
`gh run watch` sessions. `gh run watch` polls every few seconds and can exhaust
the GitHub API quota during slow matrix jobs, which turns a CI follow-up into an
authentication or rate-limit problem.

Use a single status read, then sleep before the next read when the run is still
active:

```bash
gh run view <run-id> \
  --repo EffortlessMetrics/tokmd \
  --json attempt,status,conclusion,headSha,jobs,url
```

or for swarm:

```bash
gh run view <run-id> \
  --repo EffortlessMetrics/tokmd-swarm \
  --json attempt,status,conclusion,headSha,jobs,url
```

If the run is still `in_progress`, record active jobs and wait a bounded
interval before checking again. Only fetch logs after GitHub reports the run or
job as terminal; logs for active runs may be unavailable or incomplete. Rerun
failed jobs only after the failed steps show an infrastructure-only failure,
such as checkout/auth, and record the rerun attempt separately from the original
attempt.

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
