# Friction Item: Librarian doctest baseline cost

When attempting to write integration tests to verify `docs/reference-cli.md` examples, the use of `unwrap()` and `expect()` caused thousands of lines of cascade renumbering changes in `policy/no-panic-allowlist.toml`.

**Impact:** Pull request was rejected due to "baseline cost dominating the change". The PR scope was also too broad (bundled unrelated `.jules` artifacts and accidentally rebased upstream changes into the PR).

**Recommendation:**
1. Integration tests testing examples should use `Result<(), Box<dyn std::error::Error>>` and the `?` operator instead of `unwrap()`/`expect()` to avoid triggering `no-panic` policy updates.
2. Future feature work should target `EffortlessMetrics/tokmd-swarm` directly rather than the publication repo, as per `docs/ci/swarm-routing.md`.
