# How To Extend tokmd Analysis

Status: contributor extender guide for the `tokmd analyze` receipt
(`AnalysisReceipt`, schema family `analysis`).

Use this guide when you want to add an **analysis enricher**, wire a section
into a **preset**, or render an existing section in a new **output format**. It
explains where each contract lives and the proof a contributor PR should run. It
does not restate every analysis section; the receipt shape lives in
[`docs/SCHEMA.md`](SCHEMA.md) and the type definitions.

For the broader "first useful contribution" flow, start from the
[contributor guide](contributor-guide.md). To add a field to the PR **evidence
packet** manifest instead of the analysis receipt, use
[how to add a packet field](how-to-add-a-packet-field.md).

> **Language support is upstream.** Per-language line/comment/blank counting
> comes from the `tokei` dependency, isolated in `tokmd-scan`. Adding or fixing a
> language is a `tokei` change, not a tokmd extension point, so it is out of
> scope for this guide.

## The Three Extension Shapes

| Shape | What you add | Primary crates |
| --- | --- | --- |
| Enricher | A new optional section on `AnalysisReceipt` computed from the scan | `tokmd-analysis-types`, `tokmd-analysis` |
| Preset | A new named bundle of enrichers, or a section toggled in an existing preset | `tokmd-analysis` (`grid`) |
| Output format | A new rendering of the existing receipt | `tokmd-format` (`analysis`) |

An enricher and a preset toggle usually ship together: the enricher computes the
data, and one or more presets decide when it runs.

## Where The Contract Lives

Touch these together for an enricher:

| Surface | Path | Role |
| --- | --- | --- |
| Result type | `crates/tokmd-analysis-types/src/<section>.rs` | Serde struct for the section, re-exported from `lib.rs`. |
| Receipt field | `crates/tokmd-analysis-types/src/receipt.rs` | Adds the `Option<T>` field to `AnalysisReceipt`. |
| Schema version | `crates/tokmd-analysis-types/src/lib.rs` | `ANALYSIS_SCHEMA_VERSION` and its assertion test. |
| Enricher compute | `crates/tokmd-analysis/src/<section>.rs` | Pure computation from the scan/export inputs. |
| Enricher wiring | `crates/tokmd-analysis/src/analysis/enrichers/<group>.rs` | `run_<section>` that checks the preset plan and fills outputs. |
| Outputs holder | `crates/tokmd-analysis/src/analysis/outputs.rs` | The `AnalysisOutputs` field the enricher fills. |
| Receipt assembly | `crates/tokmd-analysis/src/analysis/mod.rs` | Copies `outputs.<section>` into the returned `AnalysisReceipt`. |
| Preset plan | `crates/tokmd-analysis/src/grid/presets.rs` | `PresetPlan` flag and every `PRESET_GRID` row. |
| Feature flag | `crates/tokmd-analysis/Cargo.toml` | Optional-dependency / feature gate when the enricher is heavy. |
| Rendering | `crates/tokmd-format/src/analysis/markdown.rs` (+ `markdown/<section>.rs`) | Human-facing Markdown; other formats live beside it under `analysis/`. |
| Schema docs | `docs/SCHEMA.md`, `docs/schema.json` | Formal receipt documentation and validation. |

## Adding An Enricher

The analysis pipeline is a fixed set of enricher groups. Each group's `run(...)`
fills a shared `AnalysisOutputs` struct, and `analyze` copies those outputs into
`AnalysisReceipt`. A section is only computed when its preset-plan flag is set.

Worked example: add an optional `naming` section (illustrative — choose a
deterministic, meaningful name for real work).

1. **Define the result type** in `crates/tokmd-analysis-types/src/naming.rs` and
   re-export it from `lib.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingReport {
    pub snake_case_files: usize,
    pub other_case_files: usize,
}
```

2. **Add the receipt field** to `AnalysisReceipt` in `receipt.rs`:

```rust
pub naming: Option<NamingReport>,
```

3. **Add the outputs field** in `analysis/outputs.rs`:

```rust
pub(super) naming: Option<NamingReport>,
```

4. **Add the compute module** `crates/tokmd-analysis/src/naming.rs` with a pure
   function over inputs the pipeline already has (for example `&ExportData`).
   Keep it deterministic (sorted collections, no wall-clock, no I/O beyond the
   files the pipeline already collected).

5. **Wire the enricher** in the matching group under
   `analysis/enrichers/`. Follow the existing `run_<section>` pattern, which
   checks `plan.<flag>` and, when a heavy feature is off, pushes a
   `DisabledFeature` warning instead of silently dropping the section:

