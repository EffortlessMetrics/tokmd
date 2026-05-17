# Plan: Release And Distribution Readiness

- Status: active
- Related proposal:
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
   - Status: pending.
   - Add `docs/examples/real-user-path-smoke-run.md` after running the current
     affected/proof-plan, cockpit, review-packet-check, and handoff sequence on
     a small real PR or branch range.
   - Record what was clear, what was confusing, and what was fixed or deferred.
   - Do not commit generated packets, CI logs, or large artifact dumps.
4. Add an agent handoff prompt template.
   - Status: pending.
   - Add `docs/agent-workflows/handoff-prompt.md` as a short copy-ready prompt
     for Codex, Claude, Cursor, or another coding agent consuming
     `.handoff/work-order.md`, `.handoff/code.txt`, `review-links.json`, and
     `proof-links.json`.
   - Keep it as a consumer bridge, not a second planning system.
5. Add a handoff work-order contract test.
   - Status: pending.
   - Cover the user-facing sections that make `.handoff/work-order.md` useful
     to agents: changed surfaces, review evidence, proof expectations,
     missing/stale/degraded evidence, and agent stop conditions.
   - Preserve handoff as a linker and summarizer for external evidence, not a
     verifier for proof or review packets.
6. Add browser-to-native adoption guidance if current browser docs still leave
   the trial-to-native path ambiguous.
   - Status: pending.
   - Keep browser mode as a no-install trial lens.
   - Keep native mode as the review, proof, and handoff instrument.
7. Add a release evidence quickstart.
   - Status: pending.
   - Compose existing publishing evidence, version consistency, affected proof,
     and proof-plan commands.
   - Make clear that these are pre-release evidence, not release mutation or
     release approval.
8. Decide whether a release-readiness wrapper receipt is needed.
   - Status: pending.
   - Start with a proposal, not code.
   - The default answer remains "no wrapper yet" unless the install, Action,
     smoke-run, or release evidence guides prove that existing artifacts are
     insufficient for a named consumer.
9. Compress the README first-run path after the adoption guides exist.
   - Status: pending.
   - Keep README above the fold focused on install, inspect, review, handoff,
     and CI entry points.
   - Link deeper reference pages instead of explaining the control plane inline.
10. Close the lane.
    - Status: pending.
    - Close only when install/try, Action adoption, real smoke evidence, agent
      prompt guidance, handoff contract coverage, browser-to-native guidance,
      release evidence quickstart, and the release-readiness receipt decision
      are recorded.

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
