# Architecture Consolidation Plan

Status: active planning baseline for owner-module consolidation.

This plan turns the current architecture direction into small implementation
batches. It starts from the present workspace and proof-policy state rather
than from historical microcrate names.

## Goal

Keep `tokmd`'s public package surface stable while continuing to split large
implementation files into single-responsibility owner modules.

The intended result is:

- public crates remain contracts, facades, workflows, capability surfaces, or
  products;
- implementation seams live as modules under the owning crate;
- proof scopes in [`ci/proof.toml`](../ci/proof.toml) preserve targeted test,
  coverage, and mutation routing as files move;
- crates.io publish closure stays verified by
  `cargo xtask publish-surface --json --verify-publish`.

## Current Surface

The current publish-surface verifier reports:

| Class | Crates |
| --- | --- |
| Public product | `tokmd`, `tokmd-core`, `tokmd-wasm` |
| Public contract | `tokmd-analysis-types`, `tokmd-envelope`, `tokmd-io-port`, `tokmd-settings`, `tokmd-types` |
| Public workflow | `tokmd-cockpit`, `tokmd-gate`, `tokmd-sensor` |
| Public capability | `tokmd-analysis`, `tokmd-format`, `tokmd-git`, `tokmd-model`, `tokmd-scan` |
| Non-crates.io workspace packages | `tokmd-fuzz`, `tokmd-node`, `tokmd-python`, `xtask` |

There are no current target-gap support crates, internal module-family
packages, dev-only workspace packages, or publish-surface violations.

## Consolidation Rules

1. Do not add new implementation microcrates.
2. Prefer `pub(crate)` module boundaries unless a public API is already part of
   the crate contract.
3. Preserve existing JSON schemas and receipt semantics unless a separate
   schema-change PR documents and validates the contract change.
4. Keep moves mechanical first: split modules, preserve exports, run targeted
   proof, then simplify visibility in a follow-up when needed.
5. Update `ci/proof.toml` in the same PR when a moved file would otherwise
   weaken affected-scope routing.
6. Run `cargo xtask publish-surface --json --verify-publish` for any change
   that moves dependencies, manifests, public exports, or crate ownership.
7. Keep proof-control-plane lanes advisory unless a maintainer explicitly
   approves a promotion decision.

## Current Pressure Points

The first consolidation candidates are large production files, not test
fixtures:

