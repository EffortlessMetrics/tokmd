# Wrong Repo Intake

## Summary
The work executed in `palette_binding_dx_01` was performed directly in `EffortlessMetrics/tokmd`, but the PR was rejected as "wrong-repo intake". According to the shared-history topology (and `docs/ci/swarm-routing.md`), normal development work must start in `EffortlessMetrics/tokmd-swarm`.

## Detail
The `.jules/README.md` explicitly warns: "Normal tokmd development starts from `EffortlessMetrics/tokmd-swarm:main`. Create narrow PRs there, wait for `Tokmd Rust Small Result`, and squash-merge aligned work into the swarm repo." The `tokmd` repository is strictly for publication (merge-commit PRs only).

The prompt instructed me to act as a "one-shot branch author on a fresh clone of EffortlessMetrics/tokmd" and to "finish with a PR-worthy diff", which directly conflicts with the dual-repo topology rules if I attempt to merge it here.

## Recommendation
Future runs assigned to `EffortlessMetrics/tokmd` should immediately short-circuit to a learning PR that identifies the dual-repo topology mismatch, rather than attempting to land functional patches directly in the publication repository. Alternatively, the harness must run the agent against `tokmd-swarm` instead.
