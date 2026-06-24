# Sibling tools and bidirectional learning

Mirror of
[`unsafe-review-swarm/docs/interop/sibling-tools.md`](https://github.com/EffortlessMetrics/unsafe-review-swarm/blob/main/docs/interop/sibling-tools.md).
Keep the convergence ledger table in sync with the canonical copy as items land.

`tokmd` is one of a series of **deterministic, fast, useful static PR
tools** that share interfaces, are composed by the same CI gate, and
deliberately learn from each other. Each is cheap, runs on a diff, and emits
trusted coverage artifacts without executing repo code or issuing a verdict.
This document makes that relationship explicit so a contributor in any of the
repos can see what is shared, what is converging, and how to keep the flow
going. Each sibling repo carries a mirror of this doc pointing back at the same
convergence ledger.

## The family

| Tool | Repo | Role (coverage instrument) |
|---|---|---|
| `unsafe-review` | `EffortlessMetrics/unsafe-review-swarm` | unsafe-contract coverage: which unsafe seams are reviewable, what evidence exists/is missing |
| `ripr` | `EffortlessMetrics/ripr-swarm` | mutation / weak-oracle exposure coverage |
| `cargo-allow` | `EffortlessMetrics/cargo-allow` | owned exception ledger (unsafe/panic/lint/etc. allow entries) |
| `tokmd` | `EffortlessMetrics/tokmd-swarm` | token-aware repo receipts and PR context packets |
| `ub-review` | `EffortlessMetrics/ub-review` | **CI gate** — composes the family (configurable) + LLM lanes; owns PR analysis, review, posting, and the blocking decision |

Each family member is a deterministic, fast, static PR tool — an instrument.
`ub-review` is the CI gate built on top of them: it keeps a repo's mandatory CI
surface clean and simple, then dynamically adds the PR-relevant gate items by
composing the family (the whole set, a user-configured subset, or others) and
running LLM lanes over their coverage artifacts for analysis, review, and
gating. None of the family members is itself the gate or the LLM reviewer (see
UNSAFE-REVIEW-SPEC-0028). They are complementary, not competing, and a
capability proven in one is expected to flow to the others.

## The wider pattern: sensors + two orchestrators

The same shape recurs across the org: **fast static sensors emit receipts; an
orchestrator compiles them into a merge surface.** There are two orchestrators,
at very different maturity:

- **`ub-review`** — *live, in heavy use.* The LLM review layer: composes the
  sensors + runs model lanes for PR analysis, review, and gating. This is the
  proven orchestrator and it leads the contracts.
- **`cockpitctl`** — *early alpha, not yet used; likely bumpy.* The deterministic
  twin: ingests sensor receipts (`artifacts/*/report.json`, opaque tool payload,
  contract in the envelope) and renders one deterministic merge surface
  (`cockpit.report.v1`). Aspirational; it converges toward the proven contracts
  as it matures, not the other way around.

Beyond the four instruments above, a broader **CI-sensor fleet** (covguard,
perfgate, lintdiff, diffguard, depguard, semverguard, buildfix, builddiag,
shiplog) follows the same sensor pattern and is emerging on the cockpitctl side.

**No NIH.** The first-party sensors exist only because no good tool covered the
gap (e.g. unsafe-contract coverage). Where a good tool already answers the
question, the orchestrator composes it — `ub-review` already runs third-party
sensors (`ast-grep`, `actionlint`, `semgrep`, `zizmor`, `gitleaks`,
`osv-scanner`, `cargo-audit`, `cargo-deny`, `shellcheck`) alongside the
first-party ones. The bar for building a sensor is a real, uncovered need; the
default is to compose an existing good tool. The shared receipt envelope is what
lets first-party and third-party sensors sit in the same merge surface.

**Convergence goal — one receipt, two orchestrators:** a sensor should emit a
single receipt envelope that *both* `ub-review` (LLM) and `cockpitctl`
(deterministic) consume. The envelope is anchored on the **proven** side first —
`ub-review`'s real consumption and `ripr`'s shipped `gate-decision.json`
(ripr-swarm #1038) — and the deterministic merge-surface side (cockpitctl #173)
aligns to it. unsafe-review's gate manifest (SPEC-0034) targets that shared
envelope, not a parallel format.

Maturity is honest here: the emerging side will be bumpy. The posture is to
exercise it, file precise issues against the receiving repo as bumps surface,
and drive the fix — proven side leads, emerging side catches up.

## Shared contracts (converging)

These are the interfaces the family is aligning on so `ub-review` can route all
sensors uniformly instead of special-casing each:

- **Sensor CLI shape**: `--root`, `--base`, `--diff`, `--head`, `--format`,
  `--out`. Diff/PR context flows in the same way to every sensor.
- **`<tool>-gate.json` manifest envelope**: a versioned (`schema_version`)
  manifest with `status`, a summary count block, artifact pointers, and a
  `trust_boundary`, so the orchestrator routes by schema, not by scraping
  stdout/markdown.
- **Ledger evidence taxonomy and dialect marker**: typed evidence prefixes
  (`test:`, `doc:`, `spec:`, `adr:`, `ripr:`, `unsafe-review:`, `coverage:`,
  `issue:`, `pr:`, `baseline-init:`), a `policy = "<dialect>"` dialect marker,
  owner/classification/lifecycle discipline, and one settled `schema_version`
  form (integer vs string). The `baseline-init:` prefix is unsafe-review-specific
  and marks entries generated automatically by `baseline init` for pre-existing
  debt capture.
- **Coverage-movement vocabulary**: `new` / `worsened` / `resolved` /
  `inherited` reported as posture against a baseline, with diff-scoped
  attribution; the orchestrator decides blocking.
- **Trust-boundary and advisory discipline**: advisory by default; conservative
  vocabulary; explicit "this does not prove ..." no-finding wording; no
  overclaim (no proof / UB-free / Miri-clean / site-execution / calibrated
  precision-recall / policy-readiness).
- **Spec lifecycle dashboard + wording-contract verifier**: machine-checked
  spec status with proof commands, and a verifier that rejects overclaim drift.

## Manifest envelope constants

The following fixed values are the schema-routing anchor for
`unsafe-review-gate.json` (SPEC-0034). Consumers MUST route by `schema_version`
and `dialect`; they MUST NOT infer tool identity from filenames or stdout format.

| Field | Fixed value | Notes |
|---|---|---|
| `schema_version` | `"unsafe-review-gate/v1"` | String form; agreed with ripr envelope |
| `dialect` | `"unsafe-review"` | Sensor identity marker for shared router dispatch |
| `status` | `"advisory"` | Always advisory; manifest carries posture, not a block verdict |
| `trust_boundary` | `"static unsafe-review coverage evidence; not proof, not a merge verdict"` | Fixed advisory string; no proof / UB-free / Miri-clean / site-execution claim |

These constants are verified by `cargo test -p unsafe-review-core gate_manifest` and
enforced in `check-pr`. See SPEC-0034 for the full envelope shape including the
`summary`, `artifacts`, `tool`, and `tool_version` fields.

## This tool (tokmd)

What tokmd is adopting from siblings and what it teaches back. Tracking issues
live in the ledger below; this table is the local orientation row.

| Direction | Capability | tokmd evidence | Status |
|---|---|---|---|
| Adopting | Sensor CLI shape (`--root`, `--base`, `--diff`, `--head`, `--format`, `--out`) | `tokmd sensor`, `docs/sensor-report-v1.md` | shipped |
| Adopting | Trust-boundary / advisory discipline on receipts and renderers | cockpit, handoff, context, `tokmd render` claim boundaries | ongoing |
| Adopting | `<tool>-gate.json` manifest envelope routing | consumer alignment pending family gate manifests | open |
| Adopting | Ledger evidence taxonomy + dialect marker | policy ledgers, `docs/POLICY_ALLOWLISTS.md` | partial |
| Adopting | Coverage-movement vocabulary (`new` / `worsened` / `resolved` / `inherited`) | baseline/trend surfaces | open |
| Teaching | `tokmd.packets/v1` input-schema ownership + `tokmd render --from-packets` | `docs/specs/tokmd-packets-render.md`, PR #300 | partial (phase 2: #301) |
| Teaching | Integer vs string schema identity split (receipt vs cross-tool manifest) | `docs/adr/0014-schema-identity-idioms.md`, PR #303 | landed (#224 closed) |
| Teaching | Per-family integer `schema_version` versioning | `docs/adr/0007-schema-family-versioning.md`, `docs/SCHEMA.md` | shipped |
| Teaching | `sensor.report.v1` fleet envelope for conforming sensor mode | `docs/sensor-report-v1.md`, `tokmd sensor` | shipped |
| Teaching | Evidence packet workflow (`tokmd.evidence-packet/v1`) | `docs/specs/evidence-packet-workflow.md`, `docs/evidence-packet.md` | shipped |

## Bidirectional learning ledger

Live convergence items. Direction = which tool is ahead; the receiving repo
owns the tracking issue. Keep this table current as items land (sync with the
canonical copy in unsafe-review-swarm).

| Capability | Ahead | Receiving | Tracking | Status |
|---|---|---|---|---|
| Versioned gate manifest + baseline-debt-delta schema | ripr | unsafe-review | unsafe-review-swarm #1522 | open |
| Multi-mode gate (visible-only → calibrated) | ripr | unsafe-review | unsafe-review-swarm #1522 | open |
| Canonical `new_unsuppressed` counter for threshold consumers | ripr | (consumer contract) | ripr-swarm #1038 | open |
| Exception-ledger rigor: typed evidence, classification, structural selectors, ownership, dialect marker | cargo-allow | unsafe-review | unsafe-review-swarm #1523 | partial (typed-evidence prefix gate landed; structural selectors and dialect marker open) |
| `schema_version` integer-vs-string convergence | tokmd (documented split) | family-wide | cargo-allow #1465, tokmd-swarm #224 | partial (ADR-0014 landed #303; cargo-allow #1465 open) |
| Machine-checked spec-status dashboard | unsafe-review | ripr | ripr-swarm #1040 | open |
| No-finding wording-contract verifier | unsafe-review | ripr | ripr-swarm #1040 | open |
| Diff-first consumer contract alignment | unsafe-review | ripr | ripr-swarm #1041 | open |
| Coverage-movement vocabulary (new/worsened/resolved/inherited) | unsafe-review | cargo-allow | cargo-allow #1471 | open |
| tokmd-packets input-schema ownership + `--from-packets` consumer | unsafe-review (producer) ↔ tokmd (consumer) | tokmd | tokmd-swarm #222, #301 | partial (phase 1 #300 landed; phase 2 #301 open) |
| `check-local-context` / limited-runtime vocabulary — `downstream_consumable` operator field shipped; `has_unix_absolute_machine_path` / `has_session_state_marker` predicates shipped | ripr | unsafe-review | unsafe-review-swarm #1520 | closed |
| Pre-guard scratch GC for shared CI runners | ripr | unsafe-review | unsafe-review-swarm #1519 | open |

## Standing process

When work in one tool surfaces something a sibling should learn (interface
alignment, manifest envelope, ledger/evidence schema, movement vocabulary,
trust-boundary discipline, receipts/confirmation, spec rails):

1. File the issue in the **receiving** tool's `-swarm` repo (the one that should
   adopt), tagged with **direction**, **evidence** (file:line / artifact), and a
   one-line **proposal**.
2. If a shared contract is involved, co-design it across both repos rather than
   asserting one side's schema — cross-link the sibling issue.
3. Add a row to this ledger (in both repos' mirror of this doc).
4. Do not duplicate an existing issue; comment with the concrete contract
   instead.

## Trust boundary

Cross-pollination changes interfaces and rigor, not claims. Every sibling tool
stays advisory by default and within its own trust boundary; sharing a manifest
envelope or a ledger schema never lets one tool assert another's proof. The
family converges on honesty discipline as much as on interfaces.
