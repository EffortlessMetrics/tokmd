# Friction Item

**Date:** 2026-05-06
**Persona:** Specsmith
**Context:** Tightening stderr assertions in `cli_error_paths_w51.rs`, `cli_errors_w66.rs`, and `error_handling_w70.rs`.

**Issue:**
The intended patch was superseded by PR #1654. The maintainer noted that #1654 kept the current CLI stderr assertion tightening without deleting `plan.md` or rewriting existing `.jules` run provenance.

**Impact:**
Wasted effort on a redundant fix and a workflow collision.

**Recommendation:**
Always double-check open PRs or maintainer comments for overlapping work before investing significant time in a patch, and be cautious about modifying files outside the immediate scope (like `plan.md`).
