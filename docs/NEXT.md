# Next Program

The generated PR drain is complete. `PR_DRAIN.md` is now a historical ledger for the duplicate/stale queue and should only change for PR-drain-specific corrections. Active product and control-plane work moves here.

Factory Droid PR #1541 was declined in its original external-service form. A
later safe Droid migration is now active through the pinned
`EffortlessMetrics/droid-action-safe` wrapper with MiniMax BYOK, same-repo /
trusted-actor guards, disabled raw debug artifacts, and the service policy in
`docs/external-services.md`.

## Current Operating Mode

The Rust-native proof control plane is in routine-observation mode. It now owns
proof policy, affected planning, scoped advisory execution, executor
observation collection, fast proof-run observation collection, and artifact
verification. Do not promote fast proof into the required aggregate or enable
default Codecov upload without a fresh maintainer decision backed by collected
evidence.

The v1.11 browser runtime polish lane is closed on main: cache semantics,
worker and repo-load progress, retry/rate-limit guidance, authenticated fetch
UX, loaded-bundle capability filtering, and local browser file input are all
implemented.

The cockpit review-packet lane is stable for explicit proof imports: review
packets can preserve imported proof artifacts, surface proof evidence in
`evidence.json`, `review-map.md`, and `comment.md`, and verify packet-local
hashes without promoting proof gates. Keep `tokmd cockpit` as the PR-review
evidence surface before adding any separate `tokmd review` command.

Architecture consolidation is paused unless fresh product or proof evidence
shows a real owner-module problem. The performance mini-lane has a measurement
spine and several small fixes on main; future performance work should continue
to cite `cargo xtask perf-smoke` receipts or explicitly say it is
structure-only.

The source-of-truth stack is installed and in routine maintenance: proposals
own exploratory rationale, specs own behavior contracts and proof
requirements, ADRs own durable architecture decisions, plans own sequencing,
`.jules/goals/active.toml` owns small machine-readable active-agent state, and
checked policy TOMLs own machine-enforced rules. The completed doc-artifacts
checker plan is closed out; the current active goal now points at cockpit
review usefulness.

## Next Work Packets

1. Keep collecting routine proof observations under the current advisory shape;
   do not promote gates or Codecov defaults.
2. Fix cockpit review-packet and Action-hosting gaps only when fresh evidence
   shows a product, verifier, or hosted-comment issue.
3. Preserve `tokmd cockpit` as the review evidence implementation surface until
   a separate review orchestrator has a real contract.
4. Continue architecture consolidation in batches, preserving `ci/proof.toml`
   scope granularity as implementation microcrates collapse into SRP modules.
5. Use bounded performance timing receipts before optimizing hot paths.
6. Keep source-of-truth docs, active goal state, and proof-policy routing
   aligned as new lanes start; do not reopen the doc-artifacts checker lane
   unless the spec changes.
7. Draft AST foundation work only after the review packet and consolidation
   direction are stable.

## Directional Rules

- `tokmd-config` is retired. It must remain forbidden by policy, with ownership in `tokmd-settings`, `tokmd-core`, and `tokmd`.
- `.jules` is an allowed knowledge workspace for durable specs, investigations, friction notes, persona learnings, runbooks, ledgers, envelopes, and generated indexes.
- Coverage remains advisory telemetry until maintainers intentionally promote it to a gate.
- Fast proof-run and scoped coverage observations remain advisory until maintainers intentionally promote them.
- Cockpit remains the current PR-review evidence surface until a separate review command has a distinct artifact contract.

## Checkpoints