| Area | Current file | Approx. lines | Owner direction |
| --- | --- | ---: | --- |
| Content complexity | `crates/tokmd-analysis/src/content/complexity.rs` and `crates/tokmd-analysis/src/content/complexity/` | `tests/unit.rs` 1451; production owner modules <=187 | Scoring, nesting, and function-span helpers now live under owner modules; remaining work is mostly test split and aggregation cleanup |
| Content I/O helpers | `crates/tokmd-analysis/src/content/io.rs` and `crates/tokmd-analysis/src/content/io/` | coordinator 351; tag owner/tests 71 | Keep file reads, text detection, hashing, and entropy helpers in `io.rs` for now; keep tag counting and delimited matching in `io/tags.rs`. Preserve `crate::content::io::*` call sites and the `analysis_content_assets` proof scope. |
| Analysis orchestration | `crates/tokmd-analysis/src/analysis/mod.rs` and `crates/tokmd-analysis/src/analysis/` | coordinator 152; setup 20; files 53; outputs 22; enricher owners <=160 | Derived/source setup, file collection, optional output storage, and inventory/content/git/semantic/code-quality/effort enrichers now live in owner modules while `analyze` remains the public coordinator. Keep `analysis_orchestration` proof scoped to both the historical `analysis.rs` path and the new `analysis/**` owner paths during the file-to-folder transition |
| Analysis git report | `crates/tokmd-analysis/src/git/mod.rs`, `git/freshness.rs`, and `git/churn.rs` | coordinator 221; freshness/code-age owner 192; predictive churn owner/tests 174 | Keep export-row matching, hotspot, bus-factor, coupling, and intent assembly in `mod.rs` for now; keep freshness, age buckets, and refresh trend in `freshness.rs`; keep predictive churn in `churn.rs`. Preserve the `analysis_git` proof scope. |
| Analysis archetype inference | `crates/tokmd-analysis/src/archetype/mod.rs` and `crates/tokmd-analysis/src/archetype/rules.rs` | coordinator 74; rule owner/tests 403 | Keep export-row filtering and path normalization in `mod.rs`; keep ordered Rust/Next.js/container/IaC/Python/Node archetype rules and rule tests in `rules.rs`. Preserve `analysis_security_identity` proof coverage. |
| Analysis preset grid | `crates/tokmd-analysis/src/grid/mod.rs`, `grid/presets.rs`, and `grid/disabled_feature.rs` | `mod.rs` 10; preset matrix owner 445; disabled-feature warning owner 66 | Keep `mod.rs` as the public barrel; keep preset identity, preset-to-enricher matrix, and plan lookup in `presets.rs`; keep disabled-feature warning text in `disabled_feature.rs`. Preserve the `analysis_orchestration` proof scope. |
| Analysis API surface | `crates/tokmd-analysis/src/api_surface/mod.rs` and `crates/tokmd-analysis/src/api_surface/` | `mod.rs` 13; report owner 227; symbol dispatcher 56; language scanners <=68; symbol tests 385 | Keep `mod.rs` as a thin coordinator, report aggregation in `report.rs`, source scanning in language owner modules under `symbols`, and leave large integration tests under `api_surface/tests` |
| Analysis effort implementation | `crates/tokmd-analysis/src/effort/size_basis.rs`, `crates/tokmd-analysis/src/effort/classify.rs`, and `crates/tokmd-analysis/src/effort/classify/` | size-basis aggregation 271; file classification coordinator 80; `.gitattributes` owner/tests 145; heuristic owner/tests 215 | Authored/generated/vendored line aggregation stays in `size_basis.rs`; `classify.rs` coordinates explicit rules and fallback classification; `.gitattributes` parsing and path-pattern matching live in `classify/gitattributes.rs`; generated sentinels and heuristic file tagging live in `classify/heuristics.rs`. Keep this under the `analysis_effort` proof scope. |
| Context packing and handoff | `crates/tokmd/src/context_pack.rs`, `crates/tokmd/src/context_pack/`, `crates/tokmd/src/commands/context.rs`, and `crates/tokmd/src/commands/handoff.rs` | `context_pack.rs` 15; selection coordinator 305 + policy owner/tests 192 + selection tests 1368; ranking/packing owner 165; output/log submodule 224; render submodule 204; manifest submodule 195; budget parser/tests 220; context command 218; handoff command 268; handoff capability owner 113; handoff intelligence coordinator 196 + complexity owner 434; handoff output owner/tests 242 | Budget parsing, file selection, inclusion-policy preparation, selection proof tests, ranking/packing, bundle text rendering, single-output/log writing, context bundle manifest writing, handoff capability detection, handoff intelligence construction, lightweight source-complexity estimation, and handoff artifact output writing now live in owner modules; keep context/handoff proof scoped while future handoff splits depend on fresh evidence |
| Format core outputs | `crates/tokmd-format/src/diff.rs`, `crates/tokmd-format/src/diff/`, `crates/tokmd-format/src/export.rs`, `crates/tokmd-format/src/export/`, `crates/tokmd-format/src/summary.rs`, and `crates/tokmd-format/src/summary/` | `diff.rs` 313; diff compute owner 168; diff render owner 274; export 101; export redaction owner 179; export CSV owner 47; export JSON owner 69; export JSONL owner 120; export CycloneDX owner 136; summary 424; summary JSON owner 117; language summary owner 86; module summary owner 53 | Diff row/total computation, diff Markdown rendering, export row redaction, CSV/JSON/JSONL/CycloneDX export rendering, language/module summary table rendering, and summary JSON receipt/file writing now live in dedicated owner modules while public helpers and output behavior stay unchanged; future summary coordinator reductions should be snapshot-backed and preserve the root `tokmd-format` re-exports |
| Format fun outputs | `crates/tokmd-format/src/fun/mod.rs` and `crates/tokmd-format/src/fun/` | `mod.rs` 18; OBJ owner 136; MIDI owner 199 | OBJ code-city rendering and MIDI rendering now live in format-specific owner modules while the root fun module preserves public re-exports and feature-gated behavior |
| Format redaction utilities | `crates/tokmd-format/src/redact/mod.rs` and `crates/tokmd-format/src/redact/` | `mod.rs` 251; safe extension owner 44 | Redaction hashing and path redaction remain in the public root module while safe file-extension preservation policy lives in its own private owner module; keep this under the `format_redaction_scan_args` proof scope |
| Core context/handoff DTO contracts | `crates/tokmd-types/src/lib.rs`, `crates/tokmd-types/src/inventory.rs`, `crates/tokmd-types/src/context.rs`, and `crates/tokmd-types/src/diff.rs` | `lib.rs` 453; inventory owner 453; context owner 432; diff owner 85 | Lang/module/export/run inventory receipt DTOs, context receipts, context bundle manifests, handoff manifests, token estimation/audit DTOs, artifact/capability DTOs, and diff receipt DTOs now live in dedicated owners while root re-exports preserve the public contract |
| Cockpit receipt DTO contracts | `crates/tokmd-types/src/cockpit.rs` and `crates/tokmd-types/src/cockpit/` | receipt/metrics/review/trend coordinator 352; evidence gate owner 241 | Cockpit receipt, metric, review-plan, and trend DTOs stay re-exported from the `cockpit` module while evidence gate DTOs live in `cockpit/evidence.rs`. Preserve the `tokmd_cockpit` proof scope and cockpit schema behavior. |
| Envelope contract | `crates/tokmd-envelope/src/lib.rs`, `crates/tokmd-envelope/src/artifact.rs`, `crates/tokmd-envelope/src/capability.rs`, `crates/tokmd-envelope/src/finding.rs`, `crates/tokmd-envelope/src/gate.rs`, `crates/tokmd-envelope/src/ffi.rs`, and `crates/tokmd-envelope/src/findings.rs` | `lib.rs` 395; artifact owner/tests 85; capability owner/tests 96; finding DTO owner/tests 289; gate owner/tests 154; FFI envelope helper 299; finding ID registry 84 | Keep `lib.rs` as the root sensor report contract and public re-export surface; keep artifact DTO builders and tests in `artifact.rs`, capability availability DTO builders and tests in `capability.rs`, portable finding DTO builders and tests in `finding.rs`, gate result/item DTO builders and tests in `gate.rs`, FFI response extraction in `ffi.rs`, and finding ID constants in `findings.rs`. Preserve the `tokmd_envelope_contract` proof scope. |
| Settings contracts | `crates/tokmd-settings/src/lib.rs`, `crates/tokmd-settings/src/commands.rs`, `crates/tokmd-settings/src/config.rs`, `crates/tokmd-settings/src/profile.rs`, and `crates/tokmd-settings/src/scan.rs` | `lib.rs` 36; command settings owner 249; TOML config owner 259; profile owner 32; scan owner 61 | Keep `lib.rs` as root re-export glue; keep workflow command settings in `commands.rs`, TOML configuration contracts in `config.rs`, legacy profile compatibility in `profile.rs`, and shared scan settings in `scan.rs`. Preserve the `tokmd_settings_contract` proof scope. |
| Git adapter boundary | `crates/tokmd-git/src/lib.rs`, `crates/tokmd-git/src/command.rs`, `crates/tokmd-git/src/intent.rs`, and `crates/tokmd-git/src/refs.rs` | git adapter coordinator 251; command/env isolation owner 82; intent classifier owner 165; ref-resolution owner 242 | Keep public history, diff, base-ref, and intent APIs stable from `lib.rs`; keep subprocess environment scrubbing and `git_cmd` construction in `command.rs`; keep Conventional Commit and keyword-based intent classification in `intent.rs`; keep revision existence, base-ref fallback resolution, and env-ref sanitization in `refs.rs`. Preserve `git_subprocess_boundary`, `tokmd_git_intent_classification`, and `tokmd_git_ref_resolution` as git adapter files move. |
| Analysis DTO contracts | `crates/tokmd-analysis-types/src/lib.rs` and owner DTO modules | `lib.rs` 113; baseline owner 37 + complexity-baseline submodule 256 + complexity-section submodule 37 + determinism submodule 22 + metrics submodule 45 + file-entry submodule 23; envelope owner 24; receipt owner 42; topics owner tests 42; entropy owner tests 58; license owner tests 42; churn owner tests 50; complexity owner tests 51 + file submodule 46 + risk submodule 43 + halstead submodule 30 + maintainability submodule 25 + histogram submodule 79 + technical-debt submodule 42; effort owner 25 + estimate submodule 70 + assumptions submodule 35 + cocomo submodule 48 + model submodule 37 + confidence submodule 45 + delta submodule 56 + driver submodule 43 + size submodule 65 + results submodule 45 | Keep root receipt glue and public re-exports stable while moving remaining DTO ownership into modules |
| Derived analysis report | `crates/tokmd-analysis/src/derived/mod.rs` and `crates/tokmd-analysis/src/derived/` | `mod.rs` 220; ratio owner 157; integrity owner 124; file metrics owner 119; language owner 88; distribution owner 80 | Keep `mod.rs` as the receipt assembly coordinator; distribution/histogram calculations, integrity hashing, density/ratio reports, top-file/offender reports, and language composition reports now live in coherent owner modules. Future derived work should start from concrete product or proof evidence rather than splitting for line count. |
| Core facade and FFI | `crates/tokmd-core/src/lib.rs`, `crates/tokmd-core/src/tests.rs`, `crates/tokmd-core/src/mutation_tests.rs`, `crates/tokmd-core/src/receipts.rs`, `crates/tokmd-core/src/workflows/`, and `crates/tokmd-core/src/ffi/` | `lib.rs` 159; root facade test owner 137; mutation test owner 330; receipt helper owner 144; workflow coordinator 31; workflow support owner 99; language workflow owner 84; module workflow owner 94; export workflow owner 106; diff workflow owner 64; analysis workflow owner 169; analysis input owner 118; analysis request owner 178; cockpit workflow owner 90; FFI coordinator 98; FFI test owner 913; envelope owner 15; mode owner 149; input owner 187; parse owner 257; settings parser owner 150 | Language, module, export, diff, analysis, and cockpit workflow facades now live under `workflows/`; core receipt construction lives in `receipts.rs`; root facade tests and mutation-killing receipt/parser tests live in private test owners; shared workflow support helpers live in `workflows/support.rs`; analysis in-memory export preparation lives in `workflows/analyze/input.rs`; analysis request construction lives in `workflows/analyze/request.rs`; public `run_json` coordination, FFI tests, in-memory input decoding/path validation, strict JSON parsing, FFI settings construction, mode dispatch, and response-envelope conversion live under `ffi/`; remaining work is extracting any future FFI helpers without changing public APIs |
| Python binding boundary | `crates/tokmd-python/src/lib.rs`, `crates/tokmd-python/src/runtime.rs`, `crates/tokmd-python/src/args.rs`, `crates/tokmd-python/src/envelope.rs`, and `crates/tokmd-python/src/tests.rs` | `lib.rs` 445; runtime owner 175; argument builder owner 59; envelope helper owner 19; binding proof owner 806 | Keep PyO3 wrapper functions and module registration in `lib.rs`; keep shared Python execution and JSON-module handling in `runtime.rs`; keep Python argument-dict construction in `args.rs`; keep envelope extraction and exception mapping in `envelope.rs`; keep Python binding contract tests in `tests.rs`. Preserve the `tokmd_python_binding` proof scope. |
| Analysis complexity | `crates/tokmd-analysis/src/complexity/mod.rs` + `complexity/functions.rs` + `complexity/functions/` + `complexity/details.rs` + `complexity/details/` + `complexity/summary.rs` + `complexity/risk.rs` + `complexity/debt.rs` + `complexity/histogram.rs` + `complexity/language.rs` + `complexity/math.rs` + `complexity/tests/unit.rs` | 156 + 223 + Rust function owner 126 + 122 + Rust span owner 37 + Python span owner 79 + JS/TS span owner 57 + Go span owner 47 + C-family span owner 52 + 138 + 78 + 69 + 33 + 35 + 5 + 346 | Keep shared complexity logic in `tokmd-analysis`, with Rust function counting plus Rust, Python, JavaScript/TypeScript, Go, and C-family function-span detection in owner modules; split remaining language/source/summary helpers and local unit tests only when fresh evidence shows a clear next seam |
| Analysis Halstead | `crates/tokmd-analysis/src/halstead/mod.rs`, `halstead/operators.rs`, `halstead/operators/sets.rs`, and `halstead/tokenizer.rs` | report coordinator 205; operator dispatch owner 34; operator set owner 310; tokenizer owner 116 | Keep aggregate report construction in `mod.rs`, language/operator lookup in `operators.rs`, static per-language operator sets in `operators/sets.rs`, and per-file token counting in `tokenizer.rs`. Preserve the internal `crate::halstead::*` test surface and keep changes under the `analysis_halstead` proof scope. |
| Analysis maintainability | `crates/tokmd-analysis/src/maintainability/mod.rs`, `maintainability/index.rs`, and `maintainability/tests/` | Halstead attachment coordinator 129; index formula owner 65; behavior/proof tests in dedicated test modules | Keep maintainability index formula, grading, rounding, and formula tests in `index.rs`; keep Halstead metric attachment in `mod.rs`; preserve the internal `crate::maintainability::*` test surface and keep changes under the `analysis_maintainability` proof scope. |
| Analysis near-duplicate detection | `crates/tokmd-analysis/src/near_dup/mod.rs`, `near_dup/clusters.rs`, `near_dup/fingerprint.rs`, and `near_dup/pairs.rs` | report coordinator 274; cluster owner 305; fingerprint owner 95; pair scoring owner 186 | Keep report assembly, eligibility filtering, fingerprint orchestration, and scope partitioning in `mod.rs`; keep cluster construction in `clusters.rs`; keep Winnowing tokenization, k-gram hashing, file reads, and fingerprint selection in `fingerprint.rs`; keep inverted indexing, shared-fingerprint counts, Jaccard scoring, and deterministic pair ordering in `pairs.rs`. Preserve the `analysis_near_dup` proof scope. |
| CLI parser | `crates/tokmd/src/cli/parser.rs` and `crates/tokmd/src/cli/parser/` | `parser.rs` 209; command enum owner 103; global scan args owner 133; shared value-enum owner 235; analyze parser owner 178; context/handoff parser owner 166; cockpit/baseline parser owner 89; diff parser owner 67; gate parser owner 55; sensor parser owner 43; export parser owner 43; badge parser owner 45; module parser owner 36; lang parser owner 26; run parser owner 25; check-ignore parser owner 15; completions parser owner 21; init parser owner 36; tools parser owner 15 | Context, handoff, analyze, cockpit, baseline, diff, gate, sensor, export, badge, module, lang, run, check-ignore, completions, init, tools, command enum, global scan args, and shared value enums now live under parser owner modules; continue command-family splits only when clap snapshots prove behavior is unchanged |
| CLI config resolution | `crates/tokmd/src/config.rs`, `crates/tokmd/src/config/resolve.rs`, and `crates/tokmd/src/config/resolve/` | `config.rs` 313; resolver barrel 10; lang owner 123; module owner 148; export owner 184; parse owner 35 | Keep config discovery, profile selection, `ConfigContext`, and `ResolvedConfig` in the root config module while lang/module/export CLI-to-receipt argument resolution and string-to-enum parsing live in focused resolver owner modules. Preserve root re-exports. |
| Gate command | `crates/tokmd/src/commands/gate.rs` and `crates/tokmd/src/commands/gate/` | `gate.rs` 135; policy owner/tests 215; receipt owner/tests 144; render owner 111 | Gate policy loading, ratchet loading, and config-rule conversion live in the policy owner; receipt loading and compute-then-gate preparation live in the receipt owner; text/JSON result rendering lives in the render owner; the command coordinator keeps flow, evaluation, and result combining. Keep `tokmd_gate` proof scoped to all gate command paths. |
| Gate policy engine | `crates/tokmd-gate/src/evaluate.rs`, `crates/tokmd-gate/src/evaluate/rule.rs`, `crates/tokmd-gate/src/evaluate/compare.rs`, `crates/tokmd-gate/src/ratchet.rs`, `crates/tokmd-gate/src/ratchet/evaluate.rs`, `crates/tokmd-gate/src/ratchet/policy.rs`, `crates/tokmd-gate/src/numeric.rs`, and `crates/tokmd-gate/src/ratchet/change.rs` | `evaluate.rs` 182; policy rule owner/tests 450; compare owner 84; ratchet coordinator/tests 460; ratchet rule evaluator 175; ratchet policy owner 41; numeric owner 37; ratchet change owner 43 | Policy coordination, single-rule evaluation, rule value comparison, and ratchet coordination/tests live in separate workflow/owner modules while single-rule ratchet evaluation, ratchet policy aggregation, shared JSON numeric coercion, and ratchet percentage-change behavior live in private owner modules; strict ratchet evaluation delegates to the configurable evaluator to avoid behavior drift. Keep all gate engine changes under the `tokmd_gate` proof scope. |
| Sensor command | `crates/tokmd/src/commands/sensor.rs` and `crates/tokmd/src/commands/sensor/` | `sensor.rs` 157; findings owner/tests 344; gate mapping owner/tests 189; output owner/tests 141 | Sensor finding emission, cockpit-evidence-to-envelope gate mapping, and 3-layer output topology now live in owner modules while the command keeps git resolution, cockpit execution, and envelope assembly. |
| Model aggregation | `crates/tokmd-model/src/lib.rs`, `crates/tokmd-model/src/aggregate.rs`, `crates/tokmd-model/src/children.rs`, `crates/tokmd-model/src/rows.rs`, and `crates/tokmd-model/src/sorting.rs` | `lib.rs` 255; aggregation owner/tests 344; child-language owner/tests 160; row collection owner/tests 411; sorting owner/tests 91 | Report builders, file-row collection, child/embedded language aggregation, in-memory row detection, and row sorting now live in owner modules; future model work should start from concrete product or proof evidence rather than splitting for line count |
| Scan root handling | `crates/tokmd-scan/src/lib.rs`, `crates/tokmd-scan/src/ignore_patterns.rs`, `crates/tokmd-scan/src/in_memory.rs`, `crates/tokmd-scan/src/roots.rs`, and `crates/tokmd-scan/src/path/` | scan facade 389; ignore-pattern owner/tests 104; in-memory materialization owner/tests 194; root validation/rebasing owner 62; bounded path primitives/tests under `path/` | Keep public scan/config facades in `lib.rs`; keep in-memory file materialization and logical path normalization in `in_memory.rs`; keep tokei ignore-pattern expansion for canonicalized scan roots in `ignore_patterns.rs`; keep validated root construction and caller-facing report-path rebasing in `roots.rs`; keep bounded relative/root path primitives under `path/`. Preserve the `model_scan_path_normalization` proof scope. |
| Scan init templates | `crates/tokmd-scan/src/tokeignore/mod.rs` and `crates/tokmd-scan/src/tokeignore/templates.rs` | init coordinator 131; template catalog owner/tests 261 | Keep CLI-independent `.tokeignore` file creation, force behavior, and print behavior in the coordinator; keep profile template selection and template invariants in `templates.rs`. Preserve the `tokmd_init_tokeignore` proof scope. |
| Scan walk Git boundary | `crates/tokmd-scan/src/walk/mod.rs` and `crates/tokmd-scan/src/walk/git.rs` | walk facade 341; git listing/bounding owner 141 | Keep filesystem and MemFs traversal in `walk/mod.rs`; keep git-backed listing, subprocess environment scrubbing, and tracked-file path bounding in `walk/git.rs`. Preserve the `git_subprocess_boundary` proof scope. |

