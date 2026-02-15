# tokmd responsibilities

tokmd is the **Change Surface + Repo Intelligence** sensor in the sensors → receipts → cockpit stack.

It exists to answer one reviewer / LLM question well:

> “What changed, where is risk concentrated, and what should I look at first?”

tokmd stays useful standalone. It integrates with the cockpit via **artifacts + receipts**, not by becoming a director.

---

## Scope boundaries

### tokmd owns

**Repo truth (deterministic, cheap)**
- File inventory (paths, kinds, LOC/bytes/tokens, module grouping)
- Content-derived signals (complexity summaries, entropy/tagging when enabled)
- Stable receipts and stable ordering

**Diff truth lens (deterministic, bounded)**
- Base/head change surface summary
- Review plan (top files to inspect first)
- Git-enriched context when available (hotspots, churn, coupling), capability-gated

**LLM handoff and context packing**
- Produce a token-budgeted code bundle (reading payload)
- Produce a manifest-indexed handoff bundle (index + horizon + warning label + code)
- Prevent “context blindness” (LLM assuming the bundle is the whole repo)

**Governance + trending**
- Baselines for trend tracking (baseline command)
- Ratchet evaluation over baselines (gate command)
- Policy evaluation over receipts (gate command)

**Embedding**
- clap-free library facade (`tokmd-core`)
- FFI JSON entrypoint and thin language bindings (Python/Node)

### tokmd explicitly does not own

**Not a director**
- does not aggregate other sensors
- does not decide the global merge verdict
- does not post PR comments via network APIs

**Not build truth**
- does not run tests/coverage/clippy/bench
- does not map build artifacts onto diffs (covguard/lintdiff/perfgate do that)

**Not machine truth**
- does not validate local environment/tool installs/hashes (env-check)

**Not an actuator**
- does not write repo changes/fixes (buildfix)

Boundary discipline is the anti-monolith guardrail.

---

## Interfaces (contracts)

tokmd has three public integration surfaces:

1) CLI (humans + CI)
2) Artifacts/receipts (machines + cockpit + LLM bundles)
3) Embedding API (Rust + FFI + Python/Node)

### CLI responsibilities

Each command must be independently useful.

**Inventory**
- `tokmd export` is the “horizon” (complete inventory).

**Intelligence**
- `tokmd analyze` is structured intelligence (risk/complexity/coupling) with capability gating.

**PR context**
- `tokmd cockpit` is the PR-friendly summary + review plan (budgeted output).

**Context packing**
- `tokmd context` is the reading payload builder (token budget + deterministic selection).

**Handoff**
- `tokmd handoff` composes inventory + intelligence + code into an LLM-ready bundle.

**Governance**
- `tokmd baseline` snapshots metrics for trend tracking.
- `tokmd gate` evaluates policy + ratchet vs baseline.

### Canonical artifacts for cockpit integration

When used as a cockpit sensor, tokmd must be able to emit stable artifacts:

```

artifacts/tokmd/
├── report.json   # full tokmd cockpit receipt (tokmd-native schema)
└── comment.md    # compact summary (3–8 bullets max)

```

`comment.md` is intentionally short and deterministic. The cockpit director can inline it and link to `report.json`.

**Default cockpit posture:** informational unless configured otherwise.

---

## Context packing layers

Context packing is separated into layers to prevent duplication and “context blindness”.

### Layer owners (one authoritative source each)

**Index**
- `manifest.json` (authoritative): budgets, capabilities, artifacts, included/excluded, hashes.

**Horizon**
- `map.jsonl` (authoritative): complete inventory of what exists.

**Warning label**
- `intelligence.json` (summary-only): tree skeleton + top-N risk hints + warnings.

**Reading payload**
- `code.txt` (authoritative): token-budgeted file contents.

### Handoff bundle contract

`tokmd handoff` writes:

```

<out-dir>/
├── manifest.json      # authoritative index (schema v3)
├── map.jsonl          # full inventory horizon
├── intelligence.json  # payload-only warning label
└── code.txt           # token-budgeted code bundle

```

Non-negotiables:
- manifest is the only place for global metadata/capabilities/hashes
- map.jsonl is the only full inventory
- intelligence.json is capped summary (no second inventory)
- code.txt is the only code payload

---

## Capability gating

Git enrichment is optional and must never fail the bundle by accident.

Capabilities are always recorded as:
- `available`
- `skipped` (e.g., `--no-git`)
- `unavailable` (not in a git repo / shallow / missing tool)

When unavailable:
- omit the dependent fields (hotspots/churn/coupling)
- emit a warning explaining why (in manifest/intelligence/cockpit as appropriate)

No “green by omission”.

---

## Determinism and stability requirements

tokmd must guarantee:
- stable file ordering (explicit tie-breakers)
- stable selection given identical inputs and exclusions
- stable truncation behavior (caps are deterministic and signaled)
- forward-slash, repo-relative paths in artifacts
- output directories are excluded by construction and recorded as exclusions
- artifact integrity hashes (blake3) for non-self artifacts

Schemas and IDs:
- receipt schemas are versioned
- finding IDs are stable (never rename; deprecate/alias only)

---

## Internal architecture responsibilities

### Contracts (stable-ish API)
- `tokmd-types`, `tokmd-analysis-types` contain DTOs and schema-versioned contracts.

### Domain (pure logic)
- context packing selection should converge on a **single PackPlan** concept:
  - budgets and sub-budgets
  - included_files ordered list
  - exclusions with reason codes
  - deterministic ordering rules
  - capability snapshot

Both `tokmd context` and `tokmd handoff` must consume the same plan to prevent drift.

### Adapters (I/O)
- scan/walk/content/git are adapters feeding the domain
- CLI and handoff writers are I/O composition

---

## Interactions with the rest of the stack

tokmd complements the other sensors:

- builddiag / depguard / diffguard: enforce repo/diff policy contracts
- covguard / lintdiff / perfgate: consume build outputs and map truth onto diffs
- env-check: validates machine state
- buildfix: applies allowlisted fixes from receipts
- cockpitctl: ingests receipts and renders one merge surface

tokmd does not replace any of these. It provides the missing “context” slice that makes the cockpit readable.

---

## Open work (final-form)

High-leverage remaining work:
1) PackPlan unification (one selection engine shared by context + handoff)
2) Meta-budget partitioning in handoff (tree/risk/header allocation without bloating code budget)
3) Deep preset fidelity (optional additional analysis artifact or embedded receipt only in deep mode)
4) ~~tokmd-settings split~~ Done (v1.6.0) — `tokmd-settings` crate decouples clap from library API
5) Schema validation for context bundle manifest (if treated as public API)

---

## Related docs

- `docs/handoff.md` — handoff usage and output layout
- `docs/handoff-schema.md` + `docs/handoff.schema.json` — handoff manifest contract
- `docs/tokmd-in-cockpit.md` — cockpit integration contract and policy defaults
- `docs/SCHEMA.md` — core receipt families and schema versions