- Dependency-boundary checks now read `ci/proof.toml` while preserving the existing sorted `tokmd-analysis*` manifest scan and `dependencies` / `dev-dependencies` / `build-dependencies` coverage.
- Fixture-blob checks now read `ci/proof.toml` while preserving the existing crypto extension and marker detection plus the `.claude`, `.jules`, `vendor`, proof-policy source, and checker-source allowlist behavior.
- `cargo xtask affected --base origin/main --head HEAD --json` now maps changed files to proof scopes from `ci/proof.toml`, reports unknown files, and keeps non-Rust unknown handling policy-driven.
- `cargo xtask affected --base origin/main --head HEAD --json-output <path>` now writes the same `tokmd.affected.v1` report as a Rust-owned JSON artifact, so CI and handoff workflows do not need shell redirection to capture `affected.json`.
- `cargo xtask proof --profile affected --base origin/main --head HEAD --plan` now prints a stable proof plan without running commands; `fast`, `release`, and `deep` profiles are plan-only placeholders for the next CI integration slices.
- `cargo xtask proof --plan --plan-json <path>` now writes the same `tokmd.proof_plan.v1` report as a Rust-owned JSON artifact, so CI and handoff workflows do not need to rely on shell redirection to capture `proof-plan.json`.
- CI now validates `cargo xtask proof-policy --check` as part of the required aggregate and uploads PR-only affected proof artifacts while keeping existing jobs authoritative.
- The proof scope registry now covers first-class product/control-plane surfaces for CLI, gate, cockpit, WASM, browser runner, the composite GitHub Action, schema contracts, and the proof control plane.
- Release, mutation, Nix validation, and label-sync workflows now have explicit proof-scope routing so Dependabot or workflow-only action updates do not fail affected planning as unknown files before they reach their relevant release, policy, or documentation proof.
- Clippy policy now has a governed ledger and `cargo xtask check-lint-policy` check while keeping the repository MSRV at 1.92 and leaving crate-wide lint inheritance as a later cleanup stack.
- Analysis and formatting module scopes now route `tokmd-analysis`, `tokmd-analysis-types`, and `tokmd-format` changes to targeted package, snapshot, renderer, and module proof commands.
- Affected proof plans now include advisory scoped coverage and mutation commands derived from matched scope metadata while leaving existing proof commands required and CI behavior unchanged.
- `cargo xtask proof --plan --summary-md <path>` now writes a Markdown proof-plan artifact, and the informational PR affected-plan job appends that Rust-generated summary while keeping existing CI jobs authoritative.
- `cargo xtask proof --plan --evidence-json <path>` now writes a machine-readable planned-evidence artifact for scoped coverage and mutation commands. The artifact records `planned` / `not_executed` status and zero executed counts so consumers do not confuse planned advisory evidence with passing evidence.
- `cargo xtask proof --plan --executor-summary <path>` now writes an informational coverage-only executor summary prototype. It selects non-required coverage commands, records them as skipped with `tool_execution_not_enabled`, and does not invoke `cargo llvm-cov`.
- `cargo xtask proof --plan --executor-summary <path> --executor-mode dry-run` now exercises the executor selection boundary for at most one non-required coverage command without invoking `cargo llvm-cov`.
- Executor summaries now include an `execution_guard` block. CI evidence execution remains disabled unless a future workflow explicitly passes `--allow-ci-evidence-execution`, and current executor modes still report zero executed commands.
- Rust-generated proof-plan Markdown now surfaces executor guard status whenever an executor summary is requested, so the affected-plan CI summary shows whether planner-selected evidence execution is blocked or explicitly opted in.
- `ci/proof.toml` now declares the first executor promotion rule: coverage is the only supported executor family, CI execution requires explicit opt-in, and dry-run selection is policy-limited before any CI job can execute planner-selected evidence commands.
- `cargo xtask proof-policy --check` and `--json` now report the active executor policy rule alongside scope, allowlist, fixture blob, and dependency-boundary counts.
- `cargo xtask proof --plan --executor-manifest <path>` now writes a planner-selected executor command manifest with the executor policy, guard status, selection rule, stable command ids, and zero executed counts.
- The PR-only affected-plan CI artifact job now writes and uploads `executor-manifest.json` alongside `affected.json`, `proof-plan.json`, `proof-evidence.json`, `executor-summary.json`, and `proof-plan.md`, without opting into executor command execution.
- `cargo xtask proof-artifacts-check` now verifies executor summary/manifest consistency without executing planned commands, including schema, guard, count, and command-entry drift checks.
- The PR-only affected-plan CI artifact job now runs `cargo xtask proof-artifacts-check` after artifact generation, records its status, and uploads the verifier output while remaining informational.
- The affected-plan CI job now fails on affected-scope generation, proof-plan generation, or proof artifact verifier failure, while executor command execution remains disabled and existing proof commands remain separately authoritative.
- The proof scope registry now classifies model/scan path-normalization changes and `.jules` provenance updates so generated knowledge artifacts and core path hot-path work do not appear as unknown files in affected plans.
- No-panic policy now has a governed allowlist (`policy/no-panic-allowlist.toml`, schema 0.3) and a semantic checker (`cargo xtask check-no-panic-family`). Identity is `path + family + selector` so reformatting source files does not invalidate receipts. Default mode is advisory: schema/shape, expired entries, and stale entries block; unallowlisted findings are reported only. `cargo xtask no-panic-propose` writes proposed allowlist entries to `target/no-panic-proposed-allowlist.toml`. The strict (blocking) flip is staged behind member-crate `[lints] workspace = true` adoption and a panic-family debt burn-down.
- Workspace dependency graph changes now have an explicit proof scope for root/workspace manifests and `Cargo.lock`, routing affected plans to deny, dependency-boundary, and publish-surface checks instead of leaving lockfile-only changes unknown.
- Fuzz harness changes now have an explicit proof scope for `.github/workflows/fuzz.yml`, `fuzz/Cargo.toml`, targets, dictionaries, corpora, and harness docs, routing affected plans to the fuzz harness inventory check before deeper fuzz execution is promoted.
- The proof executor now has a deliberately opt-in local coverage execution experiment. `--executor-mode execute` cannot be combined with `--plan`, requires explicit local or CI opt-in, runs only planner-selected non-required coverage commands, and writes executor summary/manifest artifacts while required proof jobs remain authoritative.
- `cargo xtask proof-artifacts-check` now allows enabled execution guards for non-executed artifacts; this verifier still rejects executed artifacts by `execution_status` and executed counts until an execution verifier lands.
- `cargo xtask proof-execution-artifacts-check` now verifies opted-in executed executor artifacts separately from the no-execution verifier, requiring executed status, an enabled guard, zero failed commands, and matching summary/manifest command records.
- `.github/workflows/proof-executor.yml` now provides a scoped coverage executor experiment for manual dispatch and non-required PR runs. It runs planner-selected non-required coverage commands with explicit CI opt-in, verifies executed artifacts, uploads proof artifacts, and leaves required PR proof jobs unchanged.
- Manual proof-executor run `25464543145` on `main` passed on 2026-05-06. It verified the workflow-dispatch no-diff path: `affected_status=0`, `executor_status=0`, `verifier_status=0`, `execution_guard.reason=ci_explicit_opt_in_enabled`, and zero selected/executed commands.
- Manual proof-executor run `25465495509` on a disposable branch passed on 2026-05-06. It matched `crates/tokmd-core/tests/ffi_parity_w53.rs` to `tokmd_core_ffi`, selected one advisory coverage command, executed it with `exit_code=0`, produced `target/proof/coverage/tokmd_core_ffi.lcov`, and passed `cargo xtask proof-execution-artifacts-check`.
- The proof executor workflow now runs on pull requests as `Scoped Coverage Executor (Non-Required)`. It remains outside the required CI aggregate, executes only planner-selected non-required coverage commands, keeps Codecov upload manual-only, and leaves existing proof jobs authoritative.
- `tokmd cockpit --review-packet-dir <dir>` now emits the cockpit review packet while leaving default stdout and the existing `--artifacts-dir` director contract unchanged.
- The cockpit review packet now includes `review-map.json` / `review-map.md` generated from the existing cockpit `review_plan`, giving packet consumers a stable prioritized review map without introducing a separate `tokmd review` command.
- The composite GitHub Action now exposes an opt-in cockpit `review-packet` input. In `mode: cockpit`, `review-packet: true` writes `.tokmd/review`, exposes it as the `review-packet` output, keeps `.tokmd/review/comment.md` as the packet-local summary output, and prepares `tokmd-review-packet-comment.md` for optional hosted pull request comments when metadata is added.
- The composite Action self-test now exercises `mode: cockpit`, `review-packet: true`, `artifact: true`, and `comment: false` together, proving packet artifact upload stays independent from pull request commenting.
- `tokmd_core::cockpit_workflow` now has a feature-gated contract test against a real temporary git repo, and the cockpit proof scope routes that facade test through the affected proof plan.
- Cockpit `comment.md` now includes compact evidence availability counts so missing, degraded, stale, skipped, or unavailable evidence is visible in the PR-comment-ready artifact, not only in packet JSON.
- Browser worker protocol v2 now emits run progress messages for in-memory worker execution. Worker runs produce `start`, `fetch`, optional `analyze`, `done`, and `error` progress phases while keeping cancellation explicitly unavailable.
- The browser runner UI now displays worker-run progress in a dedicated run-progress panel, while preserving the latest successful result during later repo-load or worker-run progress updates.
- Browser runner terminal worker messages now follow the same active-request guard as progress messages, so stale `result` or `error` events from an older run cannot overwrite a newer run's UI state.
- Browser runner GitHub token UX now uses session-only storage, shows anonymous/authenticated state without exposing the raw token, and provides an explicit clear-token action.
- Browser runner GitHub token UX now marks rejected tokens after GitHub auth/access failures, surfaces update-or-clear guidance, and keeps raw token text out of logs and status output.
- Browser GitHub ingest now surfaces numeric or HTTP-date `Retry-After` guidance in the UI and enables a manual retry action only for retryable GitHub rate-limit failures.
- Browser worker mode and preset reporting now reads the `tokmd-wasm` capability payload when present and intersects it with actual exported entrypoints, so the UI and runtime validation do not promise modes the loaded bundle cannot execute.
- Browser runner mode controls now consume that loaded worker capability surface: unavailable modes are disabled, unsupported current selections are switched to the first supported mode, and analyze sample args choose a preset advertised by the loaded bundle.
- Browser runner GitHub repo-load args now use that same loaded-bundle analyze preset fallback, so a bundle that advertises only `receipt` does not receive a synthetic `estimate` preset after inputs are loaded.
- Browser runner local file selection now fills the existing ordered in-memory `inputs` payload without GitHub fetch, token state, or cache sharing, preserving the latest successful result while users stage local browser-safe runs.
- The cockpit review packet manifest now carries evidence availability counts and gate-id capability groups linked to `evidence.json#/gates`, so packet consumers can see unavailable or degraded evidence without parsing the full cockpit receipt.
- `cargo xtask proof-execution-artifacts-check` now verifies that executed entries with declared artifact paths point at existing, non-empty files, so a passing executor command cannot claim missing LCOV evidence.
- Executed coverage artifacts now must be LCOV-shaped text with `SF:` and `end_of_record` records before `proof-execution-artifacts-check` accepts them.
- `cargo xtask proof-execution-observation` now turns verified executed summary/manifest pairs into `proof-executor-observation.json`, a compact cross-PR observation artifact for collecting non-required executor runs without promoting them to required gates or default Codecov uploads.
- `cargo xtask proof-execution-observations-summary --observation <path>...` now summarizes downloaded executor observation artifacts by family, scope, and source-run window, giving maintainers a Rust-owned collection view before any required-gate or default Codecov-upload promotion. It also accepts `--observations-dir <dir>` to recursively collect `proof-executor-observation.json` artifacts from downloaded GitHub run directories, and `--source-runs-json <path>` to report expected, observed, missing, and unmatched executor-run artifacts from the saved `gh run list` window.
- The proof executor workflow now uploads `proof-executor-observation-collection.json` alongside the per-run observation by running the Rust-owned observation summary command over `target/proof`.
- `cargo xtask proof --run-required --allow-local-required-execution --proof-run-summary <path>` now provides the first opt-in required proof-command runner. It executes only required commands from the proof plan, excludes advisory coverage/mutation commands, writes `tokmd.proof_run_summary.v1`, and still leaves CI/default invocations in plan-only mode unless a future workflow explicitly opts into required execution.
- `cargo xtask proof-run-artifacts-check --proof-run-summary <path>` now verifies required proof-run summaries before workflow adoption: schema, enabled local/CI guard, zero failed commands, empty unknown files, count consistency, and required-only passed entries.
- CI now runs `Fast Proof Run (Advisory)` on pull requests, which executes `cargo xtask proof --profile fast --run-required --allow-ci-required-execution`, verifies `tokmd.proof_run_summary.v1`, uploads `fast-proof-run` artifacts, and deliberately stays outside the required aggregate while existing jobs remain authoritative.
- `ci/proof.toml` now owns the advisory PR fast proof-run defaults under `[proof_run.pr]`: `profile = "fast"`, `default_enabled = true`, `required = false`, and `artifact_name = "fast-proof-run"`. CI resolves those checked defaults before running the advisory job.
- `cargo xtask proof-run-observation --proof-run-summary <path> --output <path>` now writes `tokmd.proof_run_observation.v1`, a compact observation artifact derived from a verified required proof-run summary. The advisory Fast Proof Run CI job uploads it alongside `proof-run-summary.json`.
- `cargo xtask proof-run-observations-summary --observation <path>...` now summarizes downloaded fast proof-run observation artifacts by profile, scope, guard reason, and source-run window. This gives maintainers Rust-owned visibility into routine PR fast-proof observations without promoting the advisory job into the required aggregate.
- Fast proof-run observation collection over PR CI run `25522285718` passed on 2026-05-07 after #1756 merged. The collection recorded 1 observed run, 0 missing runs, 4 required/executed/passed commands, 0 failed commands, and 4 scopes: `boundaries`, `fixture_blobs`, `proof_policy`, and `workspace`. The guard reason was `ci_explicit_required_opt_in_enabled`, and the advisory job remains outside the required aggregate.
- Observation collection can now enforce readiness thresholds with `--min-observations`, `--min-executed`, `--min-scopes`, and `--min-artifacts`, so downloaded non-required executor runs can prove a multi-run/multi-scope evidence floor before any promotion decision.
- Observation collection can now also write `--summary-md`, and the proof executor workflow appends that Rust-generated Markdown collection report to the GitHub job summary while still uploading the JSON artifact.
- `cargo xtask proof` now accepts `--executor-max-commands <n>` as a positive override for the policy-selected advisory executor command limit. The proof executor workflow keeps PR runs at a small policy-backed command limit by default, while manual dispatches can raise `max_evidence_commands` to collect multi-scope evidence without changing required gates.
- `.github/workflows/proof-observation-collection.yml` now provides a manual collector for successful `proof-executor.yml` runs. It saves the successful-run list as `target/proof-observations/runs.json`, downloads prior `proof-executor-artifacts`, runs the Rust-owned observation collection thresholds over the downloaded artifacts and source-run window, uploads the collection, and appends the Markdown collection summary without executing new evidence commands. The default collector floor requires an observation artifact but not executed commands/scopes/artifacts yet, so maintainers can record the current evidence floor before choosing stricter promotion thresholds.
- Manual proof-observation collector run `25487797962` on `main` passed on 2026-05-07. It downloaded 11 successful proof-executor observations and recorded the current floor: 11 observations, zero selected/executed/passed commands, zero scopes, and zero artifacts. That proves the collector path while confirming that stricter promotion thresholds need intentionally collected coverage-enabled observations first.
- Manual proof-executor run `25489053208` on disposable branch `codex/proof-executor-coverage-sample` passed on 2026-05-07. It changed `crates/tokmd-core/tests/ffi_parity_w53.rs` and `crates/tokmd-format/src/redact/mod.rs`, selected two non-required coverage commands, executed/passed both, and produced two LCOV artifacts for `tokmd_core_ffi` and `format_redaction_scan_args`.
- Manual proof-observation collector run `25489377912` on `main` passed on 2026-05-07 with stricter thresholds: `--min-observations 1`, `--min-executed 2`, `--min-scopes 2`, and `--min-artifacts 2`. The collection recorded 19 observations total, with 2 selected/executed/passed coverage commands, 2 covered scopes, and 2 artifacts.
- `cargo xtask proof-execution-artifacts-check` now resolves downloaded executor artifacts without manually reconstructing `target/proof`: it still honors workflow-relative paths as written, and also resolves `target/proof/...` artifact paths against the downloaded artifact root. The downloaded artifacts from run `25489053208` now re-verify locally with 2 executed commands and guard `ci_explicit_opt_in_enabled`.
- Advisory Codecov coverage now has README visibility, documented lane boundaries in `docs/ci/coverage.md`, proof-control-plane routing for `.github/workflows/coverage.yml`, `codecov.yml`, and the coverage docs, plus a narrow `project_readme` scope so README-only edits route to docs checks instead of unknown-file failures. Coverage remains telemetry, not a ratchet or required gate.
- `cargo xtask coverage-receipt` now emits `tokmd.coverage_receipt.v1` for `coverage.json`, `coverage.txt`, and `lcov.info`, and the coverage workflow uploads `target/coverage/coverage-receipt.json` with the coverage artifacts. The receipt records coverage artifact presence and byte counts without making coverage a required gate.
- Codecov project and patch statuses remain informational during the baseline phase, and coverage receipt generation is owned by `cargo xtask coverage-receipt` rather than a duplicate workflow heredoc.
- `cargo xtask ci-actuals` now emits `tokmd.ci_actuals.v1` from a GitHub Actions `needs` payload plus optional timing sidecar data. Missing timing is recorded as missing rather than zero so later budget and learned-estimate work can consume the receipt without inventing measurements.
- Top-level project truth docs (`ROADMAP.md`, ADRs, architecture, design, implementation plan, requirements, and specification) now have a proof-policy scope that routes changes to `cargo xtask docs --check` instead of leaving architecture-doc-only fixes as unknown files.
- The external-service policy now reflects the active safe Droid integration:
  Factory Droid is approved only through the pinned safe action wrapper,
  requires `FACTORY_API_KEY` and `MINIMAX_API_KEY`, skips fork PR auto-review,
  restricts manual `@droid` commands to trusted actors, and keeps raw debug
  artifact upload disabled.
