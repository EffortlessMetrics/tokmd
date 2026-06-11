# Surveyor Note: Repo Topology Constraints

When attempting to improve architecture and structural coherence, be aware that `EffortlessMetrics/tokmd` may not be the correct intake repository for normal implementation changes. Refactors should typically be implemented as a narrow PR in `EffortlessMetrics/tokmd-swarm` and imported into `tokmd` via merge commit.
