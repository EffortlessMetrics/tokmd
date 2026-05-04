# Decision

## Option A: Consolidate `cargo fuzz` friction into Fuzzer persona notes
- **What it is:** Move `FRIC-20260413-001.md`, `FRIC-20260428-001.md`, and `cargo_fuzz_asan_linker_failure.md` to `.jules/friction/done/` and create a consolidated learning in `.jules/personas/fuzzer/notes/sandbox_limitations.md`.
- **Why it fits:** Directly answers the primary target of the Archivist persona (consolidating recurring friction themes). It provides future Fuzzer runs with a clear, single source of truth regarding sandbox limitations.
- **Trade-offs:**
  - *Structure:* Cleaner friction queue, better documentation for Fuzzer.
  - *Velocity:* High, easy to do.
  - *Governance:* Aligns with `.jules/` storage rules.

## Option B: Summarize per-run packets into an index
- **What it is:** Parse `.jules/runs/` and generate a markdown index of all runs.
- **When to choose it instead:** If there is no recurring friction to consolidate.
- **Trade-offs:**
  - Less impactful than reducing friction noise. Indexes can become outdated quickly unless automated.

## Decision
**Option A**. There are three separate open friction items all describing the same core issue: `cargo fuzz` does not work in the standard execution environment due to missing nightly/ASAN support or Windows/MSVC toolchain issues. Consolidating these into a Fuzzer persona note will prevent future fuzzer runs from repeatedly hitting and recording the exact same friction, and cleans up the open friction queue.
