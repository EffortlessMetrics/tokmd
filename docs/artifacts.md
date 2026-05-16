# Artifact Glossary

Use this page when a `tokmd`, cockpit, proof, CI, or handoff workflow gives you
an artifact name and you need to know what it means before acting on it.

This is a user-facing dictionary, not the formal schema source. Formal receipt
contracts live in the schema files, command docs, and verifier docs linked from
each section.

## Reading Rules

- A receipt means a tool wrote a structured fact. It is not automatically a
  merge verdict.
- A verifier receipt means a checker inspected a specific artifact or packet.
  It does not prove unrelated files, external artifacts, or future runs.
- A proof plan is expected work. A proof run summary is executed work.
- Advisory evidence remains advisory unless a maintainer explicitly promotes it
  through policy.
- Packet-local artifacts can be hash-verified by their packet verifier.
  Linked artifacts remain external evidence handles.

## Open First

| Job | Open first | Why |
| --- | --- | --- |
| Inspect a repo | Markdown command output or `receipt.json` | Gives the smallest stable repo-shape fact set. |
| Review a PR | `.tokmd/review/comment.md`, then `.tokmd/review/review-map.md` | Gives the summary, review-first order, evidence state, and reproduction commands. |
| Verify a review packet | `target/tokmd/review-packet-check.json` | Shows whether packet-local manifest paths, schemas, and hashes were checked. |
| Understand CI evidence | `target/proof/affected.json`, then `target/proof/proof-plan.json` | Shows changed files, matched proof scopes, and selected proof commands. |
| Check executed required proof | `target/proof-run/proof-run-summary.json` | Shows which required proof commands actually ran and passed or failed. |
| Prepare an agent | `.handoff/work-order.md`, then `.handoff/manifest.json` | Gives the agent task map, linked evidence summary, bundle index, and guardrails. |
| Audit docs/source-of-truth changes | `target/docs/doc-artifacts-check.json` | Shows source-of-truth artifact shape, links, active-goal state, and policy checks. |

## Core Repo And Change Receipts

| Artifact | Usual path | Writer | Means | Does not mean | Verify or inspect |
| --- | --- | --- | --- | --- | --- |
| `receipt.json` | `.runs/<name>/receipt.json` or a context bundle sidecar | `tokmd run`, context bundle workflows | Inventory for a saved run, usually pointing at generated language, module, export, or analysis artifacts. | It is not proof that a PR is safe, tested, or reviewed. | Use `tokmd diff <old>/receipt.json <new>/receipt.json` or inspect schema docs in [SCHEMA](SCHEMA.md). |
| `analysis.json` | Output from `tokmd analyze --format json` or run directories | `tokmd analyze`, `tokmd run --analysis ...` | Deterministic derived analysis facts such as risk, complexity, churn, dependencies, topics, or supply signals depending on preset. | It is not a linter, SAST result, or legal/license opinion. | Re-run the same analyze command and compare stable JSON. |
| `diff.json` | `tokmd diff --format json` output when redirected or saved | `tokmd diff` | Structured before/after change facts between receipts, runs, or refs. | It is not review prioritization by itself. | Re-run `tokmd diff <before> <after> --format json`. |
| `gate.json` | `tokmd gate --format json` output when saved | `tokmd gate` | Policy evaluation result for a receipt and a TOML policy or baseline ratchet. | It does not replace the underlying test/build tools named by policy. | Re-run `tokmd gate ... --format json` with the same policy and receipt. |
| `baseline.json` | `.tokmd/baseline.json` or user-selected baseline path | `tokmd baseline`, `tokmd gate` ratchet flows | Stored comparison point for ratchets and drift checks. | It is not a current scan; it can become stale. | Recreate from the intended reference state before relying on it. |

## Cockpit Review Packet

