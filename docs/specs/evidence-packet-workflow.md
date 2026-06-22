# Spec: Evidence Packet Workflow

- Status: active
- Schema family, if any: `tokmd.evidence-packet/v1` (`docs/evidence-packet.schema.json`);
  optional syntax receipts use `tokmd.syntax_receipts.v1`
- Related ADRs:
- Related proof scopes: `tokmd_cli`, `tokmd_context_handoff`, `project_truth_docs`,
  `doc_artifacts_policy`

## Contract

A PR evidence packet is a bounded, reproducible directory under `sensors/tokmd/`
that indexes scoped `tokmd` receipts for high-risk review, coding-agent
handoff, and review-bot consumption.

The packet is a review optic. It packages what changed, which paths were in
scope, which artifacts were produced, whether the packet is complete or
degraded, advisory first-read hints, reproduction commands, and explicit
non-claims.

The packet is not a merge verdict, CI proof result, undefined-behavior
detector, memory-safety proof, release gate, cockpit review packet, or policy
promotion surface. It does not prove public reachability, safety, undefined
behavior presence or absence, or merge readiness.

`tokmd evidence-packet` is the manifest writer and verifier for the packet
directory. It indexes existing receipts; it does not replace `tokmd analyze`,
`tokmd context`, or `tokmd syntax`.

The planned thin orchestration command (`tokmd packet generate`) and dedicated
GitHub Action step are workflow conveniences described in
`docs/packet-workflows.md`. This spec owns the packet shape, producer rules,
consumer rules, support model, and verifier semantics regardless of whether
generation is manual, orchestrated locally, or run from CI.

## Inputs

Packet generation consumes explicit operator or workflow inputs:

| Input | Owner | Used for |
| --- | --- | --- |
| Base ref | Operator or workflow | Effort delta, manifest metadata, ref validation. |
| Head ref | Operator or workflow | Effort delta, manifest metadata, ref validation. |
| Scoped paths | Operator or workflow | Analyze, context, syntax, and manifest path scope. |
| Analysis preset | Operator or workflow | Default `bun-ub` for UB-review packets. |
| Pre-generated receipts | `tokmd analyze`, `tokmd context`, optional `tokmd syntax` | Source evidence indexed by the manifest. |
| Optional artifact path overrides | `tokmd evidence-packet` flags | Non-default artifact locations. |
| Git repository context | Host checkout | Base/head ref resolution when the `git` feature is enabled. |

Input paths recorded in `manifest.json` must be repo-relative with forward
slashes. The workflow must not depend on hidden local paths, downloaded CI
logs, credentials, or operator memory.

Producers must use the same base ref, head ref, preset, and path scope across
every receipt before writing `manifest.json`.

## Outputs

### Directory layout

The canonical packet root is `sensors/tokmd/` relative to the repository root
or workflow checkout root:

```text
sensors/tokmd/
  manifest.json       # required packet index
  analyze.md          # required human-first analysis summary
  analyze.json        # required machine-readable analysis receipt
  context.md          # required context budget audit
  syntax.json         # optional syntax receipt packet
```

`manifest.json` is the first file consumers should open. Receipt files remain
the authoritative evidence; the manifest records paths, status, warnings,
errors, non-claims, reproduction commands, and optional `review_priority`
hints.

Required artifact basenames for the canonical layout:

| Basename | Produced by | Required |
| --- | --- | --- |
| `manifest.json` | `tokmd evidence-packet` | yes |
| `analyze.md` | `tokmd analyze --format md` | yes |
| `analyze.json` | `tokmd analyze --format json` | yes |
| `context.md` | `tokmd context` | yes |
| `syntax.json` | `tokmd syntax` | no |

When `tokmd analyze --output-dir <DIR>` is used, the command writes
`analysis.md` and `analysis.json`, not `analyze.md` and `analyze.json`.
Producers using `--output-dir` must either rename outputs to the canonical
basenames or pass explicit `--analyze-md` and `--analyze-json` paths to
`tokmd evidence-packet`.

### Manifest schema

The manifest schema identifier is `tokmd.evidence-packet/v1`. Required manifest
fields, artifact keys, and `review_priority` semantics are defined in
`docs/evidence-packet.md` and `docs/evidence-packet.schema.json`. This spec
does not duplicate the full field table; it owns workflow semantics around that
schema.

### Command sequence

Until a thin orchestration command exists, producers generate the packet with
this sequence from the checkout root:

