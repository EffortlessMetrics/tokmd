# Agent Guidance

Canonical shared repo guidance is mirrored in `agents/shared/repo.md`.
This file provides guidance to AI agents (Claude, Factory Droid, etc.) when working with code in this repository.

## Repo Source-of-Truth Stack

tokmd uses a linked source-of-truth stack:

```text
Roadmap -> Proposal -> Spec -> ADR -> Plan -> Active goal -> PR -> Proof
```

Before changing source-of-truth artifacts or selecting implementation work, read:

1. `docs/reference/SPEC_SYSTEM.md`
2. `docs/source-of-truth.md`
3. `.jules/goals/active.toml`
4. the linked implementation plan
5. the linked spec for the selected work item
6. linked ADRs

Work on one semantic artifact or one implementation work item per PR unless the
selected plan item says otherwise. Do not broaden support claims without proof,
do not hand-edit generated status unless the plan names that as the required
change, and stop instead of guessing when linked artifacts are missing or a
request conflicts with an ADR.

## Droid Auto Review

**tokmd** uses Factory Droid for automated code review. Droid runs on all pull requests using the safe action configuration with MiniMax BYOK, and can be invoked manually with `@droid review` or `@droid security` comments.

- See `agents/shared/droid-migration.md` for the rollout design
- See `.factory/rules/droid-review.md` for review standards and finding format
- See `.github/workflows/droid-review.yml` for auto-review configuration
- See `.github/workflows/droid.yml` for manual `@droid` command handling
- See `.github/workflows/droid-security-scan.yml` for scheduled security scanning

## Project Overview

**tokmd** is a Rust CLI tool and library that wraps the `tokei` library to generate "inventory receipts" and derived analytics of code repositories. It produces human-readable summaries (Markdown/TSV) and machine-friendly datasets (JSON/JSONL/CSV) for AI-native workflows, LLM context generation, and code analysis pipelines.

## Build and Test Commands

```bash
cargo build                          # Debug build
cargo build --release                # Release build with LTO
cargo test --verbose                 # Run all tests
cargo fmt-fix                        # Format code across the workspace
cargo fmt-check                      # Verify workspace formatting
cargo trim-target --check            # Report reclaimable target/debug space
cargo sccache-check                  # Verify local sccache setup
cargo clippy -- -D warnings          # Lint with strict warnings
cargo install --path crates/tokmd    # Local install
```

On Windows, prefer `cargo fmt-fix` / `cargo fmt-check` over `cargo fmt --all`; the full workspace can exceed Cargo's formatter argv budget even when long paths are enabled.
Windows MSVC builds in this repo also default to line-table debuginfo to keep `target/debug` from being dominated by full PDBs.
If you need full local symbols for a debugging session, use `$env:RUSTFLAGS='-C debuginfo=2'; cargo test ...`.
For opt-in local build caching, use `cargo with-sccache ...`; the wrapper sets `RUSTC_WRAPPER=sccache` and defaults `CARGO_INCREMENTAL=0` unless you pass `--keep-incremental`.
For cross-worktree cache reuse, use `cargo xtask sccache --basedir <PATH> -- <cargo args>` so the wrapper can set `SCCACHE_BASEDIRS` explicitly.

## Architecture

The codebase follows a tiered crate-and-module architecture:
**types → scan/model → format/adapters → analysis/cockpit/gate → core → products**.
Public crates represent durable contracts, facades, adapters, or products.
Implementation details that do not need an independent package live as
single-responsibility owner modules inside those crates.

### Crate Hierarchy

| Tier | Crate | Purpose |
|------|-------|---------|
| 0 | `tokmd-types` | Core receipt data structures |
| 0 | `tokmd-analysis-types` | Analysis receipt types |
| 0 | `tokmd-settings` | Clap-free workflow settings |
| 0 | `tokmd-envelope` | Shared sensor/FFI envelope contracts |
| 0 | `tokmd-io-port` | Host-abstracted file access contracts |
| 1 | `tokmd-scan` | tokei wrapper for code scanning |
| 1 | `tokmd-model` | Aggregation logic (lang, module, file rows) |
| 1 | `tokmd-sensor` | Sensor substrate and report builder |
| 2 | `tokmd-format` | Output rendering, redaction, badges, export-tree, analysis rendering |
| 2 | `tokmd-git` | Git history analysis |
| 3 | `tokmd-analysis` | Analysis orchestration and enrichers |
| 3 | `tokmd-cockpit` | PR cockpit metrics and review evidence |
| 3 | `tokmd-gate` | Policy evaluation with JSON pointer rules |
| 4 | `tokmd-core` | Library facade with FFI layer |
| 5 | `tokmd` | CLI binary |
| 5 | `tokmd-python` | PyO3 bindings for PyPI |
| 5 | `tokmd-node` | napi-rs bindings for npm |
| 5 | `tokmd-wasm` | wasm-bindgen bindings for browser/worker callers |

