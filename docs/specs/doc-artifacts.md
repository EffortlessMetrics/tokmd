# Spec: Documentation Artifacts

- Status: draft
- Schema family, if any: `tokmd.doc_artifacts_check.v1`
- Related ADRs: `docs/adr/0000-adr-process.md`
- Related proof scopes: `project_truth_docs`, `proof_control_plane`, `jules_workspace`

## Contract

Documentation artifacts are the durable routing layer for tokmd planning,
behavior contracts, architecture decisions, implementation sequencing, active
agent state, and machine-checked policy.

The source-of-truth model in `docs/source-of-truth.md` defines the human-facing
roles. This spec defines the first machine-checkable contract for those
artifacts so `cargo xtask doc-artifacts --check` can verify
shape, links, and routing without judging prose quality.

The checker must be conservative. It should verify that artifacts are present,
linked, parseable, repo-relative, and routed to the right document family. It
must not decide whether an idea is good, whether a PR should merge, or whether
proof gates should be promoted.

## Inputs

The documentation artifact checker reads these repo paths:

- `.tokmd-spec/README.md`
- `.tokmd-spec/index.toml`
- paths indexed by `.tokmd-spec/index.toml`
- `docs/source-of-truth.md`
- `docs/proposals/**/*.md`
- `docs/specs/**/*.md`
- `docs/adr/**/*.md`
- `docs/plans/**/*.md`
- `.jules/goals/active.toml`
- `ci/proof.toml`
- `policy/*.toml`

The checker should ignore generated build output, downloaded artifacts, raw run
logs, and `.jules` run packets outside `.jules/goals/`.

## Artifact Families

### Proposals

Files under `docs/proposals/` explain exploratory rationale, alternatives, and
open questions before a direction becomes a contract.

The checker should require:

- a top-level Markdown heading;
- a status line using the local README vocabulary;
- no machine-policy claims that belong in `ci/proof.toml` or `policy/*.toml`.

### Specs

Files under `docs/specs/` define testable behavior, artifact shapes, proof
requirements, and accepted semantics.

The checker should require:

- a top-level Markdown heading;
- a status line using the local README vocabulary;
- `## Contract`;
- `## Proof Requirements`, unless the spec is explicitly marked as a draft
  with no implementation yet.

### ADRs

Files under `docs/adr/` define durable architecture, packaging, boundary, or
governance decisions.

The checker should require:

- ADR filenames to be either `README.md` or numbered `NNNN-*.md`;
- a top-level Markdown heading;
- a status line for non-README ADRs;
- no PR-by-PR implementation checklist as the primary content.

### Plans

Files under `docs/plans/` define sequencing, work packets, validation commands,
dependencies, and stop conditions.

The checker should require:

- a top-level Markdown heading;
- a status line using the local README vocabulary;
- `## Work Packets`;
- `## Validation`;
- `## Stop Conditions`.

### Active Agent State

`.jules/goals/active.toml` is the current machine-readable active-agent state.
It should stay small and link to durable human docs.

The checker should require:

- `schema = "tokmd.jules.active_goal.v1"`;
- a known `status`;
- non-empty `program` and `lane`;
- `[links]` entries that are repo-relative and exist;
- `[rules]` entries that do not promote proof gates or Codecov defaults unless
  a later ADR and policy explicitly allow that;
- `[stop_conditions]` entries that name concrete commands or observable queue
  state.

### Machine Policy

`ci/proof.toml` and files under `policy/` own machine-checkable rules. The
documentation artifact checker should defer policy semantics to the policy
checker that owns each file.

The checker should verify only routing-level facts:

- referenced policy files exist;
- source-of-truth documentation does not claim a policy file that is absent;
- doc-artifact rules do not conflict with `cargo xtask proof-policy --check`.

### Repo-Native Spec Namespace

`.tokmd-spec/README.md` defines the durable repo-native namespace and its
separation from tool-local state.

The checker should require:

- a top-level Markdown heading;
- `## Durable ownership`;
- `## External and awareness-only namespaces`;
- `## Source-of-truth chain`.

`.tokmd-spec/index.toml` is the index for durable artifacts in or linked from
the namespace.

The checker should require:

- `schema_version = "1.0"`;
- `repo = "tokmd"`;
- `namespace = ".tokmd-spec"`;
- each `[[artifact]]` entry to have non-empty `id`, `kind`, `path`, and
  `status`;
- each `[[lane]]` entry to have non-empty `id`, `path`, and `status` when lane
  entries are present;
- indexed paths to be repo-relative, existing paths;
- indexed paths to avoid tool-local namespaces such as `.codex/`, `.spec/`,
  `.claude/`, and `.jules/`;
- duplicate indexed IDs to fail the check.

## Outputs

The default checker output is human-readable text and a process exit code:

- exit `0` when all checked artifacts pass;
- non-zero exit when any required artifact shape, link, path, or routing rule
  fails.

The checker may also write an optional JSON receipt:

```bash
cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json
```

The JSON receipt uses schema `tokmd.doc_artifacts_check.v1`:

```json
{
  "schema": "tokmd.doc_artifacts_check.v1",
  "ok": true,
  "checked": {
    "required_docs": 1,
    "family_files": 11,
    "active_goals": 1
  },
  "errors": []
}
```

The receipt is visibility-only. It must not promote proof gates, enable default
Codecov upload, or change product behavior.

## Compatibility

The first checker must account for existing docs that predate this directory
layout. Top-level documents such as `docs/specification.md`,
`docs/review-packet.md`, and `docs/cockpit-proof-evidence.md` remain valid
even though new focused specs should prefer `docs/specs/`.

The checker must not require moving existing docs into the new directories. It
should enforce the shape of new source-of-truth artifacts without creating a
large documentation migration.

## Proof Requirements

The implementation PR for `cargo xtask doc-artifacts --check` should run:

```bash
cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan
cargo fmt-check
git diff --check
```

If the checker touches publish/package/export surfaces, also run:

```bash
cargo xtask publish-surface --json --verify-publish
```

## Open Questions

- Whether `docs/proposals/` should allow accepted proposals to remain in place
  forever or require an explicit link to the follow-on spec, ADR, or plan.
- Whether the first implementation should enforce every section listed in
  `policy/doc-artifacts.toml` or start with a reporting-only mode for older
  ADRs and specs that predate this source-of-truth stack.
