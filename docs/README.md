# tokmd documentation

This directory contains user guides, product contracts, architecture notes,
schemas, and verification policy for `tokmd`.

## Key references

- [contributor-guide.md](contributor-guide.md) - make a first contributor PR
  without reading the whole repository history first.
- [debugging.md](debugging.md) - debug local tests, snapshots, receipts, CI
  mismatches, and performance evidence while developing tokmd.
- [start-here.md](start-here.md) — choose the shortest path for repo
  inspection, PR review, CI evidence, agent handoff, or browser evaluation.
- [install-and-try.md](install-and-try.md) — install tokmd, run the first
  useful commands, and move from local trial to review, handoff, CI, browser,
  or release-facing evidence.
- [user-paths.md](user-paths.md) — map each job to the command, primary
  artifact, first file to open, meaning, non-meaning, and next action.
- [workflows.md](workflows.md) — copy-ready command sequences for inspection,
  PR review, proof planning, proof observations, agent handoff, browser trial,
  and publishing evidence.
- [action-quickstart.md](action-quickstart.md) — copy-ready GitHub Action
  workflows for receipt artifacts and cockpit review packets.
- [examples/](examples/README.md) — small artifact-tree walkthroughs for
  review packets, handoff bundles, proof status, browser receipts, and
  publishing evidence.
- [browser.md](browser.md) — no-install browser workflow and native-only
  boundaries.
- [browser-to-native.md](browser-to-native.md) — move from a browser trial to
  native review packets, handoff bundles, and CI evidence.
- [analyze/bun-ub.md](analyze/bun-ub.md) — scoped `bun-ub` analysis artifacts
  for Bun undefined-behavior review bots, local reviewers, and agent handoff.
- [integrations/ub-review.md](integrations/ub-review.md) - copy-ready
  `ub-review` sensor recipe for `sensors/tokmd/analyze.md`,
  `sensors/tokmd/analyze.json`, and `sensors/tokmd/context.md`.
- [VERIFICATION.md](VERIFICATION.md) — README badge meanings, generated endpoints, and PR evidence boundaries.
- [agent-workflows/handoff-prompt.md](agent-workflows/handoff-prompt.md) —
  copy-ready prompt template for coding agents consuming `.handoff/` bundles.
- [agent-workflows/source-of-truth.md](agent-workflows/source-of-truth.md) — maintainer and agent workflow for following source-of-truth artifacts.
- [ci/swarm-routing.md](ci/swarm-routing.md) — dual-repo topology for
  `tokmd` publication imports and `tokmd-swarm` active development.
- [handoff.md](handoff.md) — coding-agent handoff bundle workflow and guardrails.
- [publishing-evidence.md](publishing-evidence.md) — release-facing package
  surface, metadata, and CI ownership evidence before release mutation.
- [release-readiness.md](release-readiness.md) — quickstart for pre-release
  evidence checks without publishing, tagging, or creating releases.
- [releases/1.12.md](releases/1.12.md) — user-facing release notes for the 1.12
  Bun UB evidence-readiness release.
- [releases/1.12-ledger.md](releases/1.12-ledger.md) — maintainer release
  record for the 1.12 Bun UB evidence-readiness release.
- [releases/1.11.md](releases/1.11.md) — user-facing release notes for the 1.11
  evidence-consumption release.
- [releases/1.11-ledger.md](releases/1.11-ledger.md) — lane-by-lane maintainer
  ledger for the 1.11 evidence-consumption release.
- [reference-cli.md](reference-cli.md) — generated CLI flag reference.
- [SCHEMA.md](SCHEMA.md) — receipt format documentation.
- [architecture.md](architecture.md) — crate hierarchy and data flow.
- [testing.md](testing.md) — testing strategy and frameworks.
