# tokmd Architecture Role

tokmd is the **change-surface + repo intelligence sensor** in the Effortless cockpit stack.

It answers the questions other sensors do not:

- What changed, structurally (scope, composition, hotspots)?
- Where is risk concentrated (churn x complexity x coupling)?
- What should a reviewer / LLM look at first (review plan)?
- How do we hand a repo to an LLM without it hallucinating structure (handoff bundles)?

tokmd is deliberately **not** a director, and it does not replace build-truth tools.

---

## 1) Position in the cockpit ecosystem

### Truth layer
tokmd is **repo truth** with an optional **diff truth** lens.

- Repo truth: inventory, module structure, token/LOC accounting, complexity (content-derived).
- Diff truth: base/head change surface and review plan.
- Optional git enrichment: churn/hotspots/coupling when `git` is available and not disabled.

### Non-goals (boundary discipline)
tokmd must not become:

- **Director**: does not orchestrate other tools; does not produce the global merge decision.
- **Build-truth consumer**: does not run or map coverage/clippy/bench outputs to diffs (covguard/lintdiff/perfgate).
- **Machine truth**: does not verify local binaries/hashes; that is env-check.
- **Actuator**: does not write fixes; that is buildfix.

This boundary discipline is what keeps the ecosystem modular.

---

## 2) Integration contracts

tokmd has three integration surfaces:

1. CLI (humans + CI)
2. Receipts / artifacts (machines, directors, LLM bundles)
3. Library + FFI (embedders: Rust/Python/Node)

### 2.1 Canonical artifact paths

When tokmd is used as a cockpit sensor, it should emit a **canonical artifacts directory** with:

- `artifacts/tokmd/report.json` (tokmd sensor receipt; stable schema/version)
- `artifacts/tokmd/comment.md` (short, budgeted summary; optional but recommended)

The director (cockpitctl) should:
- inline only a small tokmd summary (3-8 bullets max),
- link to `comment.md` and `report.json` for depth.

**Policy (cockpit)** should treat tokmd as informational by default:

```toml
[sensors.tokmd]
blocking = false
missing = "warn"
highlights = 5
```

### 2.2 Capability gating (git is optional)

tokmd must degrade gracefully and *declare why*.

* If git is unavailable / disabled / shallow:

  * still produce inventory + content-derived intelligence
  * omit churn-based metrics
  * record capability status and warnings in outputs (handoff manifest, cockpit receipt, etc.)

No "green by omission."

### 2.3 Handoff bundle contract (LLM handoff)

tokmd provides an LLM-native bundle composer:

* `tokmd handoff --out-dir .handoff ...`

Output directory:

```
<out-dir>/
|-- manifest.json       # authoritative index (budgets, included/excluded, artifact hashes, capabilities)
|-- map.jsonl           # full inventory horizon (authoritative)
|-- intelligence.json   # warning label (tree skeleton + top-N hotspots/summary + warnings)
`-- code.txt            # token-budgeted reading payload (selected files)
```

**Dedupe rule:** each "big truth" is owned once:

* inventory lives in `map.jsonl`
* code lives in `code.txt`
* intelligence is summary-only (top-N + tree skeleton + warnings), not a second inventory
* manifest is index/provenance only (not a second intelligence report)

This is intentionally separate from cockpitctl's multi-sensor aggregation contract.

---

## 3) CLI surface (final shape)

tokmd's CLI groups into coherent families. Each command must remain standalone-useful.

### A) Inventory (repo truth)

* `tokmd lang`
* `tokmd module`
* `tokmd export` (full inventory; JSONL map)

### B) Intelligence (repo truth; optional git enrichment)

* `tokmd analyze --preset {health|risk|deep}`

### C) PR context (diff truth lens)

* `tokmd diff --base ... --head ...`
* `tokmd cockpit --base ... --head ...` (PR summary + review plan + capped evidence)

### D) Governance + trending

* `tokmd baseline` (baseline snapshot)
* `tokmd gate` (policy + ratchet vs baseline)

### E) Context packing + LLM handoff

* `tokmd context` (code bundle under budget; deterministic)
* `tokmd handoff` (composed bundle: manifest + map + intelligence + code)
* `tokmd tools` (machine schema for agent calling)

---

## 4) Internal architecture (crates and boundaries)

tokmd is a workspace; the repo's internal architecture follows a tiered "bounded domains" approach.

### Tier 1 - Contracts (publishable / stable-ish)

* `tokmd-types` (core DTOs: file rows, context rows, export contracts)
* `tokmd-analysis-types` (analysis DTOs, baselines, finding IDs)

These crates are "API surfaces": schema/version discipline applies.

### Tier 2 - Adapters

* `tokmd-scan`, `tokmd-walk` (inventory & discovery)
* `tokmd-content` (content-derived signals)
* `tokmd-git` (git as an adapter, not a core dependency)
* `tokmd-redact`, `tokmd-tokeignore` (utilities)

### Tier 3 - Orchestration / policy

* `tokmd-analysis` (+ format/render crates)
* `tokmd-gate` (policy + ratchet evaluation)

### Tier 4 - Facade / config

* `tokmd-core` (clap-free embedding facade + FFI entrypoint)
* `tokmd-config` (CLI-facing)
* (planned) `tokmd-settings` split to keep clap out of embedding surfaces

### Tier 5 - Runtimes

* `tokmd` (CLI)
* `tokmd-python`, `tokmd-node` (bindings over tokmd-core FFI)

---

## 5) Context packing: layered architecture (what we're standardizing)

Context packing is treated as a first-class domain, not CLI glue.

### Layer responsibilities

* `context` is the **reading payload** generator (selected file text under a token budget).
* `export` is the **horizon** (full inventory).
* `analyze` is **intelligence** (risk/complexity/coupling).
* `handoff` is **composition** (manifest-indexed bundle of all three layers).

### PackPlan principle (single source of truth)

Selection and bundling should share one internal plan object:

* budget partitions (meta vs code)
* exclusions (including output directory)
* included file list (ordered; token estimates)
* deterministic ordering tie-breakers
* capability snapshot (git status)

This prevents drift between:

* what was selected,
* what was written,
* what the manifest claims.

---

## 6) How tokmd fits in cockpitctl

tokmd contributes the **Change Surface** section:

* diff stats (files/lines)
* top hotspots touched (top 3)
* risk grade + short reasons
* review plan (top 5 files)
* link to tokmd artifacts for depth

cockpitctl remains responsible for:

* global budgets
* blocking policy
* missing receipt policy
* unified presentation

tokmd remains responsible for:

* producing deterministic, budgeted, explainable outputs on its own.

---

## 7) Stability rules (treat these as API)

* Receipt schemas are versioned; additive changes within a version only.
* Finding IDs are stable; never rename (deprecate/alias instead).
* Output ordering is deterministic (stable sort keys).
* Canonical artifact paths are stable.
* Capability-gated fields must be explicit: available / skipped / unavailable + reason.

---

## 8) Roadmap (what still needs tightening)

* Make pack selection "plan-based" (PackPlan) so context + handoff cannot drift.
* Ensure context/handoff manifests always record included_files[] and excluded_paths[].
* Add cockpit artifact output mode (`--artifacts-dir`) if not already present, producing `report.json` + compact `comment.md`.
* Finish the `tokmd-settings` split so tokmd-core/bindings never depend on clap types.
* Add schema validation + golden determinism fixtures for handoff + context manifests.
