# Droid Action Migration Rollout

Standardized rollout design for migrating EffortlessMetrics repositories from unsafe `Factory-AI/droid-action` to safe, standardized `EffortlessMetrics/droid-action-safe@01e76b659e4b1e5f23feedc8cfabf8dc14c7485f`.

## Executive Summary

This design ensures no direct Factory-AI/droid-action use for secrets-backed BYOK workflows. The rollout proceeds in phases: patch existing unsafe refs, converge to baseline with MiniMax BYOK, then move shared plumbing into org-level reusable workflows.

## Operating Invariants

```text
✓ No direct Factory-AI/droid-action use for secrets-backed BYOK workflows.
✓ No raw $HOME/.factory/** upload.
✓ No raw droid-prompts/** upload.
✓ No mutable action refs (no @main, no @v5).
✓ No fork PR secret execution.
✓ upload_debug_artifacts: false in all standard runs.
✓ show_full_output: false in all standard runs.
```

## Design Layers

```
Layer 1: Safe action
EffortlessMetrics/droid-action-safe@01e76b659e4b1e5f23feedc8cfabf8dc14c7485f

Layer 2: Repo workflow wrapper
.github/workflows/droid-review.yml
.github/workflows/droid.yml
.github/workflows/droid-security-scan.yml

Layer 3: Repo-local review context
AGENTS.md
.factory/skills/review-guidelines/SKILL.md
.factory/rules/droid-review.md
docs/agent-context/review-invariants.md
docs/agent-context/droid-smoke-tests.md

Layer 4: Later org reusable workflow plumbing
EffortlessMetrics/.github/.github/workflows/droid-*.yml
```

## Rollout Phases

### Phase 0 — Security Closeout (Manual)

Before adding MiniMax key to more repos:

1. Rotate exposed MiniMax Token Plan key
2. Update MINIMAX_API_KEY in GitHub org/repo secrets
3. Confirm FACTORY_API_KEY is still valid
4. Keep MINIMAX_API_KEY scoped only to repos in the rollout batch

**Critical**: Do not make MINIMAX_API_KEY org-wide unless all repos are intended to participate.

### Phase 1 — Emergency Safety Patch (Per-Repo PR)

**Goal**: Stop unsafe artifact behavior without broad behavior churn.

For each repo that references `Factory-AI/droid-action`, replace:

```yaml
uses: Factory-AI/droid-action@...
```

with:

```yaml
uses: EffortlessMetrics/droid-action-safe@01e76b659e4b1e5f23feedc8cfabf8dc14c7485f # based on Factory-AI/droid-action v5; raw debug artifact upload disabled
```

and add:

```yaml
with:
  upload_debug_artifacts: false
```

**Priority order**:
1. Mutable refs first: `Factory-AI/droid-action@main` or `@v5`
2. Then pinned upstream SHAs

**Scope**: Touch only Droid workflows unless a repo policy file must allow the new action ref. Do not change workflow permissions, triggers, or behavior unless required for safety.

**PR structure**:
- Title: `ci: use safe Droid action`
- Include clear statement: "Normal Droid runs should not upload raw `droid-review-debug-<run_id>` artifacts."
- Validation: Repo static checks + same-repo smoke PR + no raw artifact upload

### Phase 2 — Baseline Convergence (Per-Repo PR)

**Goal**: Bring behavior to ripr baseline after all existing Droid installs are safe.

After Phase 1 is complete in a repo, add:

```text
✓ MiniMax BYOK through ~/.factory/settings.local.json
✓ custom:MiniMax-M2.7-0 model selection
✓ review_depth: shallow
✓ show_full_output: false
✓ upload_debug_artifacts: false (already set in Phase 1)
✓ same-repo guard for automatic PR review
✓ trusted-actor guard for manual @droid
```

**PR structure**:
- Title: `ci: align Droid review baseline`
- May change behavior — keep separate from Phase 1 where possible
- Add repo-local review guidance

### Phase 3 — Reusable Workflows (Org-Level)

After a few repos prove baseline is working:

1. Create reusable workflows in `EffortlessMetrics/.github`:
   - `.github/workflows/droid-review-reusable.yml`
   - `.github/workflows/droid-tag-reusable.yml`
   - `.github/workflows/droid-security-scan-reusable.yml`

2. These reusable workflows own:
   - Checkout SHA
   - MiniMax BYOK settings.local.json bridge
   - Safe action SHA
   - upload_debug_artifacts: false
   - custom:MiniMax-M2.7-0
   - review_depth: shallow
   - show_full_output: false
   - Factory action inputs

3. Target repos still own:
   - on: triggers
   - permissions
   - if: guards
   - concurrency
   - schedule
   - repo-local guidance
   - repo validation commands

**Important**: Pin reusable workflow by commit SHA or protected tag. Do not use unprotected branch ref for broad rollout.

