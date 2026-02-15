# Explanation: The Philosophy of tokmd

## The Core Concept

**tokmd turns "counting" into "receipts" and "receipts" into "insights".**

`tokei` is a fantastic **counting engine**. It tells you how many lines of code exist.
`tokmd` is the **packaging and analysis layer**. It runs the scan and emits **artifact-shaped outputs** that humans and pipelines can trust and reuse, without shell glue. Then it derives **actionable insights** from those artifacts.

## The Evolution

### Phase 1: Receipts (v1.0)
Raw counts are packaged into deterministic, versioned artifacts with provenance.

### Phase 2: Analysis (v1.1+)
Receipts become the foundation for derived metrics: doc density, test density, distributions, effort estimation, context window planning.

### Phase 3: Intelligence (v1.2+)
Analysis incorporates external signals: git history (hotspots, freshness, coupling), file content (entropy, licenses), and semantic patterns (archetypes, topics). Context packing enables budget-aware file selection for LLM workflows.

## The Problems We Solve

### 1. "Repo shape" is useful, but tooling is friction-heavy
Raw counting tools output to terminals. Using them in PRs or pipelines requires fragile chains of `jq`, `column`, and shell scripts. `tokmd` replaces that glue with a single, cross-platform command that outputs stable artifacts.

### 2. LLM workflows need a map, not a dump
Pasting source code into an LLM wastes tokens and leaks context. Agents need a map first:
- What is here? (Languages)
- Where is the mass? (Modules)
- Which files are heavy? (Export rows)
- Will it fit? (Context window analysis)
- What should I include? (Context packing)

`tokmd` provides this map as a compact, structured dataset. The `context` command goes further, intelligently selecting files to pack into a context window within a token budget.

### 3. Preventing Process Confabulation
In automated workflows, the common failure mode is narrative ("I checked the files"). `tokmd` enforces a "receipt" posture: outputs are deterministic, versioned, and machine-verifiable. Text is untrusted; artifacts are trusted.

### 4. "Shape, not grade"
`tokmd` is explicitly **not** a productivity metric. It is a sensor for inventory, distribution, and drift detection. This aligns with the philosophy of "trusted change" rather than LOC theater.

### 5. Analysis without judgment
The analysis features provide signals, not scores. Doc density isn't "good" or "bad" — it's information. Hotspots aren't "problems" — they're areas that may warrant attention. Users interpret the data in their context.

## What is a Receipt?

A receipt is more than just JSON output. It is a **contract**.

Every `tokmd` output includes:
- `schema_version`: To allow safe evolution.
- `tool` & `args`: Provenance of how the data was generated.
- `scan` configuration: What was ignored/included.
- `totals` & `rows`: The data itself.
- `integrity`: A hash of the content for verification.

This structure allows downstream tools (dashboards, diff engines, agents) to consume the data without guessing.

## What is Analysis?

Analysis builds on receipts to derive higher-order insights:

### Derived Metrics (Zero I/O)
Computed purely from receipt data:
- **Doc Density**: Comments as a fraction of code
- **Test Density**: Test code vs production code
- **Distribution**: Statistical properties of file sizes
- **COCOMO**: Effort estimation from KLOC

### Enriched Metrics (Optional I/O)
Require additional scanning:
- **Git Metrics**: Hotspots, freshness, coupling (requires git history)
- **Content Metrics**: Entropy, licenses, imports (requires file reads)
- **Asset Metrics**: Non-code file inventory (requires filesystem walk)

### Presets
Presets bundle related analyses:
- `receipt`: Core derived metrics only
- `health`: Add TODO density
- `risk`: Add git hotspots and freshness
- `deep`: Everything available

## Trust Boundaries

- **Text** is untrusted.
- **Artifacts** (receipts) are trusted.
- **Analysis** is derived from trusted artifacts.

By generating a receipt, you create a boundary object that can be passed between agents or stored as evidence of a repository's state at a specific commit.

## Architecture Philosophy

### Microcrates
The codebase is split into focused crates:
- **Types crates** (Tier 0): Pure data structures, no I/O
- **Logic crates** (Tier 1-2): Scanning, formatting, I/O operations
- **Analysis crates** (Tier 3): Enrichment and derivation
- **Orchestration crates** (Tier 4-5): Config, CLI, facade

This enables:
- Selective compilation (skip git features if not needed)
- Library reuse (use `tokmd-analysis` without the CLI)
- Clear dependency boundaries

### Feature Flags
Heavy dependencies are feature-gated:
- `git`: Git history analysis (shells out to `git` command)
- `content`: File content scanning
- `walk`: Filesystem traversal

The default build includes everything; library users can opt out.

## Design Principles

1. **Determinism over convenience**: Same input always produces same output.
2. **Artifacts over assertions**: Show the receipt, don't claim you checked.
3. **Signals over scores**: Provide information, not judgments.
4. **Additive schema evolution**: New fields don't break old consumers.
5. **Progressive disclosure**: Simple commands for simple needs, depth when required.