- Manual proof-executor run `25499361375` on disposable branch `codex/proof-executor-more-scope-sample` passed on 2026-05-07. It changed `crates/tokmd-analysis/src/complexity/mod.rs` and `crates/tokmd-gate/src/pointer.rs`, matched `analysis_complexity` and `tokmd_gate`, selected and executed two non-required coverage commands, produced `analysis_complexity.lcov` and `tokmd_gate.lcov`, and re-verified locally with `cargo xtask proof-execution-artifacts-check`.
- Manual proof-observation collector run `25499876871` on `main` intentionally failed the stricter four-scope floor under a last-30 successful-run window: the collector found enough executed commands and artifacts, but only 3 distinct scopes. That is useful negative evidence that promotion thresholds need a recency/window rule, not just aggregate historical totals.
- Manual proof-observation collector run `25499968322` on `main` passed on 2026-05-07 with `--run-limit 100`, `--min-observations 1`, `--min-executed 4`, `--min-scopes 4`, and `--min-artifacts 4`. The collection recorded 54 observations, 8 selected/executed/passed coverage commands, 8 artifacts, and 5 distinct scopes: `analysis_complexity`, `format_redaction_scan_args`, `tokmd_cli`, `tokmd_core_ffi`, and `tokmd_gate`.
- `ci/proof.toml` now records checked scoped coverage executor promotion criteria under `[executor.promotion]`: `window = "last_successful_runs"`, `run_limit = 100`, `min_observations = 1`, `min_executed = 4`, `min_scopes = 4`, `min_artifacts = 4`, `min_passing_collector_runs = 1`, `required_gate = false`, and `default_codecov_upload = false`. `cargo xtask proof-policy --check` validates the window/thresholds and rejects required-gate or default Codecov-upload promotion until those behaviors are implemented intentionally.
- `.github/workflows/proof-observation-collection.yml` now resolves blank manual collector thresholds from checked `[executor.promotion]` policy via `cargo xtask proof-policy --json`, writes the resolved `thresholds.env` into the uploaded artifact, and shows whether each threshold came from `ci/proof.toml` or a workflow-dispatch override. The executor remains non-required, and Codecov upload remains manual-only.
- Manual proof-observation collector run `25502593070` on `main` passed on 2026-05-07 using blank workflow-dispatch inputs after #1727 merged. The workflow resolved `run_limit = 100`, `min_observations = 1`, `min_executed = 4`, `min_scopes = 4`, and `min_artifacts = 4` from `ci/proof.toml`; the collection recorded 58 observations, 8 selected/executed/passed coverage commands, 8 artifacts, and 5 distinct scopes: `analysis_complexity`, `format_redaction_scan_args`, `tokmd_cli`, `tokmd_core_ffi`, and `tokmd_gate`.
- `[executor.promotion]` now makes the recency window explicit with `window = "last_successful_runs"`; `cargo xtask proof-policy --json` reports it and validation rejects promotion blocks without a window. This preserves the current last-100 collector floor while keeping the earlier last-30 failure visible as evidence that window size is a policy decision, not an implicit workflow default.
- `[executor.promotion]` now records `min_passing_collector_runs = 1` as a promotion precondition, so a future required-gate or default Codecov-upload flip must cite at least one recent passing manual collector run in addition to the executor observation thresholds.
- The manual observation collector now emits `proof-executor-promotion-readiness.json` with schema `tokmd.proof_executor_promotion_readiness.v1`. The receipt verifies the policy-backed observation thresholds plus `min_passing_collector_runs` from recent successful `proof-observation-collection.yml` GitHub run history.
- Manual proof-observation collector run `25505861187` on `main` passed on 2026-05-07 using blank workflow-dispatch inputs after #1731 merged. The workflow resolved `run_limit = 100`, `min_observations = 1`, `min_executed = 4`, `min_scopes = 4`, `min_artifacts = 4`, and `min_passing_collector_runs = 1` from `ci/proof.toml`; the collection recorded 62 observations, 8 selected/executed/passed coverage commands, 8 artifacts, and 5 distinct scopes: `analysis_complexity`, `format_redaction_scan_args`, `tokmd_cli`, `tokmd_core_ffi`, and `tokmd_gate`. The new `proof-executor-promotion-readiness.json` receipt reported schema `tokmd.proof_executor_promotion_readiness.v1`, `ok = true`, and 1 recent passing collector run from `25502593070`.
- `[executor.pr]` now records the default-on PR executor decision directly in `ci/proof.toml`: PR observation is enabled, remains non-required, uses a small planner-selected command limit by default, and keeps Codecov upload off unless a manual workflow dispatch explicitly opts in. `.github/workflows/proof-executor.yml` resolves the command limit from `cargo xtask proof-policy --json` so the PR default is Rust-policy-backed rather than YAML-only.
- PR proof-executor run `25506897630` on #1734 passed on 2026-05-07 with policy-backed PR defaults. The uploaded `proof-policy.json` recorded `[executor.pr] default_enabled = true`, `required = false`, `max_commands = 1`, and `codecov_upload = false`; all executor statuses were zero, the execution guard reason was `ci_explicit_opt_in_enabled`, and the Codecov upload step was skipped because the event was `pull_request`. The run selected zero coverage commands because #1734 touched the proof-control-plane scope, which intentionally has `coverage = false`.
- Disposable draft PR #1736 collected the first policy-backed PR executor run on a coverage-enabled scope and was closed unmerged to avoid product-scope churn. PR proof-executor run `25508000337` passed on 2026-05-07: `affected.json` mapped `crates/tokmd-core/tests/ffi_parity_w53.rs` to `tokmd_core_ffi`, the uploaded `proof-policy.json` recorded `[executor.pr] default_enabled = true`, `required = false`, `max_commands = 1`, and `codecov_upload = false`, `executor-summary.json` selected/executed/passed one non-required coverage command, `coverage/tokmd_core_ffi.lcov` was uploaded, and local verification accepted the downloaded artifacts plus a one-observation summary with 1 executed command, 1 scope, and 1 artifact. Codecov upload remained skipped because the event was `pull_request`.
- `[executor.pr]` now widens the non-required PR executor observation default to `max_commands = 2`. This keeps PR execution advisory and Codecov-off while beginning broader scoped coverage observation when a PR touches multiple coverage-enabled proof scopes.
- Disposable draft PR #1740 proved the two-command PR default and was closed unmerged to avoid product-scope churn. PR proof-executor run `25509510820` passed on 2026-05-07: `proof-policy.json` recorded `[executor.pr] default_enabled = true`, `required = false`, `max_commands = 2`, and `codecov_upload = false`; `executor-summary.json` selected/executed/passed two non-required coverage commands for `format_redaction_scan_args` and `tokmd_core_ffi`; the run uploaded `format_redaction_scan_args.lcov` and `tokmd_core_ffi.lcov`; and local verification accepted the downloaded artifacts with `cargo xtask proof-execution-artifacts-check`.
- Manual proof-observation collector run `25510251689` on `main` passed on 2026-05-07 after #1741 merged. The workflow resolved `run_limit = 100`, `min_observations = 1`, `min_executed = 4`, `min_scopes = 4`, `min_artifacts = 4`, and `min_passing_collector_runs = 1` from `ci/proof.toml`; the collection recorded 72 observations, 13 selected/executed/passed coverage commands, 13 artifacts, and 5 distinct scopes: `analysis_complexity`, `format_redaction_scan_args`, `tokmd_cli`, `tokmd_core_ffi`, and `tokmd_gate`. The `proof-executor-promotion-readiness.json` receipt reported schema `tokmd.proof_executor_promotion_readiness.v1`, `ok = true`, and 1 recent passing collector run from `25505861187`.
- Manual proof-observation collector run `25512575044` on `main` passed on 2026-05-07 after #1743 and #1733 merged. The workflow resolved `run_limit = 100`, `min_observations = 1`, `min_executed = 4`, `min_scopes = 4`, `min_artifacts = 4`, and `min_passing_collector_runs = 1` from `ci/proof.toml`; the collection recorded 75 observations, 15 selected/executed/passed coverage commands, 15 artifacts, and 6 distinct scopes: `analysis_complexity`, `analysis_content_assets`, `format_redaction_scan_args`, `tokmd_cli`, `tokmd_core_ffi`, and `tokmd_gate`. The `proof-executor-promotion-readiness.json` receipt reported schema `tokmd.proof_executor_promotion_readiness.v1`, `ok = true`, and 1 recent passing collector run from `25510251689`.
- PR proof-executor run `25514476862` on #1748 passed on 2026-05-07 under the two-command PR default. It matched `crates/tokmd/tests/cockpit_integration.rs` to `tokmd_cockpit`, selected/executed/passed one non-required coverage command, produced `tokmd_cockpit.lcov`, and locally re-verified with `cargo xtask proof-execution-artifacts-check`.
- Manual proof-observation collector run `25515026895` on `main` passed on 2026-05-07 after #1748 merged. The workflow resolved `run_limit = 100`, `min_observations = 1`, `min_executed = 4`, `min_scopes = 4`, `min_artifacts = 4`, and `min_passing_collector_runs = 1` from `ci/proof.toml`; the collection recorded 80 observations, 16 selected/executed/passed coverage commands, 16 artifacts, and 7 distinct scopes: `analysis_complexity`, `analysis_content_assets`, `format_redaction_scan_args`, `tokmd_cli`, `tokmd_cockpit`, `tokmd_core_ffi`, and `tokmd_gate`. The `proof-executor-promotion-readiness.json` receipt reported schema `tokmd.proof_executor_promotion_readiness.v1`, `ok = true`, and 1 recent passing collector run from `25512575044`.
- Manual proof-observation collector run `25516861742` on `main` passed on 2026-05-07 after #1750 merged. The collection recorded 82 observations, 16 selected/executed/passed coverage commands, 16 artifacts, and 7 distinct scopes: `analysis_complexity`, `analysis_content_assets`, `format_redaction_scan_args`, `tokmd_cli`, `tokmd_cockpit`, `tokmd_core_ffi`, and `tokmd_gate`. The new source-run window accounting reported `expected_runs = 99`, `observed_runs = 82`, `missing_runs = 17`, and `unmatched_observations = 0`, proving the collector can distinguish successful executor runs that lacked downloadable observation artifacts from observations outside the saved run window.
- Proof-control-plane status: routine PR observations continue under the two-command default. There is no active promotion slice for a required gate, default Codecov upload, or larger command-limit default.
- `cargo xtask perf-smoke --target-repo <path> --output target/perf/perf-smoke.json` now emits `tokmd.perf_smoke.v1`, an opt-in measurement receipt for core `lang`, `module`, and `export` workflows. Receipt schema version 2 also supports repeatable `--analysis-preset <name>` timings with bounded file, byte, and git limits so content/near-dup analysis lanes can collect measurement evidence before optimization work. The receipt records row and language counts while redacting raw target paths.
- The documentation source-of-truth lane now has a draft doc-artifact contract in `docs/specs/doc-artifacts.md` and an active implementation plan in `docs/plans/doc-artifacts-check.md` for `cargo xtask doc-artifacts --check`. The checker verifies artifact shape, links, and routing without changing product behavior, proof promotion, Codecov defaults, or product receipt schemas.
- `policy/doc-artifacts.toml` now captures the draft source-of-truth artifact families, allowed statuses, required sections, active-goal schema, link rules, and optional `tokmd.doc_artifacts_check.v1` receipt schema that `cargo xtask doc-artifacts --check` consumes. The policy remains documentation-control-plane configuration; it does not promote proof gates or Codecov defaults.
- `cargo xtask doc-artifacts --check` now validates the source-of-truth artifact policy, required docs, active-goal links, doc family status values, ADR filenames, and required sections. `cargo xtask docs --check` now calls the checker after generated reference documentation is verified, keeping source-of-truth validation in the docs lane without changing product behavior.
- `cargo xtask doc-artifacts --check --json <path>` now writes a visibility-only `tokmd.doc_artifacts_check.v1` receipt with checked counts and errors for CI artifacts, review packets, or later evidencebus consumers. Text output remains the default.
- CI Docs Check now uploads the visibility-only `doc-artifacts-check` artifact from `target/docs/doc-artifacts-check.json`, so source-of-truth validation has a reviewable receipt without changing proof promotion, Codecov defaults, or product behavior.
- `ci/proof.toml` now maps source-of-truth docs, templates, `.jules/goals/**`, `policy/doc-artifacts.toml`, and doc-artifacts checker code into the `doc_artifacts_policy` scope, so affected proof selects the doc-artifacts checker whenever the source-of-truth stack changes.
- `docs/agent-workflows/source-of-truth.md` now gives maintainers and coding agents an operational checklist for reading `docs/NEXT.md`, `.jules/goals/active.toml`, linked plans/specs/ADRs, policy files, and proof output before changing a lane.
- `.jules/goals/archive/README.md` now defines how completed, paused, or superseded active goals can be archived as historical snapshots without turning archives into a second active queue.
- `tokmd cockpit --doc-artifacts-check <path> --review-packet-dir <dir>` now imports the visibility-only `tokmd.doc_artifacts_check.v1` receipt into review packets, copies it to `docs/doc-artifacts-check.json`, records it in `manifest.json` and `evidence.json`, and summarizes it in `review-map.md` / `comment.md` without changing merge behavior, proof promotion, or Codecov defaults.
- The documentation source-of-truth lane is closed through first enforcement:
  `docs/plans/doc-artifacts-check.md` is complete, the completed goal is
  archived in `.jules/goals/archive/2026-05-13-doc-artifacts-check.toml`, and
  `.jules/goals/active.toml` now points at cockpit review usefulness.
