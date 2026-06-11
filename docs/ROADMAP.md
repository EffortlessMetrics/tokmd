# tokmd-swarm Roadmap

`tokmd-swarm` is the agent workbench for advancing `tokmd` safely. Its job is
not to generate motion; its job is to turn product, workflow, release, and
evidence gaps into narrow, reviewable PRs.

The current mode is selection-first. Closed lanes stay closed unless fresh
evidence names a real consumer, missing artifact, workflow pain, or product gap.

## Current Status

The generated PR drain is complete. Proof, AST, browser, release-readiness,
publishing-evidence, user-path evidence, artifact glossary, and first-pass
product-readiness lanes are closed.

The Rust-native proof control plane is in routine-observation mode. Fast proof,
scoped coverage, mutation, coverage telemetry, and Codecov upload remain
advisory unless maintainers deliberately promote them with fresh verified
decision evidence.

The active product lane is PR evidence packet workflows: make the
`sensors/tokmd/` packet easy to generate from one local CLI command and one
GitHub Action step before adding more analysis.

## Roadmap Principles

1. Product gaps outrank process polish.
2. Workflow pain outrank speculative architecture work.
3. User-facing adoption outranks internal documentation churn.
4. Evidence artifacts should be made more consumable before inventing new ones.
5. AST work remains shadow-only until comparison evidence justifies public
   behavior or schema changes.
6. Proof work remains advisory unless maintainers explicitly promote it.
7. Each PR should be small enough to review and revert.
8. Each new lane must state:
   - the consumer,
   - the gap,
   - the artifact or behavior being changed,
   - the proof boundary,
   - the rollback path,
   - the non-goals.

## Near-Term Roadmap

### Active Lane: PR Evidence Packet Workflows

**Goal:** Make `tokmd` useful in pull request workflows and local review by
producing the same bounded evidence packet from a one-command CLI path and a
GitHub Action path.

**Why now:** `tokmd` already has the packet ingredients: scoped analysis,
context, optional syntax evidence, a manifest, and review priority. The next
gap is consumption. Users should not need to script five commands correctly or
build `tokmd` in every repository.

**Candidate work packets:**

1. Workflow contract and support model
   - Document local CLI, GitHub Action, GHCR, and Cargo fallback paths.
   - Record packet status semantics, fail-on behavior, artifact layout, and
     non-claims.
2. One-command CLI orchestration
   - Add a thin packet-generation command over existing receipts.
   - Keep base/head/path scope consistent across artifacts.
3. GitHub Action packet path
   - Download/cache a prebuilt binary by version.
   - Upload `sensors/tokmd/`, write a job summary, and expose status outputs.
4. GHCR secondary runtime
   - Verify public pull, labels, `git`/certs, entrypoint behavior, and packet
     smoke before documenting it as a supported runtime.

**Do not:**

- add a new analysis engine,
- claim UB detection, reachability proof, safety proof, or merge readiness,
- make GHCR the primary user experience,
- promote advisory proof, Codecov upload, or release mutation by default.

**Done when:**

- local users can generate `sensors/tokmd/` with one command,
- PR workflows can generate and upload the same packet with one Action step,
- packet status drives documented `fail-on` behavior,
- GHCR is either verified as a secondary runtime or explicitly marked pending.

### Lane 0: Release and Distribution Verification

**Goal:** Make it easy for maintainers and users to verify that released
artifacts are visible, installable, and mapped to the expected version.

**Why now:** Recent release evidence is strong, but GHCR visibility for
`v1.13.1` still needs maintainer-access verification. The crates.io and GitHub
release install paths are verified; GHCR public visibility remains a concrete
consumer-facing release gap, not speculative release automation.

**Candidate work packets:**

1. GHCR visibility verification guide
   - Document the expected GHCR tags.
   - Document who can verify package visibility.
   - Record the exact commands maintainers should run.
   - Preserve the boundary: do not rewrite an existing release.
2. Release artifact verification checklist
   - CLI binary install check.
   - crates.io package visibility check.
   - GitHub release asset checksum check.
   - Docker/GHCR tag visibility check.
   - WASM artifact visibility check if applicable.
3. Post-release evidence index
   - Link release workflow run, crates publish evidence, Docker/GHCR evidence,
     checksums, and any package visibility notes.
   - Keep it as documentation unless a repeated consumer proves a need for a
     machine-readable receipt.

**Do not:**

- rewrite past releases,
- add a wrapper receipt without a named consumer,
- automate npm/GHCR changes without maintainer approval,
- promote release proof gates by default.

**Done when:**

- maintainers can verify a release from one short guide,
- current GHCR visibility uncertainty is resolved or explicitly recorded,
- future release checks have a durable path.

### Lane 1: User-Facing CLI Friction

**Goal:** Improve the experience of running `tokmd` directly: help text,
actionable errors, progress, config explainability, and command discovery.

