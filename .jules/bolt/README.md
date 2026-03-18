# Bolt ⚡

What Bolt checks in tokmd:
- unnecessary allocations / cloning / string building in hot paths
- repeated parsing/formatting work that can be reused
- avoid O(n²) passes over input where a single pass works
- reduce intermediate buffers (streaming vs collect) if output determinism stays intact
- avoid regex-heavy loops if a simpler scan is correct
- move work out of loops; add capacity hints (`with_capacity`) when justified
- if compile time is the issue: feature gating / cfg hygiene (only if SRP and safe)

Proof expectations:
- benchmark output
- runtime timing
- structural proof