| Artifact | Usual path | Writer | Means | Does not mean | Verify or inspect |
| --- | --- | --- | --- | --- | --- |
| `manifest.json` | `.tokmd/review/manifest.json` | `tokmd cockpit --review-packet-dir` | Packet inventory: artifact paths, schemas, generated metadata, and BLAKE3 hashes for packet-local files. | It does not verify itself or external linked receipts. | `cargo xtask review-packet-check --dir .tokmd/review`. |
| `cockpit.json` | `.tokmd/review/cockpit.json` | `tokmd cockpit --review-packet-dir` | Full cockpit receipt with change surface, composition, contracts, review plan, and gate data. | It is not the easiest first review surface for humans. | Open when `comment.md` or `review-map.md` needs deeper source data. |
| `evidence.json` | `.tokmd/review/evidence.json` | `tokmd cockpit --review-packet-dir` | Evidence availability, gate status, imported proof/doc evidence, and missing/degraded/stale/skipped/unavailable buckets. | Missing evidence is not passing proof; advisory evidence is not a required gate. | Validate through `review-packet-check`; inspect when the review map flags missing or stale evidence. |
| `review-map.json` | `.tokmd/review/review-map.json` | `tokmd cockpit --review-packet-dir` | Machine-readable review-first routing with item reasons, evidence status, refs, proof refs, and reproduction commands. | It is not a merge queue or reviewer assignment system. | Validate through `review-packet-check`; use for tools and agents. |
| `review-map.md` | `.tokmd/review/review-map.md` | `tokmd cockpit --review-packet-dir` | Human review work order: what to inspect first, why, what evidence exists or is missing, and how to reproduce evidence. | It does not execute the reproduction commands for you. | Open after `comment.md`; run the listed commands before claiming evidence is repaired. |
| `comment.md` | `.tokmd/review/comment.md` | `tokmd cockpit --review-packet-dir` | Compact PR-comment-ready summary that points at the packet artifacts. | It is not the whole packet and is intentionally short. | Follow its links to `review-map.md`, `evidence.json`, and `cockpit.json`. |
| `proof/*.json` | `.tokmd/review/proof/*.json` | `tokmd cockpit` when proof inputs are supplied | Packet-local copies of imported proof artifacts, listed and hash-verified by `manifest.json`. | Their presence does not promote proof gates or prove unrelated scopes. | Verify packet hashes with `review-packet-check`; inspect proof freshness and scope fields. |
| `docs/doc-artifacts-check.json` | `.tokmd/review/docs/doc-artifacts-check.json` | `tokmd cockpit --doc-artifacts-check ...` | Packet-local copy of documentation-control evidence imported into the review packet. | It is not generated by cockpit; cockpit only imports it. | Reproduce with `cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json`. |

## Review Packet Verification

| Artifact | Usual path | Writer | Means | Does not mean | Verify or inspect |
| --- | --- | --- | --- | --- | --- |
| `review-packet-check.json` | `target/tokmd/review-packet-check.json` | `cargo xtask review-packet-check --json <path>` | Verifier receipt for a specific review packet: schema checks, packet-local path checks, artifact count, and hash verification. | It does not verify artifacts outside the packet, hosted comment copies, or future packet mutations. | Open this before trusting a packet; regenerate after any packet-local artifact changes. |

## Proof And CI Evidence