**Why now:** This is the strongest product lane because it benefits every user
and is less risky than schema or architecture changes.

**Candidate work packets:**

1. CLI help examples
   - Add practical examples to command help for `analyze`, `diff`, `context`,
     `gate`, `cockpit`, `handoff`, `run`, and `export`.
   - Keep examples short and tested where practical.
2. Error context pass
   - Add missing path/config context around filesystem, git, and parsing
     failures.
   - Extend existing hint patterns only when they produce clear recovery
     advice.
3. Config explainability
   - Add or plan a `--show-config` / `doctor`-style surface only if current
     config layering is demonstrably confusing to users.
   - Prefer docs first if implementation risk is high.
4. Progress consistency
   - Add progress messaging to long-running commands where missing.
   - Preserve stdout as machine-readable output and put progress on stderr.

**Do not:**

- change receipt schemas,
- change JSON/JSONL output,
- add command grouping that fights Clap unless the benefit is proven,
- create a separate review command yet.

**Done when:**

- common commands are discoverable from `--help`,
- common failure modes give recovery hints,
- long operations visibly progress,
- output remains script-safe.

### Lane 2: Review Evidence Consumption

**Goal:** Make cockpit/review packets easier for maintainers, agents, and CI
readers to consume without adding a separate review orchestrator.

**Why now:** Recent PRs improved imported proof metadata. The next useful step
is consumption: making those artifacts easier to read and triage.

**Candidate work packets:**

1. Review packet reading order
   - Clarify when to read `comment.md`, `review-map.md`, `evidence.json`,
     proof evidence, and imported artifacts.
   - Include examples for passed, advisory-missing, and failed evidence states.
2. Evidence field glossary
   - Explain `run_id`, `run_attempt`, `run_url`, `workflow`, `event_name`,
     `ref_name`, proof source, advisory status, and required/non-required
     boundaries.
3. Hosted-comment troubleshooting
   - Document common GitHub comment/update failures.
   - Keep `tokmd cockpit` as the implementation surface.
4. Small rendering improvements
   - Improve review-map priority explanations.
   - Improve missing-evidence wording.
   - Preserve machine fields and schema compatibility.

**Do not:**

- add `tokmd review` yet,
- promote advisory evidence into a required gate,
- require Codecov upload by default,
- invent another evidence artifact unless existing packet artifacts cannot
  answer a named consumer.

**Done when:**

- a maintainer can open a review packet and understand what to trust first,
- missing advisory proof is not confused with required failure,
- imported proof metadata remains traceable.

### Lane 3: Measured Performance and CI Feedback

**Goal:** Improve developer feedback speed and runtime performance only where
measurement shows a bottleneck.

**Why now:** Performance and CI issues exist, but previous guidance requires
bounded timing receipts before optimization. This lane should be measurement-led,
not speculative.

**Candidate work packets:**

1. CI feedback timing refresh
   - Measure current PR feedback bottlenecks.
   - Compare to older nextest/caching research.
   - Decide whether the old research is still accurate before implementation.
2. `cargo xtask perf-smoke` receipts
   - Add or refresh small repeatable timing receipts for common commands.
   - Use them to justify future optimization PRs.
3. Narrow clone/allocation cleanup
   - Only touch clone hot paths when profiling or perf-smoke receipts identify
     them.
   - Keep API churn minimal.
4. File I/O cache investigation
   - Start with an evidence-gathering plan.
   - Avoid implementing a large cache layer until repeated file-read cost is
     measured.

**Do not:**

- adopt large CI restructuring from stale research without fresh baseline data,
- add persistent caching without a consumer and invalidation story,
- change proof gates to make CI "look faster,"
- optimize AST paths before product behavior needs them.

**Done when:**

- current bottlenecks are known,
- at least one low-risk measured improvement lands,
- perf evidence is repeatable by maintainers.

### Lane 4: Documentation That Serves Adoption

**Goal:** Fill adoption and contributor gaps that directly help users or new
contributors succeed.

**Why now:** Recent docs work improved source-of-truth and artifact vocabulary.
The next docs should be practical, not meta.

**Candidate work packets:**

1. Contributor quickstart
   - Short "first useful contribution" guide.
   - Link to deeper architecture/testing docs.
   - Avoid duplicating the full CONTRIBUTING file.
2. "How to extend tokmd"
   - Add guides for adding an enricher, preset, output format, or language
     support.
   - Include a small concrete example.
3. Debugging guide
   - Common local test failures.
   - Snapshot workflow.
   - Property-test shrink output.
   - Git/worktree failures.
   - CI mismatch troubleshooting.
4. Crate README examples
   - Continue only where current crate layout actually exists.
   - Do not resurrect obsolete leaf-crate names.

**Do not:**