- The cockpit review packet comment now points directly to `evidence.json`, `review-map.md`, and `cockpit.json`, so hosted PR comments have a short path from the summary to the full packet artifacts.
- Cockpit review-packet evidence availability now uses the `missing` bucket for pending gates with relevant scope but no tested scope, keeping absent optional gates separate as `unavailable`.
- The composite Action now adds hosted packet metadata to review-packet PR comments, pointing reviewers to the workflow run, `tokmd-receipts` artifact, and `.tokmd/review` packet path when artifacts are uploaded.
- The composite Action now prepares hosted review-packet comments in `tokmd-review-packet-comment.md` instead of mutating `.tokmd/review/comment.md`, preserving `manifest.json` hashes for packet-local artifacts while keeping hosted PR comments useful.
- The composite Action self-test now runs `cargo xtask review-packet-check --dir .tokmd/review` after preparing the hosted comment copy, proving Action-hosted metadata does not drift packet-local manifest hashes.
- `cargo xtask review-packet-check --dir <dir> --json <path>` now emits a
  `tokmd.review_packet_check.v1` verifier receipt with schema, artifact, hash,
  and packet-local path evidence for CI upload or downstream inspection.
- The composite Action now writes `target/tokmd/review-packet-check.json` for
  cockpit review packets after preparing the hosted comment copy and uploads it
  with `tokmd-receipts` when artifact upload is enabled.
