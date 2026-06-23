# NOW / NEXT / LATER

> One-screen operational truth. Updated after selecting the PR evidence packet
> workflow lane.

## NOW (active)

- **PR evidence packet workflows are selected**: make `sensors/tokmd/`
  evidence packets boring to generate from one local command and one GitHub
  Action step, starting with docs, then a thin CLI orchestrator, then Action
  runtime support.
- **Release/distribution readiness is closed**: existing install, Action, review, handoff, browser-to-native, publishing, and release-evidence guides are the current adoption packet.
- **Proof control plane is observing, not promoting**: proof-pack routing, fast proof-run, scoped coverage, mutation, and coverage telemetry remain advisory unless maintainers deliberately promote them with fresh evidence.
- **Cockpit and handoff are the evidence surfaces**: keep `tokmd cockpit` as the PR-review surface and `tokmd handoff` as the agent work-order surface unless a fresh accepted contract selects something else.
- **Main must stay boring**: keep CI green, keep route receipts truthful about changed files and skipped-by-policy lanes, and avoid release-only branch noise in the swarm workbench.

## NEXT (short horizon)

- **Packet workflow support model**: define the CLI, Action, publication GHCR,
  and Cargo fallback paths before implementation grows. Publication GHCR
  (`ghcr.io/effortlessmetrics/tokmd`) is verified-public for `v1.13.1`; swarm
  GHCR visibility remains undecided (issue #264).
- **CLI friction**: continue practical help examples, actionable errors, progress, and config explainability where current command use shows a real user gap.
- **Review evidence consumption**: improve cockpit/review packet reading, hosted-comment, or missing-evidence behavior only when current evidence shows a concrete product or verifier gap.
- **Measured CI feedback**: improve CI/proof routing and telemetry from receipts; do not weaken proof to make CI look faster.

## LATER (roadmap)

- **Browser/WASM product continuation**: keep browser capability claims explicit and rootless preset work evidence-led.
- **MCP/server mode**: expose stable read-only receipt resources before adding long-lived execution surfaces.
- **AST shadow evidence**: continue only behind shadow/developer-facing evidence until comparison data justifies public behavior or schema changes.
