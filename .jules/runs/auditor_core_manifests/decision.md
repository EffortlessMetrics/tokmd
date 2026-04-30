## Option A
Remove unused `midly` feature/dependency from `tokmd-format`
- *Why it fits*: The `cargo tree` output previously showed `tokmd-format` having a `fun` feature depending on `midly`. But `tokmd-format` actually has multiple files (e.g. `src/fun/mod.rs`, `tests/fun_bdd.rs`, etc) that use `midly` so it's not unused.

## Option B
Find an unused or redundant dependency using `cargo machete` or manual review. `cargo machete` found nothing in the target crates `tokmd-types`, `tokmd-scan`, `tokmd-model`, `tokmd-format`.

## Option C
Tighten `serde` features. `tokmd-types` currently depends on `serde` with `features = ["derive"]` (which is often redundant or can be tightened). But it seems we don't have any obvious unneeded dependency in the core-pipeline shard. Let's record a learning PR as instructed.

## Option D
Wait, I see `tokmd-scan` depends on `anyhow`, `ignore`, `tempfile`, `tokei`, `tokmd-io-port`, `tokmd-settings`, `tokmd-types`. But `tokmd-types` depends on `serde` and `clap` (optional). The `clap` feature is enabled by `tokmd` but it might be cleanly removed if we move the CLI-specific `clap` derives out of `tokmd-types`. But changing contract surface is risky.

Let's double check `tokmd-format` `midly` dependency. It's optional.

Let's double check `tokmd-model`. It depends on `serde`, `tokei`, `tokmd-types`. `tempfile` is a dev-dependency.

Wait, `cargo machete` on `fuzz` found `anyhow`, `blake3`, `tempfile` unused. But `fuzz` is not in the primary shard, though the instruction says "If the strongest target you find is outside the shard, record it as friction instead of chasing it."

I will create a learning PR as I couldn't find a safe, unused direct dependency to remove in the primary shard.

## Decision
I choose **Option B** (Learning PR). I've thoroughly investigated `tokmd-types`, `tokmd-scan`, `tokmd-model`, and `tokmd-format` for unused dependencies using `cargo machete` and `cargo tree`. `cargo machete` reported "didn't find any unused dependencies" for all four crates. While `cargo machete` found unused dependencies in `tokmd-fuzz` and `tokmd-node` (outside the primary shard), the Auditor instructions explicitly state to "record it as friction instead of chasing it" if the strongest target is outside the shard.
