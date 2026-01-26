# Explanation: The Philosophy of Receipts

## The Core Concept

**tokmd turns "counting" into "receipts".**

`tokei` is a fantastic **counting engine**. It tells you how many lines of code exist.
`tokmd` is the **packaging layer**. It runs the scan and emits **artifact-shaped outputs** that humans and pipelines can trust and reuse, without shell glue.

## The Problems We Solve

### 1. "Repo shape" is useful, but tooling is friction-heavy
Raw counting tools output to terminals. Using them in PRs or pipelines requires fragile chains of `jq`, `column`, and shell scripts. `tokmd` replaces that glue with a single, cross-platform command that outputs stable artifacts.

### 2. LLM workflows need a map, not a dump
Pasting source code into an LLM wastes tokens and leaks context. Agents need a map first:
- What is here? (Languages)
- Where is the mass? (Modules)
- Which files are heavy? (Export rows)

`tokmd` provides this map as a compact, structured dataset.

### 3. Preventing Process Confabulation
In automated workflows, the common failure mode is narrative ("I checked the files"). `tokmd` enforces a "receipt" posture: outputs are deterministic, versioned, and machine-verifiable. Text is untrusted; artifacts are trusted.

### 4. "Shape, not grade"
`tokmd` is explicitly **not** a productivity metric. It is a sensor for inventory, distribution, and drift detection. This aligns with the philosophy of "trusted change" rather than LOC theater.

## What is a Receipt?

A receipt is more than just JSON output. It is a **contract**.

Every `tokmd` output includes:
- `schema_version`: To allow safe evolution.
- `tool` & `args`: Provenance of how the data was generated.
- `scan` configuration: What was ignored/included.
- `totals` & `rows`: The data itself.

This structure allows downstream tools (dashboards, diff engines, agents) to consume the data without guessing.

## Trust Boundaries

- **Text** is untrusted.
- **Artifacts** (receipts) are trusted.

By generating a receipt, you create a boundary object that can be passed between agents or stored as evidence of a repository's state at a specific commit.
