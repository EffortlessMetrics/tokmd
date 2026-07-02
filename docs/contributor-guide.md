# Contributor Guide

Use this guide when you want to make a first useful contribution without
reading the whole repository history first. It points to the durable docs that
own deeper details instead of duplicating them.

For full policy, hooks, crate layout, testing notes, and release guidance, see
the top-level [CONTRIBUTING.md](../CONTRIBUTING.md).

## Before You Start

Install or confirm:

- a recent stable Rust toolchain;
- Git;
- optional Nix support if you use the repository dev shell;
- optional `cargo-insta` when working on golden snapshots;
- optional `cargo-mutants` or `cargo-fuzz` only when a task specifically needs
  mutation or fuzz evidence.

Then build once from the repository root:

```bash
cargo build
```

On Windows, prefer the repository wrappers for formatting:

```bash
cargo fmt-check
cargo fmt-fix
```

The full workspace can exceed Cargo's formatter argv budget on Windows when
using plain `cargo fmt --all`.

## Choose A First Change

Start with one small user, contributor, or maintainer problem. Good first
changes usually fit one of these shapes:

| Change | First docs to read | Typical proof |
| --- | --- | --- |
| CLI help or error text | [reference-cli.md](reference-cli.md), [start-here.md](start-here.md) | targeted CLI test plus affected proof |
| Documentation fix | nearby doc plus [README.md](README.md) | docs check, doc-artifacts check when source-of-truth docs change |
| Receipt or schema behavior | [SCHEMA.md](SCHEMA.md), [schema.json](schema.json) | targeted test, schema update, affected proof |
| Analysis behavior | [architecture.md](architecture.md), [testing.md](testing.md) | targeted crate tests plus affected proof |
| Review or proof evidence | [review-packet.md](review-packet.md), [cockpit-proof-evidence.md](cockpit-proof-evidence.md) | affected proof and verifier receipts |

Avoid starting from broad cleanup. If an issue lists many gaps, pick one
reviewable packet and reference the issue rather than claiming the whole issue
is complete.

## Repository Roles

Normal development happens in `EffortlessMetrics/tokmd-swarm`. The publication
repository, `EffortlessMetrics/tokmd`, owns release, publish, signing, tags, and
`v1` alias authority.

The steady-state loop is:

```text
tokmd-swarm: branch, PR, required Tokmd Rust Result check, squash merge
tokmd: import swarm work by merge commit at publication checkpoints
tokmd-swarm: fast-forward to the publication merge commit
```

Do not use orphan content-sync PRs for normal work. See
[ci/swarm-routing.md](ci/swarm-routing.md) for the graph rules and current
proof.

## Make A PR-Sized Change

1. Start from current `main`.
2. Create a short branch name such as `docs/contributor-guide` or
   `fix/context-error-message`.
3. Change only the files needed for the selected packet.
4. Add or update tests when behavior changes.
5. Update docs, schemas, policy, or generated artifacts when they own the
   contract you changed.
6. Run the narrowest useful proof first, then broaden only as needed.

Useful local proof commands:

```bash
cargo test
cargo xtask docs --check
cargo xtask doc-artifacts --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan.json
git diff --check
```

Use `cargo xtask doc-artifacts --check` when the change touches
`.tokmd-spec/**`, source-of-truth docs, specs, plans, ADRs, templates,
`.jules/goals/**`, or documentation-control policy. For routine docs, it is
still useful because it proves the docs-control surface stayed consistent.

## What To Expect After Opening A PR

A normal swarm PR should wait for the required aggregate:

```text
Tokmd Rust Result
```

The route and runner-specific jobs are implementation details. They may skip
depending on repository, event trust, runner availability, or selected route.

The PR body should say:

- what changed;
- what it proves;
- what it does not prove;
- which commands or hosted checks ran;
- what rollback would be.

If the change is release-facing, do not publish, tag, move the `v1` alias,
create a GitHub release, push images, or rotate external-service state from a
swarm PR. Those actions belong to the publication repo and require explicit
release work.

## Key Concepts

**Receipts** are stable artifacts that let humans and tools inspect a repository
or workflow without relying on terminal logs.

**Determinism** matters because receipt diffs should explain real changes, not
map ordering, path separator, or timestamp noise.

**Proof is scoped.** A targeted test can be enough for a small change, but the
PR should name the boundary. Advisory evidence such as fast proof, scoped
coverage, mutation, or Codecov upload does not become required unless policy
changes deliberately.

**Source-of-truth docs have roles.** Proposals own why, specs own behavior,
ADRs own durable decisions, plans own sequencing, checked TOML owns
machine-enforced rules, and PR bodies own review-local evidence. See
[source-of-truth.md](source-of-truth.md).

## Where To Go Next

| Need | Read |
| --- | --- |
| First user workflow | [start-here.md](start-here.md) |
| Install and try the CLI | [install-and-try.md](install-and-try.md) |
| Architecture and crate tiers | [architecture.md](architecture.md) |
| Testing strategy | [testing.md](testing.md) |
| Debug failed tests or CI evidence | [debugging.md](debugging.md) |
| Current operating mode | [NEXT.md](NEXT.md) |
| Swarm/publication topology | [ci/swarm-routing.md](ci/swarm-routing.md) |
| Artifact meanings | [artifacts.md](artifacts.md) |
| CLI flags | [reference-cli.md](reference-cli.md) |
| Release-facing evidence | [publishing-evidence.md](publishing-evidence.md) |
