# NOW / NEXT / LATER

> One-screen operational truth. Updated after shipping the PR evidence packet
> workflow lane in `v1.14.0`.

## NOW (active)

- **PR evidence packet workflow shipped in `v1.14.0`**: `sensors/tokmd/`
  evidence packets are now boring to generate from one local command
  (`tokmd packet generate`) and one GitHub Action step (`mode: packet`), with
  `tokmd render` for packet presets. The reserved GHCR container runtime for the
  Action (`runtime: container`) remains the open follow-up; the prebuilt-binary
  runtime is the default today.
- **Release/distribution readiness is closed**: existing install, Action, review, handoff, browser-to-native, publishing, and release-evidence guides are the current adoption packet.
- **Proof control plane is observing, not promoting**: proof-pack routing, fast proof-run, scoped coverage, mutation, and coverage telemetry remain advisory unless maintainers deliberately promote them with fresh evidence.
- **Cockpit and handoff are the evidence surfaces**: keep `tokmd cockpit` as the PR-review surface and `tokmd handoff` as the agent work-order surface unless a fresh accepted contract selects something else.
- **Main must stay boring**: keep CI green, keep route receipts truthful about changed files and skipped-by-policy lanes, and avoid release-only branch noise in the swarm workbench.

## NEXT (short horizon)

- **Packet workflow GHCR runtime**: the packet CLI and `mode: packet` Action
  shipped in `v1.14.0` on the prebuilt-binary runtime; the reserved
  `runtime: container` GHCR path and Cargo fallback story are the remaining
  support-model work. Publication GHCR (`ghcr.io/effortlessmetrics/tokmd`)
  published `v1.14.0` (advisory unauthenticated manifest pass; formal
  `verified-public` maintainer receipt recorded for `v1.13.1`). Swarm GHCR
  is verified-public for `:main` (workbench/experimental tier; issue #264
  closed 2026-06-24, see `docs/specs/swarm-ghcr-image.md`).
- **CLI friction**: continue practical help examples, actionable errors, progress, and config explainability where current command use shows a real user gap.
- **Review evidence consumption**: improve cockpit/review packet reading, hosted-comment, or missing-evidence behavior only when current evidence shows a concrete product or verifier gap.
- **Measured CI feedback**: improve CI/proof routing and telemetry from receipts; do not weaken proof to make CI look faster.

## LATER (roadmap)

- **Browser/WASM product continuation**: keep browser capability claims explicit and rootless preset work evidence-led.
- **MCP/server mode**: expose stable read-only receipt resources before adding long-lived execution surfaces.
- **AST shadow evidence**: continue only behind shadow/developer-facing evidence until comparison data justifies public behavior or schema changes.