| Artifact | Usual path | Writer | Means | Does not mean | Verify or inspect |
| --- | --- | --- | --- | --- | --- |
| `affected.json` | `target/proof/affected.json` | `cargo xtask affected --json-output <path>` | Changed files mapped to `ci/proof.toml` scopes, including unknown files. | It does not run any proof commands. | Unknown files should be resolved before trusting scoped proof routing. |
| `proof-plan.json` | `target/proof/proof-plan.json` | `cargo xtask proof --plan --plan-json <path>` | Planned proof commands for affected scopes, including required and advisory commands. | It is planned evidence, not executed proof. | Use with `proof-evidence.json` or run required proof explicitly. |
| `proof-evidence.json` | `target/proof/proof-evidence.json` | `cargo xtask proof --plan --evidence-json <path>` | Machine-readable planned evidence state for coverage, mutation, and other proof families. | Planned advisory evidence is not a pass. | Inspect status fields; execute and verify proof separately when needed. |
| `proof-plan.md` | `target/proof/proof-plan.md` | `cargo xtask proof --plan --summary-md <path>` | Human-readable summary of the proof plan for PR comments and CI summaries. | It is not the source artifact for machine routing. | Compare with `proof-plan.json` if commands or counts matter. |
| `mutation-scope.json` | `target/mutation/mutation-scope.json` | `cargo xtask mutation-scope --json-output <path>` | Mutation workflow scope selection: base/head refs, production Rust candidate files, selected files, file-count cap, and scope-exceeded status. | It does not run `cargo-mutants`, summarize survivors, or make mutation testing required. | Inspect `scope_exceeded`, `all_changed_files`, and `changed_files`; pair with `mutants-summary.json` after execution. |
| `mutants-summary.json` | `mutants-summary.json` in the manual mutation workflow artifact set | `cargo xtask mutation-summary --json-output <path>` after `cargo-mutants` execution | Mutation workflow result summary: commit, base ref, selected scope, status, survivor list, killed count, timeout count, and unviable count. | It does not make mutation testing required, replace cargo-mutants output, or prove unrelated files were mutation-tested. | Inspect `status` and `survivors`; pair with `mutation-scope.json` to see what was selected. |
| `proof-run-summary.json` | `target/proof-run/proof-run-summary.json` or `target/proof/proof-run-summary.json` | `cargo xtask proof --run-required --proof-run-summary <path>` | Executed required proof commands, statuses, guard reason, changed files, and unknown files. | It excludes advisory coverage/mutation commands and does not make advisory proof required. | `cargo xtask proof-run-artifacts-check --proof-run-summary <path>`. |
| `proof-run-artifacts-check.json` | `target/proof-run/proof-run-artifacts-check.json` | `cargo xtask proof-run-artifacts-check --json-output <path>` | Verifier receipt for an executed required proof-run summary: source path, checked counts, guard reason, and validation errors. | It does not execute proof or make advisory evidence required. | Regenerate from the matching `proof-run-summary.json`. |
| `proof-run-observation.json` | `target/proof-run/proof-run-observation.json` | `cargo xtask proof-run-observation --proof-run-summary <path>` | Compact observation derived from a verified required proof-run summary. | It is an observation for collection, not a new proof run. | Verify the source `proof-run-summary.json` first. |
| `executor-summary.json` | `target/proof/executor-summary.json` | `cargo xtask proof --executor-summary <path>` or proof executor workflow | Selected non-required executor commands and their execution/skipped status. | It is advisory and not part of the required aggregate. | Use `proof-artifacts-check` for no-execution artifacts or `proof-execution-artifacts-check` for executed artifacts. |
| `executor-manifest.json` | `target/proof/executor-manifest.json` | `cargo xtask proof --executor-manifest <path>` | Stable manifest for planner-selected executor commands and policy guard state. | It does not by itself prove coverage artifacts exist. | Check it against `executor-summary.json`. |
| `proof-artifacts-check.json` | `target/proof/proof-artifacts-check.json` | `cargo xtask proof-artifacts-check --json-output <path>` | Verifier receipt for planned, non-executed executor summary/manifest consistency. | It does not prove coverage files exist or execute advisory proof. | Regenerate from the matching `executor-summary.json` and `executor-manifest.json`. |
| `proof-execution-artifacts-check.json` | `target/proof/proof-execution-artifacts-check.json` | `cargo xtask proof-execution-artifacts-check --json-output <path>` | Verifier receipt for executed scoped-coverage executor artifacts, including source paths, checked counts, guard reason, and validation errors. | It does not promote scoped coverage or Codecov upload. | Regenerate from the matching executed `executor-summary.json`, `executor-manifest.json`, and coverage artifacts. |
| `proof-executor-observation.json` | `target/proof/proof-executor-observation.json` | `cargo xtask proof-execution-observation ...` | Observation of an executed non-required proof executor run. | It does not promote scoped coverage to a required gate. | Collect with observation-summary tooling before any promotion decision. |
| `proof-observation-decision.json` | `target/proof-observations/proof-observation-decision.json` | `cargo xtask proof-observation-status --json <path>` | Advisory aggregate over supplied proof artifacts: policy state, required/advisory proof counts, freshness, thresholds, criteria met/missing, and reproduction commands. | It does not execute proof, upload coverage, promote gates, or replace source-artifact verifiers. | Check the listed `source_artifacts`, `criteria_missing`, and `reproduce` commands before making a promotion decision. |
| `proof-observation-decision-check.json` | `target/proof-observations/proof-observation-decision-check.json` | `cargo xtask proof-observation-status-check --decision <path> --json <path>` | Verifier receipt for the advisory decision packet: schema/mode, source artifact references, count consistency, policy guardrails, criteria shape, and reproduction commands. | It does not verify the original source receipts or make advisory proof required. | Run after generating `proof-observation-decision.json`; still verify source artifacts with their own checkers. |
| `coverage-receipt.json` | `target/coverage/coverage-receipt.json` | `cargo xtask coverage-receipt` | Inventory and byte-count receipt for coverage artifacts such as JSON, text, and LCOV files. | It does not say coverage is required, sufficient, or uploaded to Codecov. | Inspect artifact paths and byte counts; pair with coverage workflow logs if needed. |
| `doc-artifacts-check.json` | `target/docs/doc-artifacts-check.json` | `cargo xtask doc-artifacts --check --json <path>` | Source-of-truth docs/control-plane checker receipt: required docs, artifact family shape, active-goal links, status vocabulary, and errors. | It does not judge prose quality or merge readiness. | Re-run `cargo xtask doc-artifacts --check` or `cargo xtask docs --check`. |

