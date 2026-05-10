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
| Analysis API surface | `crates/tokmd-analysis/src/api_surface/mod.rs` and `crates/tokmd-analysis/src/api_surface/` | `mod.rs` 230; symbol scanner 385; symbol tests 449 | Keep report aggregation in `mod.rs`, source scanning and scanner tests under `symbols`, and leave large integration tests under `api_surface/tests` |
| Context packing | `crates/tokmd/src/context_pack.rs`, `crates/tokmd/src/context_pack/`, and `crates/tokmd/src/commands/context.rs` | `context_pack.rs` 11; selection submodule/tests 1896; render submodule 204; manifest submodule 195; budget parser/tests 220; context command 423 | Budget parsing, file selection, bundle text rendering, and context bundle manifest writing now live in owner modules; reassess remaining command orchestration before moving to the next pressure point; keep context/handoff proof scoped |
| Analysis DTO contracts | `crates/tokmd-analysis-types/src/lib.rs` and owner DTO modules | `lib.rs` 113; baseline owner 37 + complexity-baseline submodule 256 + complexity-section submodule 37 + determinism submodule 22 + metrics submodule 45 + file-entry submodule 23; envelope owner 24; receipt owner 42; topics owner tests 42; entropy owner tests 58; license owner tests 42; churn owner tests 50; complexity owner tests 51 + file submodule 46 + risk submodule 43 + halstead submodule 30 + maintainability submodule 25 + histogram submodule 79 + technical-debt submodule 42; effort owner 25 + estimate submodule 70 + assumptions submodule 35 + cocomo submodule 48 + model submodule 37 + confidence submodule 45 + delta submodule 56 + driver submodule 43 + size submodule 65 + results submodule 45 | Keep root receipt glue and public re-exports stable while moving remaining DTO ownership into modules |
| Core facade and FFI | `crates/tokmd-core/src/lib.rs`, `crates/tokmd-core/src/ffi.rs` | 1500 each | Split workflow facade, FFI envelope handling, and mode dispatch without changing `run_json` |
| Analysis complexity | `crates/tokmd-analysis/src/complexity/mod.rs` + `complexity/functions.rs` + `complexity/details.rs` + `complexity/summary.rs` + `complexity/risk.rs` + `complexity/debt.rs` + `complexity/histogram.rs` + `complexity/language.rs` + `complexity/math.rs` + `complexity/tests/unit.rs` | 156 + 301 + 343 + 138 + 78 + 69 + 33 + 35 + 5 + 346 | Keep shared complexity logic in `tokmd-analysis`, split language/source/summary helpers and local unit tests |
| CLI parser | `crates/tokmd/src/cli/parser.rs` | 1276 | Split command argument families while preserving clap output |
| Model aggregation | `crates/tokmd-model/src/lib.rs` | 1159 | Split aggregation, row sorting, and child-language behavior under `tokmd-model` |

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
  ffi/
    mod.rs
    modes.rs
    envelope.rs
```

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

1. Split `tokmd-analysis-types/src/lib.rs` into DTO-family modules while
   preserving re-exports.
2. Continue production owner-module splits under `tokmd-analysis`, starting
   with API surface symbol scanning and then aggregation/test cleanup where
   useful.
3. Reassess the remaining context command orchestration now that selection,
   budgeting, rendering, and manifest helpers live under `tokmd`.

Each PR should include the affected proof-plan output in the PR body and should
leave `publish-surface --verify-publish` green when public exports or
manifests are touched.