Former helper microcrates such as redaction, scan-args, badge rendering,
analysis rendering, progress, module-key, path/exclude/math, tokeignore,
context policy/git, fun renderers, content/import enrichers, and tool-schema now
live as owner modules inside `tokmd-format`, `tokmd-scan`, `tokmd-model`,
`tokmd-analysis`, `tokmd-core`, or `tokmd`.

### CLI Commands

- `tokmd` / `tokmd lang` — Language summary
- `tokmd module` — Module breakdown by directory
- `tokmd export` — File-level inventory (JSONL/CSV/CycloneDX)
- `tokmd run` — Full scan with artifact output
- `tokmd analyze` — Derived metrics and enrichments
- `tokmd badge` — SVG badge generation
- `tokmd diff` — Compare two runs or receipts
- `tokmd cockpit` — PR metrics for code review with evidence gates
- `tokmd gate` — Policy-based quality gates with JSON pointer rules
- `tokmd tools` — Generate LLM tool definitions (OpenAI, Anthropic, JSON Schema)
- `tokmd context` — Pack files into LLM context window within token budget
- `tokmd init` — Generate `.tokeignore` template
- `tokmd check-ignore` — Explain why files are being ignored
- `tokmd completions` — Generate shell completions

### Library API (tokmd-core)

The `tokmd-core` crate provides a clap-free library facade for embedding:

**Workflow Functions** (Rust):
- `lang_workflow(scan, lang) -> LangReceipt`
- `module_workflow(scan, module) -> ModuleReceipt`
- `export_workflow(scan, export) -> ExportReceipt`
- `diff_workflow(settings) -> DiffReceipt`

**FFI Layer** (`ffi::run_json`):
- Single JSON entrypoint: `run_json(mode, args_json) -> String`
- Modes: `lang`, `module`, `export`, `analyze`, `diff`, `version`
- Response envelope: `{"ok": bool, "data": {...}, "error": {...}}`

**Python Bindings** (tokmd-python):
- `tokmd.lang()`, `tokmd.module()`, `tokmd.export()`, `tokmd.analyze()`, `tokmd.diff()`
- Returns native Python dicts
- Releases GIL during long scans

**Node.js Bindings** (tokmd-node):
- All functions return Promises (async)
- Uses `tokio::task::spawn_blocking()` for non-blocking event loop

### Analysis Presets

| Preset | Includes |
|--------|----------|
| `receipt` | Core derived metrics (density, distribution, COCOMO) |
| `health` | + TODO density |
| `risk` | + Git hotspots, coupling, freshness |
| `supply` | + Assets, dependency lockfiles |
| `architecture` | + Import graph |
| `topics` | Semantic topic clouds |
| `security` | License radar, entropy profiling |
| `identity` | Archetype detection, corporate fingerprint |
| `git` | Predictive churn, advanced git metrics |
| `deep` | Everything (except fun) |
| `fun` | Eco-label, novelty outputs |

## Critical Patterns

### Deterministic Output
- Uses `BTreeMap` instead of `HashMap` everywhere for stable key ordering
- Sorting: descending by code lines, then by name
- Essential for golden snapshot tests and reproducible receipts

### Path Normalization
- All paths normalized to forward slashes (`/`) regardless of OS
- Always use `normalize_path()` before output
- Module keys computed from normalized paths

### Children/Embedded Language Handling
- `ChildrenMode::Collapse`: Merge embedded languages into parent totals
- `ChildrenMode::Separate`: Show as "(embedded)" rows
- Applies consistently across all commands

### Receipt Schema
- JSON outputs include envelope metadata with `schema_version`
- Increment schema_version when modifying JSON output structure
- Update `docs/schema.json` (formal JSON Schema) when structures change
- **Schema versions are separate for each receipt family**:
  - Core receipts (`lang`, `module`, `export`, `diff`, `run`): `SCHEMA_VERSION = 2` (in `tokmd-types`)
  - Analysis receipts: `ANALYSIS_SCHEMA_VERSION = 9` (in `tokmd-analysis-types`)
  - Cockpit receipts: `COCKPIT_SCHEMA_VERSION = 3` (in `tokmd-types`)
  - Handoff manifests: `HANDOFF_SCHEMA_VERSION = 5` (in `tokmd-types`)
  - Context receipts: `CONTEXT_SCHEMA_VERSION = 4` (in `tokmd-types`)
  - Context bundles: `CONTEXT_BUNDLE_SCHEMA_VERSION = 2` (in `tokmd-types`)

