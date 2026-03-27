# tokmd-cockpit

Compute PR metrics, evidence gates, and review plans from git diffs.

## Problem

Review needs a compact evidence trail, not just changed files.

## What it gives you

- `compute_cockpit` for git-backed PR receipts
- `load_and_compute_trend` for baseline comparisons
- `compute_composition`, `compute_code_health`, `compute_risk`, `generate_review_plan`
- `render_json`, `render_markdown`, `render_comment_md`, `write_artifacts`

## Quick use / integration notes

The default `git` feature lets the crate walk `base..head`, hash file sets, and populate evidence gates before rendering.

Without `git`, the pure render helpers stay available for prebuilt cockpit receipts.

## Go deeper

### How-to

- `../../docs/reference-cli.md`

### Reference

- `src/lib.rs`
- `src/render.rs`

### Explanation

- `../../docs/explanation.md`
