#!/bin/bash
# Apply #[allow(unused)] to suppress the test-only helper warnings in crates/tokmd-core/src/lib.rs

# We need to insert #[allow(unused)] before specific structs/functions inside the tests module.
# To keep it simple and safe, we can add a module-level allow at the top of the `mod tests` block.

sed -i 's/mod tests {/#[allow(unused)]\nmod tests {/g' crates/tokmd-core/src/lib.rs
