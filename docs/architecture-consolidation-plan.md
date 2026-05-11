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
| Analysis API surface | `crates/tokmd-analysis/src/api_surface/mod.rs` and `crates/tokmd-analysis/src/api_surface/` | `mod.rs` 13; report owner 227; symbol dispatcher 56; language scanners <=68; symbol tests 385 | Keep `mod.rs` as a thin coordinator, report aggregation in `report.rs`, source scanning in language owner modules under `symbols`, and leave large integration tests under `api_surface/tests` |
| Context packing | `crates/tokmd/src/context_pack.rs`, `crates/tokmd/src/context_pack/`, and `crates/tokmd/src/commands/context.rs` | `context_pack.rs` 15; selection coordinator/tests 1750; ranking/packing owner 165; output/log submodule 224; render submodule 204; manifest submodule 195; budget parser/tests 220; context command 218 | Budget parsing, file selection, ranking/packing, bundle text rendering, single-output/log writing, and context bundle manifest writing now live in owner modules; keep context/handoff proof scoped while CLI parser splits continue |
| Analysis DTO contracts | `crates/tokmd-analysis-types/src/lib.rs` and owner DTO modules | `lib.rs` 113; baseline owner 37 + complexity-baseline submodule 256 + complexity-section submodule 37 + determinism submodule 22 + metrics submodule 45 + file-entry submodule 23; envelope owner 24; receipt owner 42; topics owner tests 42; entropy owner tests 58; license owner tests 42; churn owner tests 50; complexity owner tests 51 + file submodule 46 + risk submodule 43 + halstead submodule 30 + maintainability submodule 25 + histogram submodule 79 + technical-debt submodule 42; effort owner 25 + estimate submodule 70 + assumptions submodule 35 + cocomo submodule 48 + model submodule 37 + confidence submodule 45 + delta submodule 56 + driver submodule 43 + size submodule 65 + results submodule 45 | Keep root receipt glue and public re-exports stable while moving remaining DTO ownership into modules |
| Core facade and FFI | `crates/tokmd-core/src/lib.rs`, `crates/tokmd-core/src/workflows/`, `crates/tokmd-core/src/ffi.rs`, and `crates/tokmd-core/src/ffi/` | `lib.rs` 879; workflow coordinator 25; language workflow owner 77; module workflow owner 87; export workflow owner 98; diff workflow owner 41; analysis workflow owner 400; cockpit workflow owner 90; `ffi.rs` 1016; envelope owner 15; mode owner 149; input owner 187; parse owner 257; settings parser owner 150 | Language, module, export, diff, analysis, and cockpit workflow facades now live under `workflows/`; in-memory input decoding/path validation, strict JSON parsing, FFI settings construction, mode dispatch, and response-envelope conversion live under `ffi/`; remaining work is public FFI coordinator cleanup without changing public APIs |
| Analysis complexity | `crates/tokmd-analysis/src/complexity/mod.rs` + `complexity/functions.rs` + `complexity/details.rs` + `complexity/summary.rs` + `complexity/risk.rs` + `complexity/debt.rs` + `complexity/histogram.rs` + `complexity/language.rs` + `complexity/math.rs` + `complexity/tests/unit.rs` | 156 + 301 + 343 + 138 + 78 + 69 + 33 + 35 + 5 + 346 | Keep shared complexity logic in `tokmd-analysis`, split language/source/summary helpers and local unit tests |
| CLI parser | `crates/tokmd/src/cli/parser.rs` and `crates/tokmd/src/cli/parser/` | `parser.rs` 209; command enum owner 103; global scan args owner 133; shared value-enum owner 235; analyze parser owner 178; context/handoff parser owner 166; cockpit/baseline parser owner 89; diff parser owner 67; gate parser owner 55; sensor parser owner 43; export parser owner 43; badge parser owner 45; module parser owner 36; lang parser owner 26; run parser owner 25; check-ignore parser owner 15; completions parser owner 21; init parser owner 36; tools parser owner 15 | Context, handoff, analyze, cockpit, baseline, diff, gate, sensor, export, badge, module, lang, run, check-ignore, completions, init, tools, command enum, global scan args, and shared value enums now live under parser owner modules; continue command-family splits only when clap snapshots prove behavior is unchanged |
| Model aggregation | `crates/tokmd-model/src/lib.rs`, `crates/tokmd-model/src/aggregate.rs`, `crates/tokmd-model/src/rows.rs`, and `crates/tokmd-model/src/sorting.rs` | `lib.rs` 267; aggregation owner/tests 427; row collection owner/tests 411; sorting owner/tests 91 | Report builders, file-row collection, in-memory row detection, and row sorting now live in owner modules; remaining work is child-language behavior only if future evidence shows the seam is still too broad |

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
  maintainability/
  halstead/
```

Required proof:

```bash
cargo test -p tokmd-analysis-types --verbose
cargo test -p tokmd-analysis --all-features complexity --verbose
cargo test -p tokmd-analysis --all-features content --verbose
cargo test -p tokmd-types schema --verbose
```

Relevant proof scopes: `analysis_receipt_types`, `analysis_types_*`,
`analysis_complexity`, `analysis_content_assets`, `analysis_halstead`, and
`analysis_maintainability`.

### Batch D: Core Facade and Binding Boundaries

Why later: `tokmd-core::run_json` and binding-facing behavior are public
contracts.

Target modules:

```text
crates/tokmd-core/src/
  lib.rs
  workflows/
    lang.rs               # language workflow owner
    module.rs             # module workflow owner
    export.rs             # file export workflow owner
    diff.rs               # diff workflow owner
    analyze.rs            # analysis workflow owner
    cockpit.rs            # cockpit workflow owner
  ffi.rs                  # public run_json coordinator during staged split
  ffi/
    inputs.rs             # in-memory input decoding and path validation
    parse.rs              # strict JSON field parsing helpers
    settings_parse.rs     # mode-specific settings construction
    mod.rs                # final coordinator once ffi.rs is split
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
`workflows/analyze.rs`, and `workflows/cockpit.rs` while the root
`tokmd_core::lang_workflow`, `tokmd_core::lang_workflow_from_inputs`,
`tokmd_core::module_workflow`, `tokmd_core::module_workflow_from_inputs`,
`tokmd_core::export_workflow`, `tokmd_core::export_workflow_from_inputs`, and
`tokmd_core::diff_workflow`, `tokmd_core::analyze_workflow`, and
`tokmd_core::analyze_workflow_from_inputs`, and `tokmd_core::cockpit_workflow`
exports remain stable. `ffi.rs` still owns public `run_json` while that public
binding boundary remains staged, and `lib.rs` remains the root facade and shared
receipt helper owner.

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
  cli/
    parser/
      mod.rs
      analysis.rs
      cockpit.rs
      context.rs
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
  rows.rs
  children.rs
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

1. Continue core facade owner-module splits with the remaining workflow
   families while preserving the root-level public exports and binding
   contracts.
2. Continue production owner-module splits under `tokmd-analysis` only where
   production seams remain broad; avoid splitting tests solely for line count.
3. Continue CLI parser command-family splits only when clap snapshots prove
   behavior is unchanged; continue model child-language behavior only if future
   evidence shows the seam is still too broad after the row owner split.

Each PR should include the affected proof-plan output in the PR body and should
leave `publish-surface --verify-publish` green when public exports or
manifests are touched.
