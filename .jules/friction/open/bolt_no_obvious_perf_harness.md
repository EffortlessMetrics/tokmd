# Friction: No Obvious Performance Harness in Analysis Stack

## Surface
`analysis-stack` (`crates/tokmd-analysis*/**`)

## Impact
When tasked as the `Bolt` persona to find and land a meaningful performance improvement, I lacked a clear target. Attempting structural changes without a benchmark harness leads to risky guessing or hallucinated "fixes" that worsen performance.

## Context
I explored `tokmd-analysis-content` and observed `BTreeMap` iterations. I attempted an explicit sort, mistakenly believing it was a Gatekeeper rule to add determinism. The code review correctly identified that this was a hallucinated, fake fix that added O(N log N) overhead, violating the `perf-proof` Gate profile.

## Proposed Resolution
- Provide explicit benchmark targets or cargo-bench instructions in `tokmd-analysis` `README.md` or `AGENTS.md`.
- Enhance the `analysis-stack` shard context with known slow paths or structural inefficiencies.
