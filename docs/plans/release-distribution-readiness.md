# Plan: Release And Distribution Readiness

- Status: complete
- Related proposal: `docs/proposals/release-readiness-receipt.md`
- Related spec: `docs/specs/publishing-evidence.md`
- Related ADR: `docs/adr/0001-production-package-publishability.md`, `docs/adr/0003-publish-surface-taxonomy.md`, `docs/adr/0005-release-train-and-rc-semantics.md`
- Related issues:

## Goal

Make tokmd easier for an outside maintainer to install, try, run in GitHub
Actions, use for PR review, prepare an agent handoff, and check
release-facing evidence without learning the internal control plane first.

This lane starts from a fresh adoption gap, not a missing parser, proof, AST,
or architecture primitive. The existing user-path, publishing evidence,
cockpit, handoff, browser, and proof-status surfaces are strong enough to use;
the work here is to make the adoption path boring:

```text
install or try
  -> inspect a repo
  -> review a PR
  -> prepare a coding-agent handoff
  -> read CI proof evidence
  -> check release/publishing readiness
```

Each packet should make one outside-user path clearer:

```text
what command do I run?
what artifact does it create?
what do I open first?
what does it prove?
what does it not prove?
what is the next action?
```

## Non-goals

- Do not publish crates, create tags, create GitHub releases, move release
  aliases, or push Docker images.
- Do not change release workflow behavior or package membership.
- Do not add a public `tokmd review` command.
- Do not promote fast proof, scoped coverage, mutation, or Codecov upload.
- Do not add AST-backed default output or browser AST capability claims.
- Do not implement evidencebus runtime integration in tokmd.
- Do not change public receipt schemas or public CLI behavior unless a later
  proposal proves existing artifacts cannot serve the named consumer.
- Do not reopen proof, AST, architecture, user-path compression, or publishing
  evidence lanes by inertia.

## Work Packets

1. Add an install-and-try guide.
   - Status: complete.
   - Added `docs/install-and-try.md` as the first-run path for Cargo install,
     release binaries, Nix, basic repo inspection, risk analysis, cockpit PR
     review, handoff, and browser mode.
   - Linked from README, `docs/README.md`, `docs/start-here.md`,
     `docs/install.md`, and
     `docs/user-paths.md` without duplicating all command details.
2. Add a GitHub Action quickstart.
   - Status: complete.
   - Added `docs/action-quickstart.md` for the adoption path from Action
     install to review packet artifact, optional comment, and verifier receipt.
   - Kept the full Action reference in `docs/github-action.md`.
   - Explained required/advisory proof boundaries, no merge verdict, and no
     default Codecov upload.
3. Record a real user-path smoke run.
   - Status: complete.
   - Added `docs/examples/real-user-path-smoke-run.md` after running
     affected/proof-plan, cockpit, review-packet-check, and handoff over the
     real GitHub Action quickstart PR range.
   - Recorded what was clear, what was confusing, what was fixed, and what was
     deferred without committing generated packets, CI logs, or large artifact
     dumps.
4. Add an agent handoff prompt template.
   - Status: complete.
   - Added `docs/agent-workflows/handoff-prompt.md` as a short copy-ready
     prompt for Codex, Claude, Cursor, or another coding agent consuming
     `.handoff/work-order.md`, `.handoff/code.txt`, `review-links.json`, and
     `proof-links.json`.
   - Keep it as a consumer bridge, not a second planning system.
5. Add a handoff work-order contract test.
   - Status: complete.
   - Strengthened the handoff integration test that covers linked review/proof
     evidence so it preserves the user-facing section order, review evidence,
     proof expectations, missing/stale/degraded evidence, stop conditions, and
     guardrails that make `.handoff/work-order.md` useful to agents.
   - Preserve handoff as a linker and summarizer for external evidence, not a
     verifier for proof or review packets.
6. Add browser-to-native adoption guidance if current browser docs still leave
   the trial-to-native path ambiguous.
   - Status: complete.
   - Added `docs/browser-to-native.md` as the bridge from no-install browser
     inspection to native `cockpit`, `handoff`, and CI evidence workflows.
   - Kept browser mode as a no-install trial lens and native mode as the
     review, proof, and handoff instrument.
7. Add a release evidence quickstart.
   - Status: complete.
   - Added `docs/release-readiness.md` as a short pre-release evidence
     quickstart that composes publish-surface, version consistency, affected
     proof, and proof-plan commands.
   - Made clear that these are pre-release evidence, not release mutation or
     release approval.