- Hosted review-packet comments now show verifier status, manifest hash status,
  and compact proof evidence counts while keeping packet-local `comment.md`
  immutable after manifest hashing.
- `docs/cockpit-proof-evidence.md` now includes the maintainer-facing local
  workflow for planning proof, optionally executing guarded required proof,
  importing proof artifacts, and verifying the review packet.
- `docs/evidencebus-integration.md` now maps verified tokmd review packets to
  evidencebus producer inputs, preserving the boundary that tokmd emits code
  evidence while evidencebus validates, inventories, bundles, and exports
  cross-tool evidence.
- Cockpit `review-map.json` and `review-map.md` now surface packet-level evidence counts and item-level evidence status, so maintainers can see what proof is present or missing while deciding what to review first.
- Cockpit review packets now keep imported proof artifacts packet-local under
  `.tokmd/review/proof/*.json`, list those artifacts in `manifest.json`, and
  link direct changed-file matches from `review-map.json` items to
  `evidence.json#/proof/*` plus the copied source proof artifact. Review-map
  Markdown now renders direct changed-file proof matches with required/advisory,
  execution, availability, freshness, command, and proof-reference lines.
  Packet-local `comment.md` now includes compact proof evidence totals for
  required/advisory proof and freshness without listing raw command output.
- Cockpit `review-map.md` now also renders a packet-level imported-proof
  overview, so coverage receipts or other proof artifacts that apply at packet
  scope remain visible to reviewers even when they do not directly match a
  changed review-map item.
