# Explanation: The Philosophy of Receipts

`tokmd` is built on a specific philosophy: **Shift Left the Context**.

## The Problem: "Chatty" Analysis

In traditional software analysis, humans (or agents) run commands, get text output, read it, then run another command.

1. `ls -R` -> "Too much output."
2. `find . -name "*.rs"` -> "Okay, but which are important?"
3. `tokei` -> "Okay, I see totals, but where are they?"

This "chatty" loop is expensive for humans and *very* expensive for LLM agents (burning context window and steps).

## The Solution: Receipts

A **Receipt** is a structured, comprehensive, deterministic artifact that describes the state of a system at a point in time.

Instead of asking 10 questions, an agent should request 1 receipt.

### Characteristics of a tokmd Receipt

1.  **Deterministic**: Running `tokmd export` on the same commit always produces the same JSON sha. This means you can cache it, diff it, and trust it.
2.  **Structured**: It's JSON/JSONL, not unstructured text. Pipelines can filter `rows.filter(r => r.code > 1000)` without regex.
3.  **Complete (Bounded)**: It lists *everything* (that matters). The consumer decides what to filter, not the producer (though `tokmd` offers pre-filtering optimization).

## Why "Shift Left"?

"Shift Left" usually refers to testing. Here, it refers to **Context Generation**.

By generating a high-quality receipt *before* the LLM starts reasoning, you:
- **Save Tokens**: You don't paste `ls -R` output. You paste a compact JSONL inventory.
- **improve Reasoning**: The LLM sees the *shape* of the codebase (file sizes, languages, module distribution) immediately. It knows that `src/legacy/` is huge and `src/new/` is empty without having to guess.

## Why not just use `tokei`?

`tokei` is an amazing tool. `tokmd` wraps it to add the "Receipt" layer:
- **Metadata**: Adding `schema_version`, `scan` args, and timestamps.
- **Pipelines**: Defaults like `jsonl` and `csv` tailored for data ingestion.
- **Safety**: Redaction and consistent sorting.

`tokmd` is the bridge between `tokei`'s raw speed and an agentic pipeline's need for structured stability.