```rust
fn run_naming(
    export: &ExportData,
    plan: &PresetPlan,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    let _ = warnings;
    if plan.naming {
        outputs.naming = Some(crate::naming::build_naming_report(export));
    }
}
```

   Call `run_naming` from the group's `run(...)`. A cheap enricher over the
   in-memory export (like `semantic`) needs no feature gate; a filesystem- or
   git-backed enricher belongs in the `content`, `git`, or `code_quality` group
   behind its feature flag, mirroring the `#[cfg(feature = "...")]` /
   `DisabledFeature` branches already there.

6. **Assemble the field** in `analyze` (`analysis/mod.rs`) by adding
   `naming: outputs.naming,` to the returned `AnalysisReceipt`.

7. **Gate it by preset** in `grid/presets.rs`: add a `naming: bool` field to
   `PresetPlan` and set it in **every** `PRESET_GRID` row. The
   `preset_table_covers_all_presets` and roundtrip tests guard grid coverage, so
   a missing row will fail to compile or fail a test.

8. **Bump the schema.** Adding a receipt field changes the serialized shape, so
   increment `ANALYSIS_SCHEMA_VERSION` in
   `crates/tokmd-analysis-types/src/lib.rs`, update the assertion test there,
   update the version note in `crates/tokmd-analysis-types/CLAUDE.md`, and update
   `docs/SCHEMA.md` / `docs/schema.json`.

9. **Render it** in `crates/tokmd-format/src/analysis/`. Add a
   `markdown/naming.rs` renderer and call it from `markdown.rs` when
   `receipt.naming` is present. Other formats (JSON is the receipt itself; SVG,
   HTML, tree, etc.) live beside it and only need changes if the section should
   appear there.

## Adding Or Adjusting A Preset

Presets are identity plus a plan of which enrichers run.

- To **toggle an existing section** in an existing preset, flip its flag in that
  preset's `PRESET_GRID` row in `grid/presets.rs`.
- To **add a new preset**, add a variant to `PresetKind`, its slug to `as_str`
  and `from_str`, an entry to `PRESET_KINDS` (and its length const), and a new
  `PRESET_GRID` row. The CLI preset argument in the `tokmd analyze` command and
  any preset lists in docs (`docs/reference-cli.md`, `AGENTS.md`, and the
  `tokmd-analysis` / root `CLAUDE.md` preset tables) should be updated to match.

Keep presets purposeful: a preset is a support promise about which sections a
caller gets, so avoid one-off presets that only shuffle flags.

## Adding An Output Format

The receipt is the source of truth; formats are pure renderings under
`crates/tokmd-format/src/analysis/`. To add a format, add a module beside the
existing ones (`markdown`, `svg`, `html`, `tree`, `mermaid`, `jsonld`, `xml`),
render from `&AnalysisReceipt`, and route to it from the format dispatch. Do not
add analysis computation in `tokmd-format`; if the format needs data that is not
on the receipt, add an enricher first.

## Schema And Compatibility Rules

- Any change to the serialized `AnalysisReceipt` shape (new field, renamed
  field, changed meaning) is a schema change: bump `ANALYSIS_SCHEMA_VERSION` and
  update `docs/SCHEMA.md` / `docs/schema.json` in the same PR.
- Prefer additive `Option<T>` sections that default to absent so existing
  consumers keep working.
- Keep output deterministic: `BTreeMap` for maps, sorted vectors, no wall-clock
  values in the section body.
- Do not promote a shadow/experimental signal onto default receipts without a
  separate schema proposal.

## Proof

Run the proof that matches the surfaces you touched. For an enricher that adds a
type, wiring, a preset flag, and rendering:

```bash
cargo test -p tokmd-analysis-types
cargo test -p tokmd-analysis
cargo test -p tokmd-analysis --all-features
cargo test -p tokmd-format
cargo xtask docs --check
cargo xtask doc-artifacts --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan.json
git diff --check
```

Add a targeted test for the new section: assert the compute function on a small
fixture, and assert that a preset which enables the flag produces the section
while one that disables it omits it.

State the claim boundary in the PR body: a new enricher proves the section is
computed deterministically, gated by its preset flag, serialized on the receipt,
and rendered. It does not change the meaning of existing sections, promote any
advisory or shadow signal into a required gate, or prove anything about safety,
correctness, or merge readiness beyond what the section measures.

## Related Docs

- [Architecture](architecture.md) — crate tiers and dependency direction
- [Schema reference](SCHEMA.md) — receipt shape and versioning
- [How to add a packet field](how-to-add-a-packet-field.md) — evidence packet
  manifest extension
- [Contributor guide](contributor-guide.md) — first-contribution flow and proof
  commands
- [Testing strategy](testing.md) — test types and how to run them