## Publishing And Release Evidence

| Artifact | Usual path | Writer | Means | Does not mean | Verify or inspect |
| --- | --- | --- | --- | --- | --- |
| Publish-surface JSON | `target/publishing/publish-surface.json` when saved, or stdout | `cargo xtask publish-surface --json --verify-publish` | Package-surface taxonomy, non-dev workspace closure, Cargo package-list checks, and `violations` for the checked workspace state. | It does not publish crates, prove crates.io visibility, create a release, or approve release mutation. | Inspect `summary`, `crates`, `packaging_checks`, and `violations`; rerun the command from the repository root. |
| Version consistency output | Terminal output or hosted `Version consistency` job log | `cargo xtask version-consistency` | Workspace, crate, binding, and release metadata versions are aligned for the checked workspace state. | It does not prove package closure, publish artifacts, or registry state. | Rerun `cargo xtask version-consistency`; pair with publish-surface evidence before release work. |
| `release_metadata` proof scope | `target/proof/affected.json` and `target/proof/proof-plan.json` | `cargo xtask affected ...` and `cargo xtask proof --profile affected --plan ...` | Release metadata or release workflow changes route to version consistency, publish-surface verification, and docs checks. | It does not execute release workflow jobs or publish artifacts. | Confirm `unknown_files` is empty and the `release_metadata` scope selects the expected commands. |
| CI release lane policy | `policy/ci-lane-whitelist.toml` | Maintained policy, validated by `cargo xtask proof-policy --check` | Release and publishing lane intent, owner, trigger, blocking status, evidence, and proof obligation. | It is not a workflow run result and does not mean a release job passed. | Inspect release lanes such as `version_consistency`, `publish_surface`, `release_build`, `release_create`, `release_publish_crates`, and `release_docker`. |
| Release workflow artifacts | Hosted release workflow artifacts and release job logs | `.github/workflows/release.yml` during intentional release runs | Actual release mutation evidence such as built binaries, GitHub release artifacts, crates.io publication logs, or Docker registry output. | They are not produced by pre-release publishing evidence checks and should not be generated without an explicit release decision. | Review the intentional release run, registry state, GitHub release state, and post-release smoke evidence. |

## Handoff And Agent Artifacts

