# NOW / NEXT / LATER

> One-screen operational truth. Updated after the docs/governance wave that
> followed the browser ZIP byte-mode upload chain (swarm #352-#354).

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
- **Publication merge-commit policy decision**: `repo-graph` reports `Diverged`
  (publication `f9d68cf4` ahead 1, swarm `4275107f` ahead 2, merge-base
  `2379eb72`) because publication squash-imported swarm #358/#359 as #2754.
  Trees are byte-identical (`0693c572`), so the only repair is a deliberate
  publication-side merge-commit import (toggle `allow_merge_commit` or push a
  local `--no-ff` merge) followed by a swarm fast-forward back to the merge
  commit, per `docs/specs/repo-topology.md`. This is a publication-repo /
  policy action and is intentionally out of scope for swarm PRs.

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
