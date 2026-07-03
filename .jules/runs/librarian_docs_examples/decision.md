## Option A / Option B
### Option A
Update the `tokmd gate` examples in `crates/tokmd/src/cli/parser/gate.rs` and `docs/reference-cli.md` to include `--policy` flags. The current examples like `tokmd gate . --preset health` will fail at runtime with `Error: No policy or ratchet rules specified.`

### Option B
Do nothing and emit a learning PR.

### Decision
Option A. It's a clear factual doc drift/example drift within the `tooling-governance` shard, and `docs-executable` requires examples to execute.