## Target Repositories

### Batch 1 — Mutable Refs (Higher Risk)

Repos using `@main` or `@v5` (highest drift risk):

- **OpenRacing**
- **adze**
- **SwiftMTP-dev**
- **SwiftMailSort**
- **shiplog**

### Batch 2 — Pinned But Unsafe Upstream Refs

Repos using direct upstream pinned SHAs (less drift-prone):

- **perl-lsp**
- **pkm-python**

### Batch 3 — New Installs

After Batches 1 and 2 are clean, add Droid to new repos in groups of 3–5, then 10–20.

## Workflow Templates

### `.github/workflows/droid-review.yml` (Auto Review)

```yaml
name: Droid Auto Review

on:
  pull_request:
    types: [opened, synchronize, ready_for_review, reopened]

concurrency:
  group: droid-review-${{ github.repository }}-${{ github.event.pull_request.number }}
  cancel-in-progress: false

jobs:
  droid-review:
    if: |
      github.event.pull_request.head.repo.full_name == github.repository &&
      !contains(github.event.pull_request.title, '[skip-review]')

    runs-on: ubuntu-latest

    env:
      MINIMAX_API_KEY: ${{ secrets.MINIMAX_API_KEY }}

    permissions:
      contents: write
      pull-requests: write
      issues: write
      id-token: write
      actions: read

    steps:
      - name: Checkout repository
        uses: actions/checkout@93cb6efe18208431cddfb8368fd83d5badbf9bfd # v5
        with:
          fetch-depth: 1

      - name: Configure MiniMax BYOK for Factory Droid
        shell: bash
        run: |
          mkdir -p "$HOME/.factory"
          cat > "$HOME/.factory/settings.local.json" <<'JSON'
          {
            "customModels": [
              {
                "displayName": "MiniMax-M2.7",
                "model": "MiniMax-M2.7",
                "baseUrl": "https://api.minimax.io/anthropic",
                "apiKey": "${MINIMAX_API_KEY}",
                "provider": "anthropic",
                "maxOutputTokens": 64000,
                "noImageSupport": true,
                "extraArgs": {
                  "temperature": 1
                }
              }
            ]
          }
          JSON

      - name: Run Droid Auto Review with MiniMax M2.7 BYOK
        uses: EffortlessMetrics/droid-action-safe@01e76b659e4b1e5f23feedc8cfabf8dc14c7485f # based on Factory-AI/droid-action v5; raw debug artifact upload disabled
        with:
          factory_api_key: ${{ secrets.FACTORY_API_KEY }}
          upload_debug_artifacts: false

          automatic_review: true
          automatic_security_review: true

          review_depth: shallow
          review_model: "custom:MiniMax-M2.7-0"
          security_model: "custom:MiniMax-M2.7-0"

          security_severity_threshold: high
          security_block_on_critical: true
          security_block_on_high: false

          include_suggestions: true
          show_full_output: false
```

**Notes**:
- Keep `contents: write` for auto review.
- Keep draft review enabled unless repo has clear reason not to.
- Do not switch to `contents: read` without proof PR showing Factory flow works.

### `.github/workflows/droid.yml` (Manual @droid)