- Architecture consolidation now has a current-state batch plan in
  `docs/architecture-consolidation-plan.md`, grounded in the live
  publish-surface verifier, large-file inventory, and `ci/proof.toml` scopes.
- The architecture-consolidation plan now reflects the live context-packing and
  derived-analysis owner-module state: context selection/render/manifest/output
  owners are already split, and derived analysis has coherent distribution,
  integrity, ratio, file-metrics, and language-composition owners.
- `tokmd-types` now has a dedicated context/handoff DTO owner module while
  keeping root-level public re-exports, schema versions, and serde receipt
  behavior stable. The `tokmd_context_handoff` and `schema_contracts` proof
  scopes both route the new owner file to targeted context/handoff and schema
  checks.
- Cockpit's Rust complexity gate now delegates function-scoped source analysis
  to `tokmd-analysis::source_complexity` instead of owning a duplicate parser.
  The `else if` double-count is fixed as a correctness improvement; impact
  analysis over 183 current relevant Rust source files found 52 files near the
  10-20 complexity range and 0 files flipping from fail to pass at threshold 15.
- Agent-facing architecture docs now reflect the current crate-and-module
  ownership model: analysis rendering lives in `tokmd-format`, review evidence
  in `tokmd-cockpit`, and implementation details should stay as SRP owner
  modules unless they are durable public surfaces.