### Feature Flags
- `git`: Git history analysis (uses shell `git log`)
- `content`: File content scanning (entropy, tags, hashing)
- `walk`: Filesystem traversal for assets

### Git Diff Syntax (Two-dot vs Three-dot)
When invoking `git diff` or `git log` with range syntax:

| Syntax | Meaning | Use Case |
|--------|---------|----------|
| `A..B` | Commits reachable from B but not A | Comparing tags/releases (`cockpit`, `diff` commands) |
| `A...B` | Symmetric difference from merge-base | CI workflows comparing PR branches |

**Rule**: Use `..` (two dots) in cockpit/diff commands comparing releases or tags. Use `...` (three dots) only in CI workflows where you want changes since branch divergence.

## Testing

- **Integration tests**: `crates/tokmd/tests/` using `assert_cmd` + `predicates`
- **Golden snapshots**: Using `insta` crate (timestamps normalized)
- **Crate-level tests**: Each crate has its own `tests/` directory
- **Unit tests**: In-module tests
- **Property-based tests**: Using `proptest` across 14 crates for invariant verification
- **Fuzz targets**: 9 targets using `libfuzzer` (see `fuzz/` directory) with seed corpus and dictionaries
- **Mutation testing**: Using `cargo-mutants` for test quality verification (configured in `.cargo/mutants.toml`)

Run a single test:
```bash
cargo test test_name --verbose
```

Update snapshots:
```bash
cargo insta review
```

Run property tests:
```bash
cargo test -p tokmd-scan properties
```

Run mutation testing:
```bash
cargo mutants --file crates/tokmd-format/src/redact/mod.rs
```

Run fuzz targets:
```bash
cargo +nightly fuzz run fuzz_entropy --features content
cargo +nightly fuzz list  # List all targets
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `tokei` | Core LOC counting |
| `clap` (derive) | CLI parsing |
| `serde`/`serde_json` | JSON serialization |
| `blake3` | Fast hashing for redaction and integrity |
| `anyhow` | Error handling |
| `ignore` | File walking with gitignore support |
| `pyo3` | Python bindings (tokmd-python) |
| `napi-rs` | Node.js bindings (tokmd-node) |

## Documentation

### Architecture & Design
- `docs/architecture.md`: Crate hierarchy, data flow, dependency rules
- `docs/design.md`: Design principles, system context, data model
- `docs/requirements.md`: Requirements, interfaces, quality bar
- `docs/implementation-plan.md`: Phased roadmap for future work

### User Guides
- `docs/tutorial.md`: Getting started guide
- `docs/recipes.md`: Real-world usage examples
- `docs/reference-cli.md`: CLI flag reference
- `docs/troubleshooting.md`: Common issues and solutions

### Technical Reference
- `docs/SCHEMA.md`: Receipt format documentation
- `docs/schema.json`: Formal JSON Schema Draft 7 definition
- `docs/testing.md`: Testing strategy and frameworks

### Product & Philosophy
- `docs/PRODUCT.md`: Product contract and invariants
- `docs/explanation.md`: Philosophy and design principles

### Project
- `ROADMAP.md`: Current status and future plans
- `CHANGELOG.md`: Version history
- `CONTRIBUTING.md`: Development setup, testing, and publishing guide

## PR Triage Rules

### Jules provenance is intentional repo state
- Treat `.jules/**` logs, ledgers, envelopes, runbooks, and friction notes as intentional provenance unless a repo owner explicitly says otherwise.
- Do not close, trim, or reject a PR merely because it includes `.jules/**` files.
- If a Jules PR is merge-worthy, keep the `.jules/**` updates with it.
- Do not split `.jules/**` updates out into a second PR just to shrink the visible diff; keep the provenance attached to the substantive change.
- If a Jules PR is not ready, the reason must be the actual code issue: correctness, scope, stacking, validation, or launch relevance.

### When a Jules PR is bad
- Do not close a Jules PR because it has provenance files.
- Close, defer, or restack only for substantive reasons such as:
  - unrelated non-`.jules/**` changes are stacked into the branch
  - the code is incorrect
  - required validation fails
  - the change is not worth merging in the current phase

### Release-prep queue discipline
- During prep or RC hardening, every open PR must either be a real blocker/improvement worth finishing now or remain explicitly parked for later.
- “Not for prep” does not automatically mean “close now”.
- If a PR may still be worth keeping, leave it open or restack it; do not churn the queue without a concrete reason.