## Batch Order

### Batch A: Cockpit Owner Modules (Complete)

Why first: cockpit is the active product lane, has strong packet/verifier
coverage, and recent splits already proved the pattern. The cockpit gates and
review-packet rendering surfaces now live under owner modules; the root
`crates/tokmd-cockpit/src/gates.rs` file is a small coordinator.

Current module shape:

```text
crates/tokmd-cockpit/src/
  gates/
    mod.rs
    availability.rs
    freshness.rs
    scope.rs
  render/
    review_packet.rs
    review_map.rs
    comment.rs
```

Required proof:

```bash
cargo test -p tokmd-cockpit --verbose
cargo test -p tokmd --test cockpit_integration --verbose
cargo test -p tokmd-core --features cockpit --test cockpit_workflow --verbose
cargo xtask review-packet-check --dir <generated-review-packet>
```

`ci/proof.toml` scope: `tokmd_cockpit`.

### Batch B: Format Analysis Rendering (Complete)

Why next: `tokmd-format` owns rendering and already has a dedicated
`format_analysis_rendering` proof scope. The production analysis renderer is
now split into format owners: `analysis/mod.rs` is a small dispatcher,
Markdown rendering has section owner modules, and HTML rendering has helper
owner modules for metric cards, table rows, embedded report JSON, and shared
formatting.