8. Decide whether a release-readiness wrapper receipt is needed.
   - Status: complete.
   - Added `docs/proposals/release-readiness-receipt.md` to record the
     decision point before any new command or receipt.
   - Recommendation: no wrapper yet. Existing publish-surface,
     version-consistency, affected, proof-plan, and proof-evidence artifacts
     are sufficient until a named release, Action, or downstream consumer needs
     one stable JSON envelope.
9. Compress the README first-run path after the adoption guides exist.
   - Status: complete.
   - Compressed the README quick-start path around install, repo inspection, PR
     review, agent handoff, CI adoption, browser-to-native, and release
     readiness links.
   - Kept deeper command inventory and control-plane references below the first
     user path instead of making them the first decision surface.
10. Close the lane.
    - Status: complete.
    - Closed after install/try, Action adoption, real smoke evidence, agent
      prompt guidance, handoff contract coverage, browser-to-native guidance,
      release evidence quickstart, release-readiness receipt decision, and
      README first-run compression were recorded.
    - The completed active goal is archived in
      `.jules/goals/archive/2026-05-17-release-distribution-readiness.toml`,
      and `.jules/goals/active.toml` is paused with no selected implementation
      lane.

## Validation

For docs-only packets in this lane, run:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-release-distribution-readiness.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-release-distribution-readiness.json --evidence-json target/proof/proof-evidence-release-distribution-readiness.json
cargo fmt-check
git diff --check
```

If a packet changes Action docs, validate examples against
`docs/github-action.md` and the current `action.yml` inputs. If a packet changes
handoff output or tests, run:

```bash
cargo test -p tokmd --test handoff_integration --verbose
cargo test -p tokmd context_pack --verbose
cargo test -p tokmd-types handoff --verbose
cargo clippy -p tokmd --all-targets -- -D warnings
```

If a packet changes release-facing docs, also run:

```bash
cargo xtask publish-surface --json --verify-publish
cargo xtask version-consistency
```

Run required affected proof if the affected proof plan selects it.

## Stop Conditions

- Stop if a packet needs release mutation, publication, tags, GitHub release
  creation, moving `v1`, or Docker/image pushes.
- Stop if existing `tokmd` and `xtask` artifacts cannot answer a consumer's
  question; write a proposal before adding a new command or receipt.
- Stop before adding a public `tokmd review` command.
- Stop before promoting advisory proof or enabling default Codecov upload.
- Stop before changing AST default behavior, browser AST capability claims, or
  public receipt schemas.
- Stop if affected planning reports unknown files.
- Stop if the work starts extending closed proof, AST, architecture,
  publishing-evidence, or user-path lanes without a fresh consumer gap.

## Checkpoint History

- 2026-05-17: Started from the parked active-goal state after queue hygiene and
  source-of-truth alignment. The selected fresh gap is adoption and release
  readiness from real user workflows, not new internal control-plane machinery.
- 2026-05-17: Added the install-and-try guide and linked it from the public
  entry points. The guide keeps install, inspect, PR review, handoff, browser,
  CI, and release-facing evidence paths on existing commands and artifacts.
- 2026-05-17: Added the GitHub Action quickstart and linked it from the public
  docs entry points. The guide shows minimal receipt and PR review-packet
  workflows while keeping the full Action reference separate.
- 2026-05-17: Recorded a real user-path smoke run over the GitHub Action
  quickstart PR range. The run verified affected planning, proof planning,
  cockpit review-packet generation, review-packet checking, and handoff
  generation while keeping generated packets out of the repo.
- 2026-05-17: Added a copy-ready handoff prompt template for coding agents
  consuming `.handoff/` bundles and linked review/proof evidence.
- 2026-05-17: Strengthened handoff work-order integration coverage so the
  agent-facing linked-evidence sections, stop conditions, and guardrails stay
  stable through future refactors.
- 2026-05-17: Added browser-to-native adoption guidance so browser trials end
  with concrete native review, handoff, and CI evidence next actions without
  implying browser/native parity.
- 2026-05-17: Added the release-readiness quickstart for pre-release package,
  version, affected-proof, and proof-plan evidence without publishing, tagging,
  creating releases, or approving mutation.
- 2026-05-17: Recorded the release-readiness wrapper receipt proposal and
  chose "no wrapper yet" until a concrete consumer cannot use the existing
  release evidence artifacts directly.
- 2026-05-17: Compressed the README first-run path so outside users see
  install, inspect, review, handoff, CI, browser, and release-readiness entry
  points before deeper command inventory.
- 2026-05-17: Closed the release and distribution readiness lane. The repo now
  has the adoption packet needed for install/try, GitHub Action use, PR review,
  agent handoff, browser-to-native transition, release evidence reading, and a
  recorded "no wrapper yet" release-readiness receipt decision. Future work
  should start from a fresh consumer, artifact, workflow, or product gap rather
  than extending this lane by inertia.
