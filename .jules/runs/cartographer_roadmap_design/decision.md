## Options Considered

### Option A: Align docs/architecture.md with reality
- What it is: Update `docs/architecture.md` to correctly list `tokmd-sensor`, `tokmd-substrate`, `tokmd-envelope`, and `tokmd-analysis-format-md` in the appropriate Tier tables, reflecting the current codebase architecture.
- Why it fits: The roadmap explicitly states the "tokmd-sensor" and "sensor integration" logic has shipped, and the files exist, but `docs/architecture.md` does not fully list them in the dependency tier lists (only in a diagram/Flow). `tokmd-analysis-format-md` is also missing from the Tier 2 or 3 list entirely.
- Trade-offs:
  - Structure: Improves structural correctness of docs.
  - Velocity: Low risk, high value for new contributors finding where crates belong.
  - Governance: Direct fix to documentation drift.

### Option B: Fix roadmap/design drift around WASM/Browser constraints
- What it is: Update `ROADMAP.md` and `docs/architecture.md` about the current state of browser constraints.
- Why it fits: The browser-safe surface section has some overlap with current constraints.
- Trade-offs: Might be less concrete than missing crates in the core architecture tier list.

## Decision
Option A. The `docs/architecture.md` file has clear tier tables, but newly created crates like `tokmd-sensor`, `tokmd-substrate`, `tokmd-envelope`, `tokmd-settings`, `tokmd-io-port`, and `tokmd-analysis-format-md` are missing from these tables or listed incorrectly. Let's fix this obvious factual drift.
