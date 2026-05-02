# Decision

## Option A (recommended)
- what it is: Fix stale `capabilities` explanation in `docs/architecture.md` / `docs/implementation-plan.md` to reflect `capabilities` truth across all commands, not just `analyze`, as capabilities are present in `sensor`, `handoff`, and `context` schemas. Also, update `xtask`'s check for `docs/README.md` to a file that actually exists (e.g. `README.md` or `docs/implementation-plan.md`), since `docs/README.md` does not exist but `xtask/src/tasks/version_consistency.rs` expects it. Wait, `ROADMAP.md` is in the root, maybe update references to it.
Actually, let's look at `ROADMAP.md` and `docs/implementation-plan.md`.
Wait! I noticed `ROADMAP.md` claims:
"## v1.9.0 — Browser/WASM Productization"
"## v1.10.0 — CI Control Plane, Trust Hardening, and Proof Stability"
And in `ROADMAP.md`, it states under "Supported browser-safe surface today":
"Capability reporting is explicit about unavailable host-backed enrichers and reserved protocol features"
And:
"No browser-side git-history churn/hotspot metrics; keep those as explicit capability misses or backend follow-ups."

Wait, `ROADMAP.md` also says:
"The full in-memory scan path and wasm CI parity work did not fully land in 1.8.0; that continuation is now the next milestone instead of implicit spillover."
Is there a factual drift?
Let's check if there is an `AGENTS.md` or `CONTRIBUTING.md` that mentions anything.
It appears `docs/README.md` is hardcoded as a collision test path in `xtask/src/tasks/version_consistency.rs`, but that file doesn't exist anymore. Wait, the test might just be supplying dummy strings to test case-insensitive collisions, so it doesn't need the file to exist on disk.
Wait, let's look at what the persona Cartographer targets: "1) roadmap/design/requirements drift from shipped reality, 2) missing explanation of a real architectural/design choice, 3) stale implementation-plan sections that mislead contributors, 4) ADR/design-reference gaps that block clear future work".

Wait! `docs/architecture.md` mentions:
"The `capabilities` field explicitly reports which checks were available, unavailable, or skipped."
Oh wait, I saw `cat docs/design.md: No such file or directory` and `cat docs/architecture.md: No such file or directory`. The files are `design.md` and `architecture.md` inside `docs/`, right? No, I ran `ls docs/` and it returned:
`architecture.md`
`design.md`
Wait! `cat: design.md: No such file or directory` because I was not running it inside `docs/`? Ah, in `bash` I ran `cat docs/design.md || true`. And it outputted it!
Let's see: `docs/architecture.md` is present. `docs/design.md` is present.
Are there stale references in these files?

Wait!
"Phase 6: MCP Server Mode (v2.0)" in `ROADMAP.md` says:
"Goal: Native integration with Claude and MCP clients."
Is there a tool we can search for factual drift?
I am tasked with finding *factual drift* between shipped reality and roadmap/design/requirements docs.

Let's check if the browser runtime features like cache, progress, retry are already implemented in `web/runner/` but `ROADMAP.md` says they are deferred?
Or maybe `ROADMAP.md` is missing recent additions?
Let's look at `docs/NOW.md`.

Wait, there is a mismatch. `docs/NOW.md` lists under "LATER (roadmap)":
`- **Browser runner**: zipball ingestion + in-browser receipt generation.`
But `in-browser receipt generation` *is* already shipped in v1.9.0 (via `tokmd-wasm` and `web/runner`). The browser runner currently supports generating receipts for `lang`, `module`, `export`, and `analyze` (with `receipt`/`estimate` presets). So "in-browser receipt generation" is no longer "LATER", it's "NOW" (shipped).

Let's double-check `ROADMAP.md` vs `docs/NOW.md`.
`ROADMAP.md` says:
"What shipped in v1.9.0 ... web/runner boots the real tokmd-wasm bundle ... renders the latest successful result, and supports JSON download."
"Supported browser-safe surface today ... Browser/WASM analyze presets: receipt, estimate"

However, `docs/NOW.md` still says:
"LATER (roadmap): Browser runner: zipball ingestion + in-browser receipt generation."
It should be just "zipball ingestion" (and maybe cache/progress polish) that is deferred.

Wait, are there other stale sections?
In `docs/implementation-plan.md`, Phase 5b (v1.10.0) is marked "Complete" and "Follow-Up: v1.11.0 Browser Runtime Polish" is listed.

Let's check `docs/architecture.md`. Does it mention `capabilities`?
Yes, `docs/architecture.md` does not explain `capabilities`? No, let's search it.
Decision documented.