| Artifact | Usual path | Writer | Means | Does not mean | Verify or inspect |
| --- | --- | --- | --- | --- | --- |
| `manifest.json` | `.handoff/manifest.json` | `tokmd handoff` | Authoritative handoff bundle index: inputs, token budget, included/excluded files, capabilities, artifacts, and BLAKE3 hashes. | It does not verify external review or proof receipts linked beside the bundle. | Open first for bundle scope and artifact hashes. |
| `work-order.md` | `.handoff/work-order.md` | `tokmd handoff` | Agent-readable task map, selected-file summary, linked-evidence summary, and guardrails. | It is not a verifier and should not be treated as proof execution. | Give this to the coding agent first. |
| `code.txt` | `.handoff/code.txt` | `tokmd handoff` | Token-budgeted source bundle selected for the agent. | It is not the whole repository unless the budget and policy allow it. | Check `.handoff/manifest.json` for included and excluded files. |
| `map.jsonl` | `.handoff/map.jsonl` | `tokmd handoff` | Full file inventory sidecar for path lookup and downstream tooling. | It is not the selected source bundle. | Use when the agent needs to locate paths beyond `code.txt`. |
| `intelligence.json` | `.handoff/intelligence.json` | `tokmd handoff` | Repository shape, hotspot, complexity, and derived signals for the bundle. | It is a warning label, not a proof result. | Use alongside `work-order.md` and review/proof receipts. |
| `review-links.json` | `.handoff/review-links.json` | `tokmd handoff --review-packet-dir/--review-packet-check` | Packet-local pointers to external cockpit review packet artifacts and verifier receipt. | It does not copy or verify the external review packet. | Open linked `review-packet-check.json` before trusting the review packet. |
| `proof-links.json` | `.handoff/proof-links.json` | `tokmd handoff --affected/--proof-plan` | Packet-local pointers to external affected-proof and proof-plan receipts. | It does not run proof or make planned proof pass. | Open linked `affected.json` and `proof-plan.json`; run required proof when needed. |

## AST Shadow Evidence

| Artifact | Usual path | Writer | Means | Does not mean | Verify or inspect |
| --- | --- | --- | --- | --- | --- |
| `heuristic.json` | `target/tokmd-ast-shadow/heuristic.json` | `cargo xtask ast-shadow-compare` | Developer-facing view of the heuristic Rust landmarks selected for AST comparison. | It is not a public `tokmd` receipt or default analysis output. | Pair with `ast.json` and `diff.json`; verify with `cargo xtask ast-shadow-check --dir target/tokmd-ast-shadow`. |
| `ast.json` | `target/tokmd-ast-shadow/ast.json` | `cargo xtask ast-shadow-compare` | Feature-gated AST-backed Rust landmarks for the same explicit file selection. | It does not claim browser/WASM AST support or replace heuristic defaults. | Inspect parser status and parse-degraded files before drawing conclusions. |
| `diff.json` | `target/tokmd-ast-shadow/diff.json` | `cargo xtask ast-shadow-compare` | Deterministic comparison of heuristic and AST landmarks, including matched, heuristic-only, AST-only, parse-degraded, and unsupported counts. | It is not a merge verdict or proof-promotion signal. | Read `summary.md` first if present; verify summary counts with `ast-shadow-check`. |
| `summary.md` | `target/tokmd-ast-shadow/summary.md` | `cargo xtask ast-shadow-compare --summary-md <path>` | Human review layer over `diff.json` with aggregate counts, per-file status, artifact paths, and reproduction command. | It is not machine authority; use JSON artifacts for tooling. | Re-run the command shown in the summary and then run `ast-shadow-check`. |
| `check.json` | `target/tokmd-ast-shadow/check.json` | `cargo xtask ast-shadow-check --json <path>` | Verifier receipt for the AST shadow artifact set: required files, schema/kind, sorted relative paths, timestamp-free content, and matching summary counts. | It does not make AST evidence public product behavior. | Regenerate after any artifact change and keep it with the compared artifact set. |

## Browser Artifacts

| Artifact | Usual path | Writer | Means | Does not mean | Verify or inspect |
| --- | --- | --- | --- | --- | --- |
| Browser-safe receipt download | Browser runner download | `web/runner` with `tokmd-wasm` | No-install summary, export, or browser-safe analysis over GitHub-loaded or uploaded inputs. | It does not include native filesystem, git-history, cockpit, gate, context, or handoff behavior. | Compare against [Browser runner](browser.md) and the [WASM capability matrix](capabilities/wasm.json). |

## Related References

- [Start Here](start-here.md) for job-based entry points.
- [User paths](user-paths.md) for command-to-artifact reading order by job.
- [Review packet contract](review-packet.md) for packet layout and verifier semantics.
- [Handoff bundles](handoff.md) for agent bundle consumption.
- [Proof evidence import contract](cockpit-proof-evidence.md) for cockpit proof inputs.
- [Coverage guidance](ci/coverage.md) for coverage receipts and Codecov boundaries.
- [Proof observation artifacts](ci/proof-observation-artifacts.md) for proof
  observation receipts, collections, readiness receipts, and promotion
  boundaries.
- [Evidencebus integration](evidencebus-integration.md) for the stack boundary.
