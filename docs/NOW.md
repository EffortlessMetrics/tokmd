# NOW / NEXT / LATER

> One-screen operational truth. Updated after the post-Lane 3 session handoff
> (swarm #413 no-panic allowlist fix, #414 NOW closeout, and #415 NOW alignment
> merged; publication imports #2807 (#413), #2808 (#414), and the #415 NOW
> alignment landed; `repo-graph` reports `Aligned` at `eba85d84`,
> publication_ahead=0, swarm_ahead=0).

## Adoption wave closeout (2026-06-30)

Agent-executable adoption work for this wave is at handoff:

- **#364**: archive ZIP `LangReport` test anchor (host filesystem scan).
- **#365**: AST shadow backend identity + mismatch taxonomy wired to test values.
- **#366**: real evidence-packet producer bridged through ub-review consumer gate.
- **#367**: `docs/how-to-add-a-packet-field.md` extender guide (+ `ci/proof.toml`
  and `docs/evidence-packet.md` cross-links).
- **Publication imports #2764 / #2765**: merge-commit imports landed; `repo-graph`
  reports `Aligned` at `6c8db52b` (publication_ahead=0, swarm_ahead=0).
- **Publication PR #2719 closed**: direct-publication Jules friction frontmatter PR
  was conflicting/stale; restack on `tokmd-swarm:main` if still wanted.

**Claim boundary**: this wave proves test anchors and docs for packet/archive/AST
shadow surfaces. It does not prove manual browser ZIP smoke, release publish, or
Nix-full validation.

## AST productization closeout (2026-07-01)

Agent-executable AST/syntax productization work is at handoff:

- **#368–#370**: CLI `--exclude` honoring, governance reconciliation, packet
  exclude forwarding.
- **#371**: `docs/workflows/syntax-evidence-guide.md` user-path guide for
  UB/crash review using `review_signals`.
- **#372**: AST shadow corpus expanded for TypeScript and Python.
- **#380**: WASM `runJsonBytes` analyze boundary parity tests.
- **Publication import #2787**: merge-commit import landed (AST batch);
  superseded by import **#2790** at **`840c3ca9`** (current alignment).
- **Current alignment**: `repo-graph` reports `Aligned` at `840c3ca9`
  (publication_ahead=0, swarm_ahead=0) after import #2790.

**Claim boundary**: this lane proves explicit opt-in syntax surfaces
(`tokmd syntax`, packet `--syntax`, shadow compare/check) and matching docs/CI.
It does not promote AST facts onto default receipts, prove function-boundary
candidate criteria, or add browser tree-sitter.

## Lane 2/5 closeout (2026-07-02)

Agent-executable review-consumption and browser/WASM continuation work for this
batch is at handoff:

- **#398**: ub-review consumer freshness/cache-identity gate tests pin the
  documented trust-order step 3 (`schema`, `tokmd_version`, `base`, `head`,
  `paths`, `preset`) independently from attachability.
- **#399**: rootless analyze preset feasibility map in
  `docs/browser-capability-matrix.md` (code-backed blockers per preset).
- **Publication import #2799**: merge-commit import landed; `repo-graph`
  reports `Aligned` at `1c864623` (publication_ahead=0, swarm_ahead=0).

**Claim boundary**: this batch proves consumer-side cache-identity pinning and
an honest rootless-preset feasibility map. It does not widen
`ROOTLESS_ANALYZE_PRESETS`, promote AST facts onto default receipts, or prove
manual browser ZIP smoke.

## Lane 4 docs batch closeout (2026-07-02)

Agent-executable adoption/contributor docs work for this batch is at handoff:

- **#345**: badge endpoint refresh (inventory badge fix).
- **#389**: badge PR CI-suppression and maintainer nudge docs.
- **#390**: `docs/ci/inventory.md` lane inventory reconciled with
  `policy/ci-lane-whitelist.toml`.
- **#391**: `docs/specs/SPEC_GAPS.md` ub-review CI gate phase 4 marked done.
- **Publication import #2790**: merge-commit import landed; `repo-graph`
  reports `Aligned` at `840c3ca9` (publication_ahead=0, swarm_ahead=0).

**Claim boundary**: this batch proves badge visibility, CI lane inventory
accuracy, and ub-review gate phase-4 documentation. It does not enable badge
auto-CI (needs org `BADGE_PAT` secret) or change publication merge-commit UI
settings.

## Lane 3 measured-performance closeout (2026-07-02)

Agent-executable measured-performance work for Lane 3 is at handoff. The lane is
measurement-led throughout: no optimization landed without a perf-smoke receipt.