Current module shape:

```text
crates/tokmd-format/src/analysis/
  mod.rs
  markdown.rs
  markdown/
  html/
  jsonld.rs
  mermaid.rs
  svg.rs
  tree.rs
  xml.rs
```

Required proof:

```bash
cargo test -p tokmd-format --lib --verbose
cargo test -p tokmd-format --test analysis_format --verbose
cargo test -p tokmd-format --test analysis_html --verbose
cargo xtask docs --check
```

`ci/proof.toml` scope: `format_analysis_rendering`.

### Batch C: Analysis Contracts and Metric Modules

Why third: these files carry more public DTO and metric risk, so they should
follow the lower-risk cockpit/format splits.

Current checkpoint: analysis-type receipt contracts have moved into owner
modules while preserving root re-exports and schema behavior. Derived analysis
is no longer a single broad implementation file: `derived/mod.rs` coordinates
receipt assembly, while `distribution.rs`, `integrity.rs`, `ratios.rs`,
`files.rs`, and `languages.rs` own the coherent subreport builders. Further
work in this batch should be driven by a concrete owner seam, product bug, or
proof gap rather than by file count alone.

Target modules:

```text
crates/tokmd-analysis-types/src/
  lib.rs                  # root re-exports and receipt envelope
  api_surface.rs
  archetype.rs
  args.rs
  assets.rs
  baseline.rs
  churn.rs
  complexity.rs
  corporate.rs
  dependencies.rs
  derived.rs
  duplication.rs
  effort.rs
  entropy.rs
  findings.rs
  fun.rs
  git.rs
  imports.rs
  license.rs
  source.rs
  topics.rs
  util.rs

crates/tokmd-analysis/src/
  complexity/
  content/complexity/
  derived/
  maintainability/
  halstead/
```

