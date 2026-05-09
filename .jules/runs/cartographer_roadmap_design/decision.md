# Decision

## Problem
The document `docs/architecture-consolidation-plan.md` contains factual drift.
The "Current Pressure Points" and "First Suggested PRs" sections are out of date:
- `crates/tokmd-cockpit/src/gates.rs` is no longer ~1196 lines; it is 118 lines. The files were split into the `crates/tokmd-cockpit/src/gates/` directory.
- `crates/tokmd-format/src/analysis/mod.rs` was claimed to be 1815 lines but it is actually 1746 lines, and there's already `markdown.rs`, `html.rs`, `jsonld.rs`, `svg.rs`, `tree.rs`, `xml.rs` in `crates/tokmd-format/src/analysis/`.
- `crates/tokmd-cockpit/src/render/` already exists with `artifacts.rs`, `comment.rs`, `evidence.rs`, `manifest.rs`, `markdown.rs`, `review_map.rs`, `review_packet.rs`, `sections.rs`.
- `crates/tokmd-analysis-types/src/lib.rs` is 1886 lines, slightly more than 1702 lines, and still seems mostly unified (though there is a `findings.rs` and `util/`).
- `crates/tokmd/src/context_pack.rs` is 2195 lines.
- `crates/tokmd-core/src/ffi.rs` is 1647 lines.
- `crates/tokmd-analysis/src/content/complexity.rs` is 2766 lines.

The document should be updated to reflect the true state of the architecture consolidation: `Batch A: Cockpit Owner Modules` is already complete. `Batch B: Format Analysis Rendering` has also seen significant progress (modules were extracted). The line numbers need to be updated.

## Options Considered

### Option A (recommended)
Update `docs/architecture-consolidation-plan.md` to fix the factual drift.
1. Remove completed tasks from the "Current Pressure Points" table and "First Suggested PRs" (like `crates/tokmd-cockpit/src/gates.rs`).
2. Update line counts for the remaining pressure points.
3. Update "Batch Order" to mark Batch A (Cockpit) and parts of Batch B (Format Analysis) as complete/removed from target list since they are mostly extracted.

### Option B
Ignore the specific line numbers and just remove the completed items from "First Suggested PRs".
- This leaves the pressure points table factually incorrect.

## Decision
Option A. It accurately reflects the current state of the repo, fixes drift in both the table and the suggested PRs, and satisfies the Cartographer persona's primary mission.
