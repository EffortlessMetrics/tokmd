# NOW / NEXT / LATER

> One-screen operational truth. Updated after the adoption-wave closeout
> (swarm #364-#367, publication imports #2764/#2765).

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
- **AST shadow evidence**: continue only behind shadow/developer-facing evidence until comparison data justifies public behavior or schema changes.