Required proof:

```bash
cargo test -p tokmd-analysis-types --verbose
cargo test -p tokmd-analysis --all-features complexity --verbose
cargo test -p tokmd-analysis --all-features content --verbose
cargo test -p tokmd-analysis --all-features derived --verbose
cargo test -p tokmd-types schema --verbose
```

Relevant proof scopes: `analysis_receipt_types`, `analysis_types_*`,
`analysis_complexity`, `analysis_content_assets`, `analysis_derived`,
`analysis_halstead`, and `analysis_maintainability`.

### Batch D: Core Facade and Binding Boundaries

Why later: `tokmd-core::run_json` and binding-facing behavior are public
contracts.

Target modules:

```text
crates/tokmd-core/src/
  lib.rs
  receipts.rs             # shared receipt construction helpers
  workflows/
    lang.rs               # language workflow owner
    module.rs             # module workflow owner
    export.rs             # file export workflow owner
    diff.rs               # diff workflow owner
    analyze.rs            # analysis workflow owner
    analyze/input.rs      # in-memory analysis export preparation
    analyze/request.rs    # analysis request construction and option parsing
    cockpit.rs            # cockpit workflow owner
    support.rs            # shared workflow support helpers
  ffi/
    mod.rs                # public run_json coordinator
    tests.rs              # FFI coordinator tests
    inputs.rs             # in-memory input decoding and path validation
    parse.rs              # strict JSON field parsing helpers
    settings_parse.rs     # mode-specific settings construction
    modes.rs              # mode dispatch owner
    envelope.rs           # response envelope owner
```