- Product truth docs now distinguish tokmd's role from the wider Effortless
  Metrics evidence stack: tokmd is the deterministic code-intelligence and
  review-receipt producer, while evidencebus remains the schema-first evidence
  backplane for cross-tool validation, inventory, bundling, and export.
- Cockpit proof-evidence import now has a docs-only contract in
  `docs/cockpit-proof-evidence.md`, defining accepted proof artifacts,
  required/advisory classification, commit freshness, rendering expectations,
  and explicit non-goals before any CLI/API import surface is added.
- Roadmap-era implementation notes now distinguish historical microcrate
  extraction from the current owner-module consolidation shape, so future
  architecture work starts from the actual crate graph instead of retired
  package names.
- ADR-0008 now defines the AST foundation as feature-gated, Rust-first, and
  shadow-mode-only until real comparison evidence justifies schema or default
  metric changes.
- The first `tokmd-analysis` AST scaffold now exists behind the `ast` feature
  with shadow-only capability metadata, stable shadow artifact names, and a
  dedicated proof-policy scope. It adds no parser dependency and changes no
  default receipts.
- The Rust AST shadow scaffold now uses optional `tree-sitter` /
  `tree-sitter-rust` dependencies behind the `ast` feature to parse
  deterministic Rust function landmarks. It still emits no default receipt
  changes and does not expose browser/WASM AST capability.

## References

- Historical drain ledger: [PR_DRAIN.md](../PR_DRAIN.md)
- External service policy: [external-services.md](external-services.md)
- Review packet contract: [review-packet.md](review-packet.md)
- Current testing strategy: [testing.md](testing.md)
