## Issue
Changes intended for the `tokmd-swarm` topology were incorrectly applied directly to the `tokmd` repository.

## Impact
The Pull Request was closed as `wrong-repo` intake. The `tokmd` repo should receive these changes via a merge commit from `tokmd-swarm`.

## Action Required
If this fix is useful, it needs to be ported as a narrow PR against `EffortlessMetrics/tokmd-swarm` with focused proof, rather than applied directly here.