Current checkpoint: in-memory input decoding/path validation and strict JSON
field parsing have moved into `ffi/inputs.rs` and `ffi/parse.rs`, and
mode-specific settings construction has moved into `ffi/settings_parse.rs`.
Mode dispatch has moved into `ffi/modes.rs`, and response-envelope conversion
has moved into `ffi/envelope.rs`. Language, module, export, diff, analysis, and
cockpit workflow construction have moved into `workflows/lang.rs`,
`workflows/module.rs`, `workflows/export.rs`, `workflows/diff.rs`,
`workflows/analyze.rs`, and `workflows/cockpit.rs`. In-memory analysis export
preparation now lives in `workflows/analyze/input.rs`, and analysis request
construction and option parsing now live in `workflows/analyze/request.rs`.
Shared path, scan-option, and in-memory row helpers have moved into
`workflows/support.rs`, root facade tests now live in `tests.rs`, and
mutation-killing receipt/parser tests now live in `mutation_tests.rs` while the root
`tokmd_core::lang_workflow`, `tokmd_core::lang_workflow_from_inputs`,
`tokmd_core::module_workflow`, `tokmd_core::module_workflow_from_inputs`,
`tokmd_core::export_workflow`, `tokmd_core::export_workflow_from_inputs`, and
`tokmd_core::diff_workflow`, `tokmd_core::analyze_workflow`, and
`tokmd_core::analyze_workflow_from_inputs`, and `tokmd_core::cockpit_workflow`
exports remain stable. `ffi/mod.rs` now owns only the public `run_json`
coordinator and `ffi/tests.rs` owns the coordinator tests while the binding
boundary remains staged. `lib.rs` remains the root facade, and shared receipt
construction helpers live in private `receipts.rs`.