```bash
BASE="${BASE:-origin/main}"
HEAD="${HEAD:-HEAD}"
SCOPE="src/runtime/api"   # one or more scoped paths
OUT="sensors/tokmd"

mkdir -p "$OUT"

tokmd analyze \
  --preset bun-ub \
  --format md \
  --effort-base-ref "$BASE" \
  --effort-head-ref "$HEAD" \
  --no-progress \
  $SCOPE \
  > "$OUT/analyze.md"

tokmd analyze \
  --preset bun-ub \
  --format json \
  --effort-base-ref "$BASE" \
  --effort-head-ref "$HEAD" \
  --no-progress \
  $SCOPE \
  > "$OUT/analyze.json"

tokmd context \
  --budget 64000 \
  --output "$OUT/context.md" \
  --force \
  $SCOPE

tokmd syntax \
  --no-progress \
  $SCOPE \
  > "$OUT/syntax.json"

tokmd evidence-packet \
  --preset bun-ub \
  --base "$BASE" \
  --head "$HEAD" \
  --output "$OUT/manifest.json" \
  --analyze-md "$OUT/analyze.md" \
  --analyze-json "$OUT/analyze.json" \
  --context-md "$OUT/context.md" \
  --syntax-json "$OUT/syntax.json" \
  $SCOPE
```

#### Output-flag rules

Producers must avoid shell redirect pitfalls that break packet verification:

| Surface | Preferred write path | Redirect pitfall |
| --- | --- | --- |
| `tokmd context` | `--output <path> [--force]` | Shell `>` is acceptable on UTF-8 shells but `--output` is preferred. |
| `tokmd analyze` | Shell `>` to canonical `analyze.md` / `analyze.json`, or `--output-dir` plus explicit manifest paths | Windows PowerShell `>` defaults to UTF-16LE and makes `analyze.json` fail UTF-8 parsing during verification. |
| `tokmd syntax` | Shell `>` with UTF-8 encoding, or an explicit UTF-8 file writer | Same UTF-16LE risk as analyze JSON on PowerShell. |
| `tokmd evidence-packet` | `--output sensors/tokmd/manifest.json` | Writes UTF-8 JSON directly; do not pipe manifest output through a shell redirect that changes encoding. |

On Windows PowerShell, prefer:

- `tokmd context --output ... --force` for `context.md`;
- `[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)` before
  redirecting analyze or syntax JSON;
- `Out-File -Encoding utf8` when a redirect is unavoidable;
- explicit `--analyze-md`, `--analyze-json`, `--context-md`, and
  `--syntax-json` paths when filenames differ from defaults.

Default `tokmd evidence-packet` artifact resolution uses the manifest parent
directory (`sensors/tokmd/` by default) for `analyze.md`, `analyze.json`,
`context.md`, and auto-detected `syntax.json`.

### Status semantics

`tokmd evidence-packet` assigns packet status from required-artifact presence,
ref resolution, and receipt inspection:

| Status | Meaning | Command exit |
| --- | --- | --- |
| `complete` | All required artifacts exist; base/head refs resolve; `analyze.json` is parseable and consistent with requested preset and paths; no warnings or errors recorded. | `0` |
| `partial` | Required artifacts exist, but non-fatal warnings bound the evidence. Examples: `analyze.json` status is `partial`; optional `syntax.json` is missing, malformed, or degraded; explicit `--syntax-json` points to a missing file. | `0` |
| `failed` | A required artifact is missing; base/head refs do not resolve; `analyze.json` cannot be parsed; preset or `source.inputs` do not match the requested scope; or another fatal verification error is recorded. | non-zero after writing manifest |

Rules:

- Do not mark a packet `complete` when the real state is `partial` or `failed`.
- Optional syntax degradation must not hide required analyze/context validity.
- `failed` packets must still write `manifest.json` so consumers can inspect
  named errors.
- Syntax evidence is advisory. Failed or malformed `syntax.json` degrades to
  `partial` with warnings unless a downstream workflow explicitly requires
  syntax and treats missing syntax as failure outside this spec.

### Support model

| Path | Role | Current status |
| --- | --- | --- |
| Local CLI manual sequence | Developer and maintainer fallback; canonical proof path. | implemented |
| Local CLI orchestration (`tokmd packet generate`) | Thin wrapper over the manual sequence. | planned |
| GitHub Action (`EffortlessMetrics/tokmd-action`) | Default PR workflow UX with prebuilt binary runtime. | planned |
| GHCR container runtime | Optional pinned Linux/container runtime for workflows. | pending public visibility verification |
| Cargo install / release binary | Local development and manual fallback, not default CI path. | implemented |
| `nix run` | Optional local install path. | implemented |

Support expectations:

- The Action path should download or invoke a prebuilt `tokmd` binary by
  version, run the packet sequence from the checkout root, upload
  `sensors/tokmd/` when requested, and expose manifest status through stable
  outputs.
