# Option A (recommended)
Record the out-of-bounds target as a friction item.

This adheres strictly to the prompt's instructions: "If the strongest target you find is outside the shard, record it as friction instead of chasing it."

# Option B
Fix the `scope_count` in `xtask/tests/proof_policy_w90.rs`.

While this fixes the immediate test failure, it violates the shard boundaries and the explicit instruction not to chase out-of-shard targets.

# Decision
I'll proceed with **Option A** to respect shard boundaries and generate a learning PR instead.
