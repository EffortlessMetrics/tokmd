# Friction Item: Workflow Collision

## Summary
The intended patch PR for `invariant_model_analysis` (adding `test_density` and `boilerplate` property tests) was superseded by another PR (#1759) that merged equivalent derived ratio property tests on main.

## Description
During the execution of prompt `invariant_model_analysis`, the agent successfully authored and tested the required proptests for bounding `test_density.ratio` and `boilerplate.ratio` to the `[0.0, 1.0]` range. However, a maintainer comment indicated that this branch's work was superseded and dropped in favor of an already-merged pull request (#1759). As a result, the work is being recorded as a learning PR instead of forcing a fake fix.

## Affected Surfaces
- `crates/tokmd-analysis/src/derived/tests/properties.rs`
- PR queue management