Required proof:

```bash
cargo test -p tokmd-core --all-targets --verbose
cargo test -p tokmd-core --features cockpit --test cockpit_workflow --verbose
cargo test -p tokmd-python --no-default-features --verbose
cargo check -p tokmd-node --all-targets
```

Relevant proof scopes: `tokmd_core_ffi`, `tokmd_python_binding`, and any
binding scope added for Node-specific packaging.

### Batch E: CLI and Context Packing

Why later: clap help, config precedence, snapshots, and handoff/context schemas
make this a high-visible surface.

Target modules:

```text
crates/tokmd/src/
  context_pack/
    mod.rs
    budget.rs
    select.rs
    render.rs
    manifest.rs
    output.rs
  commands/
    handoff/
      capabilities.rs
      intelligence.rs
      output.rs
    sensor/
      findings.rs
      gates.rs
  cli/
    parser/
      mod.rs
      analysis.rs
      cockpit.rs
      context.rs
  config/
    resolve.rs
crates/tokmd-types/src/
  context.rs              # context/handoff DTO owner with root re-exports
```

Required proof:

```bash
cargo test -p tokmd --test cli_snapshot_golden --verbose
cargo test -p tokmd --test cockpit_integration --verbose
cargo test -p tokmd --test context_handoff_deep --verbose
cargo xtask docs --check
```

