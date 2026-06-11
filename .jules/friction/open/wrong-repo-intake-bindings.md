# Wrong-repo intake for bindings-targets

The `web/runner/runtime.js` logic was modified directly in `tokmd`, but normal implementation for this surface lands in `tokmd-swarm` and is imported into `tokmd` by merge commit.

This friction item records that `bindings-targets` or browser runner changes must be submitted to the `tokmd-swarm` repository to avoid "wrong-repo intake" rejections.
