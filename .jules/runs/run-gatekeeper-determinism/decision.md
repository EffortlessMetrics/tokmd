# Decision

## Option A (recommended)
Replace `localeCompare` with strict Unicode code unit comparison (`a < b ? -1 : a > b ? 1 : 0`) in the HTML report table sorting.
- Why it fits: Matches Rust's native `String::cmp` lexicographical ordering, eliminates platform-dependent determinism drift, and aligns with the memory guideline for JS/Node environments.
- Trade-offs: Structure is simplified, velocity is high, governance aligns with strict contract determinism requirements.

## Option B
Keep `localeCompare` but explicitly pass a fixed locale like `'en-US'` with strict collator options.
- When to choose: If human-linguistic sorting is preferred over exact deterministic structural sorting.
- Trade-offs: May still suffer from subtle platform engine (V8/SpiderMonkey) implementation differences, violating strict determinism gates.

## Decision
Choosing **Option A** because the Gatekeeper persona mandates locking in deterministic behavior. `localeCompare` is explicitly known to cause determinism drift in JS environments across platforms. Replacing it with strict `<`/`>` comparisons ensures exact lexicographical sorting matching Rust's `BTreeMap` and `String::cmp` behavior, and aligns with a prior PR fix (#1551) mentioned in `PR_DRAIN.md`.
