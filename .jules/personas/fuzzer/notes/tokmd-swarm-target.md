# Note: tokmd-swarm target

When building new fuzzer targets that harden core `tokmd` boundaries (like the CLI parser), the development must happen in `EffortlessMetrics/tokmd-swarm` first, then be imported by merge commit into `tokmd`. Do not submit PRs directly to `tokmd` for this type of work.