```yaml
name: Droid Tag

on:
  issue_comment:
    types: [created]
  pull_request_review_comment:
    types: [created]
  issues:
    types: [opened, assigned]
  pull_request_review:
    types: [submitted]
  pull_request:
    types: [opened, edited]

jobs:
  droid:
    if: |
      (
        github.event_name == 'issue_comment' &&
        contains(github.event.comment.body, '@droid') &&
        contains(fromJSON('["OWNER","MEMBER","COLLABORATOR"]'), github.event.comment.author_association)
      ) ||
      (
        github.event_name == 'pull_request_review_comment' &&
        contains(github.event.comment.body, '@droid') &&
        contains(fromJSON('["OWNER","MEMBER","COLLABORATOR"]'), github.event.comment.author_association)
      ) ||
      (
        github.event_name == 'pull_request_review' &&
        contains(github.event.review.body, '@droid') &&
        contains(fromJSON('["OWNER","MEMBER","COLLABORATOR"]'), github.event.review.author_association)
      ) ||
      (
        github.event_name == 'issues' &&
        (contains(github.event.issue.body, '@droid') || contains(github.event.issue.title, '@droid')) &&
        contains(fromJSON('["OWNER","MEMBER","COLLABORATOR"]'), github.event.issue.author_association)
      ) ||
      (
        github.event_name == 'pull_request' &&
        github.event.pull_request.head.repo.full_name == github.repository &&
        (contains(github.event.pull_request.body, '@droid') || contains(github.event.pull_request.title, '@droid')) &&
        contains(fromJSON('["OWNER","MEMBER","COLLABORATOR"]'), github.event.pull_request.author_association)
      )

    runs-on: ubuntu-latest

    env:
      MINIMAX_API_KEY: ${{ secrets.MINIMAX_API_KEY }}

    permissions:
      contents: read
      pull-requests: write
      issues: write
      id-token: write
      actions: read

    steps:
      - name: Checkout repository
        uses: actions/checkout@93cb6efe18208431cddfb8368fd83d5badbf9bfd # v5
        with:
          fetch-depth: 1

      - name: Configure MiniMax BYOK for Factory Droid
        shell: bash
        run: |
          mkdir -p "$HOME/.factory"
          cat > "$HOME/.factory/settings.local.json" <<'JSON'
          {
            "customModels": [
              {
                "displayName": "MiniMax-M2.7",
                "model": "MiniMax-M2.7",
                "baseUrl": "https://api.minimax.io/anthropic",
                "apiKey": "${MINIMAX_API_KEY}",
                "provider": "anthropic",
                "maxOutputTokens": 64000,
                "noImageSupport": true,
                "extraArgs": {
                  "temperature": 1
                }
              }
            ]
          }
          JSON

      - name: Run Droid Exec with MiniMax M2.7 BYOK
        uses: EffortlessMetrics/droid-action-safe@01e76b659e4b1e5f23feedc8cfabf8dc14c7485f # based on Factory-AI/droid-action v5; raw debug artifact upload disabled
        with:
          factory_api_key: ${{ secrets.FACTORY_API_KEY }}
          upload_debug_artifacts: false

          review_depth: shallow
          review_model: "custom:MiniMax-M2.7-0"
          security_model: "custom:MiniMax-M2.7-0"
          show_full_output: false
```

**Notes**:
- Manual `@droid` must stay trusted-actor gated.
- `contents: read` is correct here (no auto-write).
- No arbitrary public comments can trigger secrets-backed jobs.

### `.github/workflows/droid-security-scan.yml` (Scheduled + Manual)

```yaml
name: Droid Security Scan

on:
  workflow_dispatch:
  schedule:
    - cron: "0 8 * * 1"

concurrency:
  group: droid-security-scan-${{ github.repository }}
  cancel-in-progress: false

jobs:
  droid-security-scan:
    runs-on: ubuntu-latest

    env:
      MINIMAX_API_KEY: ${{ secrets.MINIMAX_API_KEY }}

    permissions:
      contents: write
      pull-requests: write
      issues: write
      id-token: write
      actions: read

    steps:
      - name: Checkout repository
        uses: actions/checkout@93cb6efe18208431cddfb8368fd83d5badbf9bfd # v5
        with:
          fetch-depth: 1

      - name: Configure MiniMax BYOK for Factory Droid
        shell: bash
        run: |
          mkdir -p "$HOME/.factory"
          cat > "$HOME/.factory/settings.local.json" <<'JSON'
          {
            "customModels": [
              {
                "displayName": "MiniMax-M2.7",
                "model": "MiniMax-M2.7",
                "baseUrl": "https://api.minimax.io/anthropic",
                "apiKey": "${MINIMAX_API_KEY}",
                "provider": "anthropic",
                "maxOutputTokens": 64000,
                "noImageSupport": true,
                "extraArgs": {
                  "temperature": 1
                }
              }
            ]
          }
          JSON

      - name: Run Droid scheduled security scan
        uses: EffortlessMetrics/droid-action-safe@01e76b659e4b1e5f23feedc8cfabf8dc14c7485f # based on Factory-AI/droid-action v5; raw debug artifact upload disabled
        with:
          factory_api_key: ${{ secrets.FACTORY_API_KEY }}
          upload_debug_artifacts: false
          security_scan_schedule: true
          security_scan_days: 7
          security_model: "custom:MiniMax-M2.7-0"
          security_severity_threshold: medium
          security_block_on_critical: true
          security_block_on_high: false
          show_full_output: false
```

**Notes**:
- Keep `contents: write` because security scan/report flows may need it.
- Weekly schedule at Monday 8am UTC.
- Manual dispatch always available.

## Repo-Local Review Guidance

Each repo should have minimum local context for Droid to produce useful reviews.

### AGENTS.md

High-level guidance for AI agents working on this repo. Should address:
- Agent capabilities and constraints
- Review philosophy
- Special considerations for this codebase

### `.factory/skills/review-guidelines/SKILL.md`

Factory-native skill definition for review guidance. Contains:
- Structured review heuristics
- Finding prioritization
- Evidence standards

### `.factory/rules/droid-review.md`

Droid-specific workflow rules:
- No naked LGTM
- No arbitrary comment cap
- No extra @mentions in Droid-generated review body
- Actionable findings are repair packets
- Clean reviews include inspection record
- Evidence is split into Observed / Reported / Not verified

### `docs/agent-context/review-invariants.md`

