# Friction Item: Redundant proptests and tracking artifacts in `invariant` runs

When running the `invariant` persona on `analysis-stack`, the LLM agent often duplicates property tests that already exist elsewhere, such as in `deep_apisurface_w49.rs` vs `properties_w60.rs`. It needs stronger guidance or local greps to avoid reimplementing coverage that's already checked in.

Furthermore, if the initial patch does not remove `.jules/runs` from git track state, CI checks like `xtask gate --check` will fail, causing back-and-forth iteration. We should default to adding `.jules/runs` to `.gitignore` or strictly using `git rm -r --cached .jules/runs/` *before* the first submission.