- create more source-of-truth hierarchy docs unless the hierarchy changes,
- update closed plans without fresh reason,
- close stale issues by pretending obsolete crate names still exist.

**Done when:**

- a new contributor can find one starter path,
- extender docs explain where code actually lives now,
- stale crate-layout docs are either updated or explicitly marked historical.

### Lane 5: Browser/WASM Product Continuation

**Goal:** Continue browser-safe product value without pretending browser mode has
host/git capabilities.

**Why now:** The v1.11 browser runtime polish lane is closed, but browser/WASM
is still one of the clearest user-facing product surfaces.

**Candidate work packets:**

1. Browser capability matrix refresh
   - Verify the current browser-supported modes and presets.
   - Keep unsupported host-backed features explicit.
2. Rootless preset feasibility
   - Re-evaluate `health`, `topics`, `architecture`, `security`, `identity`,
     and `supply` for browser-safe operation.
   - Start with a proposal, not implementation.
3. Browser examples
   - Add practical browser-runner examples for public repo analysis.
   - Show authenticated fetch boundaries and cache behavior.
4. WASM embed docs
   - TypeScript/bundler guidance if the package surface is stable enough.

**Do not:**

- browser-enable git-history metrics without a backend or credible browser git
  design,
- hide capability misses,
- expose unstable schema changes from browser-only paths.

**Done when:**

- browser users know exactly what works,
- the next rootless preset candidate has evidence,
- browser examples match actual behavior.

### Lane 6: AST Shadow Evidence

**Goal:** Continue AST work only as shadow evidence until it justifies public
behavior.

**Why now:** AST foundation exists, but previous decisions concluded function
boundary candidate status is "not yet." More AST work should be evidence-led.

**Candidate work packets:**

1. Corpus selection proposal
   - Identify a broader, representative corpus.
   - Define mismatch categories before running comparisons.
2. Shadow comparison expansion
   - Add evidence for function boundaries, imports, and control-flow landmarks.
   - Preserve developer-facing artifact status.
3. Browser/WASM AST feasibility note
   - Clarify parser size, feature flags, fallback behavior, and capability
     reporting implications before any browser work.
4. Candidate decision refresh
   - Only after broader evidence exists.
   - Decide: public candidate, continue shadow, or stop.

**Do not:**

- change public receipts,
- make AST default,
- add public schema fields,
- treat control-flow evidence as product behavior,
- do AST architecture work without product-surface proposal.

**Done when:**

- AST evidence is broader, repeatable, and classified,
- maintainers can make a clear candidate decision,
- fallback and rollback are documented.

## Later Horizons

### MCP / Server Mode

MCP remains a valid future direction, but it should wait until current artifacts
and command semantics are stable enough to expose as long-lived tools/resources.

Start only with a proposal that names:

- target MCP clients,
- supported tools,
- resource model,
- streaming behavior,
- auth/security boundary,
- receipt/schema compatibility,
- cancellation behavior,
- rollback path.

Likely first slice:

- expose existing `tools` schema and read-only receipt resources;
- no new analysis behavior;
- no remote execution service.

### Plugin System

Plugin work should remain parked until:

- the host/plugin artifact contract is known,
- schema extension policy is defined,
- security boundaries are explicit,
- at least one real plugin consumer exists.

### Cloud Dashboard / Historical Service

This is out of scope for the swarm unless product ownership selects it. The
swarm can prepare receipts and evidence, but should not start service/platform
work by inertia.

## Parking Lot

These are intentionally not active:

- Proof promotion into required gates.
- Default Codecov upload.
- Public AST schema changes.
- Separate `tokmd review` command.
- New wrapper receipts for release/publishing readiness.
- Architecture consolidation without a fresh owner-module problem.
- Broad generated coverage PRs.
- Generic process documentation without a consumer.

## Lane Selection Template

Every new lane should start with this block:

```markdown
## Proposal: <lane name>

### Consumer

Who needs this?

### Gap

What can they not do today?

### Current evidence

What issue, PR, failure, release check, user report, or artifact shows the gap?

### Proposed slice

What is the smallest useful PR?

### Artifacts touched

Which docs, receipts, schemas, workflows, commands, or crates change?

### Proof boundary

What checks prove the slice, and what remains advisory?

### Non-goals

What are we explicitly not doing?

### Rollback

How do we revert safely?
```

## Merge Policy for Swarm PRs

A swarm PR is desirable when it satisfies at least one of:

- closes or advances a live issue,
- fixes a broken or fragile workflow,
- improves a user-facing path,
- makes existing evidence easier to consume,
- records fresh decision evidence,
- removes obsolete or misleading documentation,
- adds measured performance improvement.

A swarm PR is suspicious when it only:

- restates existing process,
- edits closed plans without new evidence,
- adds another artifact wrapper without a consumer,
- expands proof/control-plane behavior by inertia,
- updates stale crate names without reconciling current architecture.