- **#401–#404, #406**: `cargo xtask perf-smoke` maintainer guide, the
  core-workflow baseline receipt, the first measured hot-path fix (PR B —
  `tokmd-model` line-based byte estimate, export `model_ms` 377 → ~40 ms), and
  the file-I/O cache evidence plan (PR C). Imported at `41c05d30` (import #2802).
- **#407, #409**: Lane 3 governance closeout and the opt-in content I/O
  open-trace (PR D — `tokmd-analysis::io_trace` + `perf-smoke --trace-io`) that
  measured the duplicate-read rate (health preset re-opens capped files ~1.54×,
  confirmed). Imported at `14d611cb` (import #2803).
- **PR F (#411)**: prototype request-scoped read cache + health-preset A/B
  (`tokmd-analysis::io_cache` + `perf-smoke --cache-io`). The cache serves all
  268 confirmed duplicate `head` opens (hit_rate 0.349) but changes `analyze
  total_ms` by only ~0.1% (within run-to-run noise), so **PR E (production cache)
  is closed as a measured no-go**. Evidence:
  `docs/ci/perf-smoke-io-cache-2026-07.md`. Imported at `883007be` (import #2806).

**Claim boundary**: this lane proves a repeatable perf-smoke measurement spine,
one low-risk measured core-workflow win (PR B), a confirmed duplicate-read rate
(PR D), and an evidence-settled file-I/O cache decision (PR F: no-go). It does
not add persistent caching to the default `analyze` path, change receipt schemas
or preset defaults, or promote any advisory proof. The trace and cache
prototypes remain opt-in maintainer instruments.

## Post-Lane 3 session closeout (2026-07-02)

Agent-executable work from this session:

- **#413**: receipt the 11 Lane 3 test-code panic-family findings
  (`panic-21941`..`panic-21951`) in `policy/no-panic-allowlist.toml`, restoring
  the advisory No-panic Policy strict gate to green on swarm `main` (`3dd612b2`).
- **#414**: post-Lane 3 session NOW closeout + lane queue state.
- **Publication imports #2807 (#413) and #2808 (#414)**: merge-commit imports
  landed in `tokmd`; `repo-graph` reports `Aligned` at `9c0bb1f4`
  (publication_ahead=0, swarm_ahead=0).

**Lane queue state (lanes 2–6)**:

| Lane | Status | Notes |
| --- | --- | --- |
| Lane 2 (ub-review / rootless) | **closed** | #398/#399 imported at `1c864623` |
| Lane 3 (measured performance) | **closed** | #401–#414 + PR E no-go; imported at `9c0bb1f4` (#2807/#2808) |
| Lane 4 (docs/adoption) | **closed** | #345/#389–#391 imported at `840c3ca9` |
| Lane 5 (rootless preset widening) | **blocked/closed** | feasibility map done; no preset promotion |
| Lane 6 | **blocked/closed** | no agent-executable seam selected |
| PR #410 / #2805 (badge automation) | **open — leave** | substantive ripr count refresh (`239→237`); CI stuck in `action_required` (bot PR; needs org `BADGE_PAT` per #389/#390); do not approve full gate for 1-line badge churn |

**Human-only remaining**:

- Browser ZIP smoke execution per `docs/browser-zip-smoke.md`.

**Graph state**: `repo-graph` reports `Aligned` at swarm `eba85d84` /
publication `eba85d84` (`publication_ahead=0`, `swarm_ahead=0`) after the
#415 NOW alignment import (imports #2807 (#413) and #2808 (#414) preceded it at
`9c0bb1f4`).

**No new agent-executable seams** were found in `SPEC_GAPS.md`, open issues,
or workflow hardening without org secrets. Reopen work only from fresh consumer,
artifact, workflow, or product evidence.

**Claim boundary**: this session proves no-panic allowlist hygiene for Lane 3
test imports and records honest lane-queue disposition. It does not enable badge
auto-CI, widen rootless presets, promote AST defaults, or prove manual browser
smoke.

## Shipped this wave

- **Browser ZIP smoke recipe (#356)**: maintainer recipe for manual browser
  smoke against a real archive is documented at `docs/browser-zip-smoke.md`.
- **AST shadow backend identity vocabulary (#357)**: shadow-only identity
  vocabulary and mismatch taxonomy spec, no public behavior change.
- **jules-index rollup fix (#358)**: `cargo xtask jules-index` now includes
  done friction items in `RUNS_ROLLUP.md`.
- **ADR-0015 (#359)**: ub-review partial packet consumption decision recorded.
- **RUNS_ROLLUP regen + this handoff**: regenerated
  `.jules/index/generated/RUNS_ROLLUP.md` from current packet state so
  `cargo xtask jules-index --check` is green again (drift was generated-output
  staleness only, no logic change).

## Human-only remaining

- **Browser smoke recipe execution**: run the manual browser smoke against a
  real archive per `docs/browser-zip-smoke.md`; only execution remains, the
  recipe and capability claims are in place.

## NOW (active)

- **Browser ZIP archive byte-mode upload is wired end-to-end**: the
  `archive-zip` byte-mode chain now reaches the browser. `tokmd_core::ffi::run_json_bytes`
  (core, swarm #352) feeds the `tokmd-wasm` `runJsonBytes(mode, optionsJson,
  archiveBytes: Uint8Array)` binding (swarm #353), and the `web/runner` UI accepts a
  user-selected ZIP, reads it into a `Uint8Array`, and forwards byte-mode options to
  the worker (swarm #354). Capability reporting stays honest: the runner only offers
  ZIP upload when the loaded bundle exposes `runJsonBytes`. Proof: `tokmd-wasm`
  native + `wasm-bindgen-test` byte-parity tests and `web/runner` npm tests (65 pass,
  1 skip for an absent local wasm bundle). **Claim boundary**: manual browser smoke
  against a real archive is not yet established; maintainer recipe at
  `docs/browser-zip-smoke.md` (streaming upload and tar-family containers remain
  out of scope; see `docs/browser-capability-matrix.md`).
- **PR evidence packet workflow shipped in `v1.14.0`**: `sensors/tokmd/`
  evidence packets are now boring to generate from one local command
  (`tokmd packet generate`) and one GitHub Action step (`mode: packet`), with
  `tokmd render` for packet presets. The GHCR container runtime for the Action
  (`runtime: container`) is now wired for verification-gated tags (currently
  `1.14.0`); the prebuilt-binary runtime remains the default.
- **Release/distribution readiness is closed**: existing install, Action, review, handoff, browser-to-native, publishing, and release-evidence guides are the current adoption packet.
- **Proof control plane is observing, not promoting**: proof-pack routing, fast proof-run, scoped coverage, mutation, and coverage telemetry remain advisory unless maintainers deliberately promote them with fresh evidence.
- **Cockpit and handoff are the evidence surfaces**: keep `tokmd cockpit` as the PR-review surface and `tokmd handoff` as the agent work-order surface unless a fresh accepted contract selects something else.
- **Main must stay boring**: keep CI green, keep route receipts truthful about changed files and skipped-by-policy lanes, and avoid release-only branch noise in the swarm workbench.

## NEXT (short horizon)

- **Packet workflow GHCR runtime**: the packet CLI and `mode: packet` Action
  shipped in `v1.14.0` on the prebuilt-binary runtime; the `runtime: container`
  GHCR path is now wired for verification-gated tags (currently `1.14.0`, with
  mutable tags rejected). Extending the supported-tag set per new stable tag and
  the Cargo fallback story are the remaining support-model work. Publication GHCR
  (`ghcr.io/effortlessmetrics/tokmd`)
  published `v1.14.0` (advisory unauthenticated manifest pass; formal
  `verified-public` maintainer receipt recorded for `v1.13.1`). Swarm GHCR
  is verified-public for `:main` (workbench/experimental tier; issue #264
  closed 2026-06-24, see `docs/specs/swarm-ghcr-image.md`).
- **CLI friction lane complete**: the Lane 1 CLI-friction packets (help examples, actionable errors, `--show-config`, and stderr progress events) shipped through #316-#323 (see `docs/ROADMAP.md` Lane 1). Reopen only from fresh evidence of a concrete, unaddressed CLI usability gap.
- **Review evidence consumption**: improve cockpit/review packet reading, hosted-comment, or missing-evidence behavior only when current evidence shows a concrete product or verifier gap.
- **Measured CI feedback**: improve CI/proof routing and telemetry from receipts; do not weaken proof to make CI look faster.

## LATER (roadmap)

- **Browser/WASM product continuation**: keep browser capability claims explicit and rootless preset work evidence-led. With ZIP byte-mode upload now wired (see NOW), the next browser follow-ons are manual browser smoke against a real archive, streaming/large-archive upload, and tar-family containers; treat each as a fresh evidence-led slice rather than an implicit promise.
- **MCP/server mode**: expose stable read-only receipt resources before adding long-lived execution surfaces.
- **AST/syntax productization lane closed** (2026-07-01): explicit `tokmd syntax`,
  packet `--syntax`, shadow compare/check, syntax evidence guide, and WASM analyze
  byte-mode parity are shipped and governance-aligned. See
  `docs/plans/ast-productization.md`. Default receipts unchanged; reopen only from
  fresh function-boundary or schema-review evidence.
