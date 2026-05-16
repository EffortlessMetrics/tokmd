# User Paths

Use this page when you know the job, but not which `tokmd` artifact to trust
first. It is a consumption map, not a command reference.

For artifact definitions, use the [Artifact glossary](artifacts.md). For the
first guided walkthrough, use [Start Here](start-here.md). For small physical
layouts, use [Sample artifact trees](examples/README.md).

## At A Glance

| User job | Run first | Primary artifact | Open first | Meaning |
| --- | --- | --- | --- | --- |
| Inspect repo | `tokmd --format md --top 8` | Markdown summary | terminal output | Repo shape, language mix, and largest surfaces. |
| Review PR | `tokmd cockpit --base origin/main --head HEAD --review-packet-dir .tokmd/review` | `.tokmd/review/` | `.tokmd/review/review-map.md` | Review order, evidence state, missing evidence, and reproduction commands. |
| Prepare agent handoff | `tokmd handoff --preset risk --budget 128k --strategy spread --out-dir .handoff` | `.handoff/` | `.handoff/work-order.md` | Bounded source/context bundle plus agent task map. |
| Read CI proof evidence | `cargo xtask affected ...` then `cargo xtask proof --profile affected ... --plan` | `target/proof/` | `affected.json`, then `proof-plan.json` | Changed files, matched proof scopes, and required/advisory proof expectations. |
| Try browser mode | Browser runner | downloaded browser-safe receipt | UI summary | No-install repo inspection over browser-supported inputs. |
| Check publishing evidence | `cargo xtask publish-surface --json --verify-publish` | publish-surface JSON/stdout | command output or saved JSON | Package-surface and publish-readiness evidence before release mutation. |

## Inspect Repo

Run:

```bash
tokmd --format md --top 8
```

Artifact: terminal Markdown output.

Open first: the command output.

Means:

- the repo's language and size shape;
- the largest surfaces worth inspecting;
- a deterministic starting point for wider analysis.

Does not mean:

- tests passed;
- the repo is safe to release;
- a PR is ready to merge.

Next action:

- Use `tokmd analyze --preset risk --format md` when you need derived risk,
  effort, complexity, or git-backed signals.
- Use `tokmd export --format jsonl` when another tool needs stable file rows.

## Review PR

Run:

```bash
tokmd cockpit \
  --base origin/main \
  --head HEAD \
  --review-packet-dir .tokmd/review
```

Verify packet integrity in contributor checkouts:

```bash
cargo xtask review-packet-check \
  --dir .tokmd/review \
  --json target/tokmd/review-packet-check.json
```

Artifact: `.tokmd/review/`.

Open first:

1. `.tokmd/review/review-map.md`
2. `.tokmd/review/comment.md`
3. `.tokmd/review/evidence.json`
4. `target/tokmd/review-packet-check.json`

Means:

- what changed;
- what to review first;
- why the item was prioritized;
- what evidence is present, missing, stale, degraded, skipped, or advisory;
- which command reproduces the evidence claim.

Does not mean:

- merge approval;
- advisory proof became required;
- missing evidence is passing proof.

Next action:

- Run the reproduction commands shown in `review-map.md`.
- Inspect `evidence.json` for missing or degraded evidence before claiming the
  packet is complete.
- Use the verifier receipt before trusting packet-local hashes.

## Prepare Agent Handoff

Run a plain bundle:

```bash
tokmd handoff \
  --preset risk \
  --budget 128k \
  --strategy spread \
  --out-dir .handoff
```

When review and proof artifacts exist, link them:

```bash
tokmd handoff \
  --preset risk \
  --budget 128k \
  --strategy spread \
  --review-packet-dir .tokmd/review \
  --review-packet-check target/tokmd/review-packet-check.json \
  --affected target/proof/affected.json \
  --proof-plan target/proof/proof-plan.json \
  --out-dir .handoff
```

Artifact: `.handoff/`.

Open first:

1. `.handoff/work-order.md`
2. `.handoff/manifest.json`
3. `.handoff/code.txt`
4. `.handoff/review-links.json` and `.handoff/proof-links.json` if present

Means:

- the bounded source bundle selected for the agent;
- what external review/proof evidence is linked;
- where the agent should start;
- what proof expectations should be checked before returning.

Does not mean:

- the whole repo fit in the bundle;
- linked review/proof artifacts were verified by handoff;
- planned proof has passed.

Next action:

- Give the agent `work-order.md` first.
- Treat missing, stale, degraded, or unavailable evidence as a task, not as a
  pass.
- Verify linked review/proof artifacts with their own checkers.

## Read CI Proof Evidence

Plan affected proof:

```bash
cargo xtask affected \
  --base origin/main \
  --head HEAD \
  --json-output target/proof/affected.json

cargo xtask proof \
  --profile affected \
  --base origin/main \
  --head HEAD \
  --plan \
  --plan-json target/proof/proof-plan.json \
  --evidence-json target/proof/proof-evidence.json
```

For observation aggregates, read the decision packet:

```bash
cargo xtask proof-observation-status \
  --observations-dir target/proof-observations \
  --json target/proof-observations/proof-observation-decision.json \
  --summary-md target/proof-observations/proof-observation-decision.md
```

Artifact: `target/proof/` and `target/proof-observations/`.

Open first:

1. `target/proof/affected.json`
2. `target/proof/proof-plan.json`
3. `target/proof/proof-evidence.json`
4. `target/proof-observations/proof-observation-decision.md` when collecting
   observation evidence

Means:

- which files changed;
- which proof scopes matched;
- which commands are required versus advisory;
- which criteria are met or missing in observation evidence.

Does not mean:

- planned proof executed;
- advisory coverage or mutation is a gate;
- Codecov upload is enabled by default;
- promotion criteria changed.

Next action:

- Resolve unknown files before relying on scoped proof.
- Run required proof when the work needs executed evidence.
- Verify source receipts and status packets with their matching checkers.

## Try Browser Mode

Run: open the browser runner and load GitHub or local files.

Artifact: browser-safe receipt download.

Open first: the browser UI summary, then the downloaded receipt if you need to
save evidence.

Means:

- no-install inspection for browser-supported inputs;
- language/module/file export and browser-safe analysis where capabilities are
  available;
- a quick trial before installing native `tokmd`.

Does not mean:

- native filesystem behavior;
- git-history enrichers;
- cockpit review packets;
- gates, context, handoff, or AST capability.

Next action:

- Move to native `tokmd cockpit` for PR review.
- Move to native `tokmd handoff` when preparing agent context.
- Check [Browser runner](browser.md) and the capability matrix before relying
  on a browser result.

## Publish Or Release Safely

Run:

```bash
cargo xtask publish-surface --json --verify-publish
cargo xtask version-consistency
```

Artifact: publish-surface JSON/stdout and version-consistency output.

Open first: the publish-surface summary and `violations` fields, then the
version-consistency output.

Means:

- package-surface classification;
- non-dev workspace closure;
- package-list and publish-surface checks;
- version alignment for the checked workspace state.

Does not mean:

- crates were published;
- a tag or GitHub release was created;
- release workflow artifacts exist;
- release mutation is approved.

Next action:

- Use [Publishing evidence](publishing-evidence.md) for the release-facing
  reading order.
- Treat release mutation as a separate explicit decision.
- Pair publishing evidence with affected proof when release metadata or
  workflow files changed.