Repository-specific invariants Droid must respect:
- Arch rules
- Permission boundaries
- Test expectations

### `docs/agent-context/droid-smoke-tests.md`

Smoke test checklist for validating Droid in this repo:
- Expected false-negative rate
- Known blind spots
- Validation procedures

## Finding and Clean Review Templates

### Finding Shape

```
[P0|P1|P2] Short title

Failure mode:
Why here:
Fix direction:
Validation:
Confidence:
```

### Clean Review Shape

```
No actionable findings emitted.

Inspected surfaces:
Checks performed:
Why no comments:
Residual risk:
Validation signal:
  Observed:
  Reported:
  Not verified:
```

## Validation Strategy

### Static Validation

For each repo, use existing checks:

```bash
# Examples; adapt per repo
cargo xtask check-workflows
cargo xtask check-pr
npm test
pnpm test
pytest
swift test
go test ./...
```

Also search for bad refs:

```bash
rg "Factory-AI/droid-action|droid-action@main|droid-action@v5|upload_debug_artifacts: true|show_full_output: true" .github docs .factory AGENTS.md
```

**Expected**: No Factory-AI/droid-action refs, no mutable refs, no debug artifacts, no verbose output.

### Live Smoke Validation

For each repo after workflow deployment:

1. Open a same-repo draft PR
2. Confirm Droid Auto Review starts
3. Confirm it runs with `custom:MiniMax-M2.7-0`
4. Confirm no raw artifact named `droid-review-debug-<run_id>` is uploaded
5. Confirm clean review uses inspection-record wording
6. Comment `@droid review` as OWNER/MEMBER/COLLABORATOR
7. Comment `@droid security` as OWNER/MEMBER/COLLABORATOR
8. Trigger Droid Security Scan manually if installed
9. Confirm MiniMax usage appears in provider dashboard

**Expected artifact state**: No Droid debug artifacts in normal runs.

## Acceptance Criteria for Broad Rollout

Before expanding beyond pilot repos:

```text
✓ ripr safe action smoke is green after MiniMax key rotation
✓ At least 3 pilot repos use the safe action SHA
✓ No pilot repo uploads raw Droid debug artifacts
✓ Manual @droid review works in at least 2 repos
✓ Manual @droid security works in at least 2 repos
✓ Scheduled/manual security scan works in at least 1 repo
✓ MiniMax usage is visible and expected
✓ No repo uses Factory-AI/droid-action directly for BYOK
✓ No repo uses droid-action@main or droid-action@v5
```

## Repo State Handling

### If Repo Has Existing Droid

Do not rerun `/install-code-review`. Patch in place:

1. Replace direct Factory-AI/droid-action refs
2. Add `upload_debug_artifacts: false`
3. Add MiniMax BYOK step if missing
4. Add model inputs if missing
5. Add same-repo guard if missing
6. Add trusted-actor guard if missing
7. Add security scan workflow if repo is in pilot batch
8. Add or trim repo-local Droid guidance

### If Repo Has No Droid

Add all three workflow files and minimal guidance.

### If Repo Has Workflow Policy

Update allowlist/shell-budget policy to allow:
- BYOK heredoc `run:` block
- Safe action ref: `EffortlessMetrics/droid-action-safe@01e76b659e4b1e5f23feedc8cfabf8dc14c7485f`

## Implementation Checklist

- [ ] Phase 0: Security closeout (manual)
  - [ ] Rotate exposed MiniMax key
  - [ ] Update GitHub secrets
  - [ ] Confirm FACTORY_API_KEY validity
  - [ ] Scope MINIMAX_API_KEY to pilot batch

- [ ] Phase 1: Safety patch for Batch 1 repos
  - [ ] OpenRacing
  - [ ] adze
  - [ ] SwiftMTP-dev
  - [ ] SwiftMailSort
  - [ ] shiplog

- [ ] Phase 1: Safety patch for Batch 2 repos
  - [ ] perl-lsp
  - [ ] pkm-python

- [ ] Phase 2: Baseline convergence for Batch 1 + 2
  - [ ] MiniMax BYOK configuration
  - [ ] Model selection (custom:MiniMax-M2.7-0)
  - [ ] Review depth and output settings
  - [ ] Guards and permissions review
  - [ ] Repo-local guidance

- [ ] Phase 3: Org-level reusable workflows
  - [ ] droid-review-reusable.yml
  - [ ] droid-tag-reusable.yml
  - [ ] droid-security-scan-reusable.yml
  - [ ] Pin by SHA in EffortlessMetrics/.github

- [ ] Smoke testing for all repos

- [ ] Final validation against acceptance criteria

## References

- ripr main after #467: Safe baseline reference implementation
- EffortlessMetrics/droid-action-safe: Safe action fork with debug artifacts disabled
- CLAUDE.md: Runtime-specific notes and workflow commands