- GHCR is a secondary runtime only. It must not be documented as a supported
  default until anonymous manifest inspection, pull, `--version`, and mounted
  packet smokes pass for the published tag.
- Cargo install remains a fallback for local use and development; it is not the
  default CI adoption path.

Detailed Action inputs, outputs, and `fail-on` policy live in
`docs/packet-workflows.md`. Release-facing publishing evidence lives in
`docs/specs/publishing-evidence.md`.

### Claim boundary

An evidence packet proves only that scoped `tokmd` receipts were produced for
the recorded base, head, paths, and preset and that `tokmd evidence-packet`
verified the indexed artifacts against that scope.

An evidence packet does **not** prove:

- undefined behavior exists or is absent;
- public reachability of flagged syntax seams;
- memory safety or absence of panic paths at runtime;
- correctness of parser-backed `review_priority` ordering;
- whole-repository coverage unless `.` or the whole tree was in scope;
- CI proof, fuzzing, Miri, mutation, coverage upload, or release readiness;
- merge readiness or human-review completion;
- cockpit review-packet completeness;
- GHCR image availability or Action runtime health beyond the recorded manifest
  status.

Consumers must treat `review_priority` as advisory first-read hints with refs
back into `syntax.json`. Open referenced receipt entries before making review
claims.

### Verifier inputs and failure modes

`tokmd evidence-packet` is the packet verifier. Its inputs are:

- `--base`, `--head`, and scoped `PATH` arguments;
- `--preset` (default `bun-ub`);
- `--output` manifest path (default `sensors/tokmd/manifest.json`);
- optional `--analyze-md`, `--analyze-json`, `--context-md`, `--syntax-json`,
  and `--context-budget` overrides.

Verifier behavior:

| Check | On failure |
| --- | --- |
| Git available and base/head refs resolvable | `failed`; error names missing ref or missing git |
| Required artifacts exist on disk | `failed`; error names missing artifact |
| `analyze.json` is valid UTF-8 JSON | `failed`; parse error recorded |
| `analyze.json.status` is `complete` or `partial` | `partial` warning or `failed` for unsupported status |
| `analyze.json.args.preset` matches requested preset | `failed` |
| `analyze.json.source.inputs` matches requested paths | `failed` |
| `analyze.json.warnings` present | warnings copied into manifest; may yield `partial` |
| Optional `syntax.json` present and parseable | warnings on schema/status/path mismatch or parse failure; may yield `partial` |
| `syntax.json` `review_signals` | may populate `review_priority` with JSON Pointer refs |

Consumers re-running verification should invoke `tokmd evidence-packet` with the
same scope and artifact paths recorded in `manifest.json` `reproduce` commands.

Integration tests in `crates/tokmd/tests/evidence_packet_integration.rs` and
schema validation against `docs/evidence-packet.schema.json` are the current
automated proof for verifier behavior.

## Compatibility

This spec does not change public `tokmd` CLI behavior, receipt schema versions,
cockpit review packets, handoff bundles, branch-protection gates, proof
promotion, Codecov defaults, release mutation, or AST defaults.

Existing consumers can continue to use:

- `docs/evidence-packet.md` as the user-facing field reference;
- `docs/packet-workflows.md` as the planned Action and orchestration guide;
- `docs/integrations/ub-review.md` as the review-bot recipe;
- `docs/evidence-packet.schema.json` as the manifest JSON Schema;
- `tokmd evidence-packet` as the manifest writer and verifier.

Future changes that alter required artifact names, status semantics, support
model boundaries, or verifier failure modes must update this spec, the schema
when needed, and `crates/tokmd/tests/evidence_packet_integration.rs` in the
same PR.

## Proof Requirements

For spec-only changes:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-evidence-packet-workflow-spec.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-evidence-packet-workflow-spec.json --evidence-json target/proof/proof-evidence-evidence-packet-workflow-spec.json
git diff --check
```

Implementation or verifier changes should also run:

```bash
cargo test -p tokmd --test evidence_packet_integration --verbose
cargo test -p tokmd-types evidence_packet --verbose
```

Proof establishes spec routing, required artifact names, doc-artifact shape,
and existing integration-test coverage for manifest status behavior.

Proof does not establish Action runtime availability, GHCR public visibility,
or end-to-end UB detection.

## Open Questions

- Whether `tokmd packet generate` should become the only supported producer
  path once implemented, with manual recipes kept as troubleshooting docs only.
- Whether canonical analyze filenames should converge on `analysis.*` from
  `--output-dir` or remain `analyze.*` for ub-review compatibility.
- Whether a separate machine-readable verifier receipt should be emitted beside
  `manifest.json` for CI upload.
- Whether downstream `ub-review` should treat missing optional syntax as
  workflow failure independent of packet `partial` status.