Relevant proof scopes: `tokmd_cli`, `tokmd_context_handoff`, and
`tokmd_pipeline_integration`.

### Batch F: Model and Scan Internals

Why last: path normalization and child-language behavior are deterministic
receipt foundations.

Target modules:

```text
crates/tokmd-model/src/
  lib.rs
  aggregate.rs
  children.rs
  rows.rs
  sorting.rs

crates/tokmd-scan/src/
  roots.rs
  walk/
  path/
```

Required proof:

```bash
cargo test -p tokmd-model --verbose
cargo test -p tokmd-scan --verbose
cargo test -p tokmd --test integration --verbose
cargo test -p tokmd --test schema_sync --verbose
```

Relevant proof scope: `model_scan_path_normalization`.

## Stop Conditions

Stop and split the work if a consolidation PR:

- changes receipt JSON shape or schema version;
- changes clap help output unintentionally;
- changes package closure or publish-surface classification;
- weakens an affected proof scope;
- mixes module movement with algorithm rewrites;
- touches AST behavior beyond keeping feature-gated shadow code compiling;
- promotes coverage, mutation, fast proof, or Codecov defaults.

## First Suggested PRs

1. Continue context and handoff splits only when fresh evidence shows a broad
   owner seam; current context packing and handoff intelligence/output owners
   are already split enough for routine work.
2. Continue production owner-module splits under `tokmd-analysis` only where a
   product or proof seam remains broad. Derived analysis already has coherent
   owner modules for distribution, integrity, ratios, file metrics, and
   language composition.
3. Continue CLI parser command-family splits only when clap snapshots prove
   behavior is unchanged; continue model child-language behavior only if future
   evidence shows the seam is still too broad after the row owner split.
4. Treat future core facade or FFI work as binding-contract work, not routine
   owner-module cleanup, unless a concrete helper seam appears.

Each PR should include the affected proof-plan output in the PR body and should
leave `publish-surface --verify-publish` green when public exports or
manifests are touched.
