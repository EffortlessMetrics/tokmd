# NOW / NEXT / LATER

> One-screen operational truth. Updated after the strip_prefix redaction import
> batch (swarm #428 redaction hardening merged at `0b67f750`; publication import
> #2822 open; prior alignment at `6f8e6c50` after #427 handoff fix).

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

**Graph state**: `repo-graph` reports `Aligned` at swarm `84bc8882` /
publication `84bc8882` (`publication_ahead=0`, `swarm_ahead=0`) after the
perf/determinism batch import #2817 (#425); the prior backlog top-5 batch
import #2810 (#416–#419) landed at `276bdb22`.

**No new agent-executable seams** were found in `SPEC_GAPS.md`, open issues,
or workflow hardening without org secrets. Reopen work only from fresh consumer,
artifact, workflow, or product evidence.

**Claim boundary**: this session proves no-panic allowlist hygiene for Lane 3
test imports and records honest lane-queue disposition. It does not enable badge
auto-CI, widen rootless presets, promote AST defaults, or prove manual browser
smoke.

## Backlog top-5 batch closeout (2026-07-03)

Agent-executable backlog cleanup for this batch is at handoff:

- **#416**: NOW state alignment to `eba85d84` and wasm-caller landed marker.
- **#417**: reject zero for analyze/badge git limits
  (`crates/tokmd/src/cli/parser/{analysis,badge}.rs` + `cli_errors_w66.rs`).
- **#418**: deterministic invalid-UTF-8 CLI parser regression test
  (`crates/tokmd/tests/cli_parser_fuzz_regression.rs`).
- **#419**: executable `tokmd gate` doc examples
  (`docs/reference-cli.md`, `crates/tokmd/src/cli/parser/gate.rs`,
  `cli_error_help_w73.rs`).
- **Publication import #2810**: merge-commit import landed the batch;
  `repo-graph` reports `Aligned` at `276bdb22` (publication_ahead=0,
  swarm_ahead=0).
- **Superseded publication PRs closed**: direct-publication duplicates whose
  substantive content landed via #2810 — **#2793**/**#2783** (zero/positive git
  max-commit validation → #417), **#2781** (invalid-UTF-8 fuzz regression →
  #418), and **#2804** (gate doc drift → #419). Closed with routing comments.
- **PR #410 / #2805 (badge automation)**: left open — badge endpoint churn still
  needs org `BADGE_PAT` (see #389/#390); do not approve a full gate for badge
  churn.

**Scout disposition (no PR landed)**: remaining direct-publication backlog items
stay open pending their own proof paths — bolt `FileStatRow` defer (#2780) and
bolt double-UTF-8 (#2752) both need a perf-smoke receipt first (Lane 3 is
measurement-led); dependabot rust-minor group (#2788) needs a full workspace
build/test lane; determinism regression fix (#2756) and FileRow sorting
properties (#2774) remain candidate slices. None were clear + narrow +
proof-ready enough to land in this session beyond this NOW alignment.

**Claim boundary**: this batch proves CLI arg-validation hardening, an
invalid-UTF-8 parser regression test, executable gate docs, and honest backlog
disposition. It does not enable badge auto-CI, widen rootless presets, land any
perf optimization without a receipt, or prove manual browser smoke.

## Perf/determinism batch closeout (2026-07-04)

Agent-executable backlog cleanup for this batch is at handoff:

- **#421**: determinism guard against silent CLI failures
  (`crates/tokmd/tests/determinism_regression.rs`).
- **#422**: FileRow sorting and aggregation property tests
  (`crates/tokmd-model/tests/file_row_properties.rs`).
- **#423**: defer `FileStatRow` creation to report boundaries
  (`crates/tokmd-analysis/src/content/file_stats.rs`; perf-smoke receipt in
  PR body).
- **#424**: borrow file text via `as_text` to drop double UTF-8 pass
  (`crates/tokmd-analysis/src/content/mod.rs`).
- **#425**: rust-minor-patch dependency group bump (8 updates).
- **Publication imports #2814–#2817**: merge-commit imports landed the batch;
  `repo-graph` reports `Aligned` at `84bc8882` (publication_ahead=0,
  swarm_ahead=0).
- **Superseded publication PRs closed**: direct-publication duplicates whose
  substantive content landed via swarm keepers — **#2780** (FileStatRow defer →
  #423), **#2752** (as_text UTF-8 → #424), **#2756** (determinism guard →
  #421), **#2774** (FileRow properties → #422), **#2788** (rust-minor-patch →
  #425). Closed with routing comments.
- **PR #410 / #2805 (badge automation)**: left open — badge endpoint churn still
  needs org `BADGE_PAT` (see #389/#390); do not approve a full gate for badge
  churn.

**AST scout (2026-07-04)**: explicit opt-in syntax surfaces are working locally —
`cargo test -p tokmd-analysis --features ast ast` (60 tests), `cargo test -p
tokmd --features ast --test cli_syntax_integration` (4 tests),
`cargo xtask ast-shadow-compare` / `ast-shadow-check` (20 corpus files, 717
matched landmarks), and `tokmd syntax` on `fixtures/syntax/` emit
`tokmd.syntax_receipts.v1`. Packet `--syntax` coverage exists in
`packet_generate_integration.rs` and `evidence_packet_integration.rs`. No
default-receipt promotion gap; support tier remains `experimental` opt-in per
`docs/specs/ast-syntax-support-tier.md`.

**Scout disposition (no PR landed beyond NOW)**: remaining direct-publication
backlog items stay open pending their own proof paths — dependabot and Jules
draft PRs (#2812–#2819) need narrow restack or maintainer review; manual browser
ZIP smoke remains human-only per `docs/browser-zip-smoke.md`. Reopen AST or
browser work only from fresh consumer, artifact, or function-boundary evidence.

**Claim boundary**: this batch proves determinism regression coverage, FileRow
property tests, two measured perf seams (FileStatRow defer, as_text UTF-8),
dependency hygiene, and honest AST end-to-end scout receipts. It does not enable
badge auto-CI, widen rootless presets, promote AST onto default receipts, or
prove manual browser smoke.

## Strip_prefix redaction closeout (2026-07-04)

Agent-executable security hardening for this batch is at handoff:

- **#428**: use `short_hash` instead of `redact_path` for export `strip_prefix`
  under Paths/All redaction (`tokmd-core`, `tokmd-format` json/jsonl); regression
  test in `test_redaction_leak.rs`. Squash-merged at **`0b67f750`**.
- **Publication import #2822**: merge-commit import open (TRUE merge from swarm
  `0b67f750`; supersedes Jules draft **#2819**).
- **Scout disposition (skipped)**: **#2812** perf top-offenders (needs perf-smoke
  receipt); **#2813** git spawn unification (22k-line no-panic allowlist churn).

**Graph state**: after #428 merge, `repo-graph --expect swarm-ahead` until import
#2822 lands and swarm fast-forwards to the publication merge commit.

**Claim boundary**: this batch proves opaque `strip_prefix` redaction for
directory-like prefixes that resemble filenames. It does not change file-path
redaction semantics, enable badge auto-CI, promote AST defaults, or prove manual
browser smoke.

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
