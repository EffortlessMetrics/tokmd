## 💡 Summary
Remove `ast` from the `default` features of the `tokmd` crate to fix standard `wasm32-unknown-unknown` builds.

## 🎯 Why
The memory explicitly notes: "In the `tokmd` project, the `ast` feature (which pulls in `tree-sitter` and its parsers) requires a C standard library (`stdlib.h`) and breaks standard `wasm32-unknown-unknown` builds. It should not be included in `default` features for crates intended to be WASM compatible." By including it in `default`, compiling the `tokmd` crate or testing it across targets inherently fails on environments missing C standard libraries, causing friction and feature-boundary hygiene issues.

## 🔎 Evidence
```text
$ cargo check -p tokmd --target wasm32-unknown-unknown
warning: tree-sitter-python@0.25.0: src/tree_sitter/parser.h:10:10: fatal error: 'stdlib.h' file not found
warning: tree-sitter-typescript@0.23.2: ./typescript/src/tree_sitter/parser.h:10:10: fatal error: 'stdlib.h' file not found
error: failed to run custom build command for `tree-sitter-python v0.25.0`
```

## 🧭 Options considered
### Option A (recommended)
- Remove `ast` from `default` features in `crates/tokmd/Cargo.toml`.
- Why it fits: Aligns precisely with the prompt's `core-rust` constraints and the Surveyor mandate to fix feature-boundary hygiene.
- Trade-offs: `ast` must now be explicitly requested using `--features ast`, which slightly increases friction for niche users but guarantees compatibility for the rest of the ecosystem.

### Option B
- Keep `ast` in default features, create a learning PR documenting the build failure as friction.
- When to choose it: If removing `ast` would break too many downstream dependencies that expect syntax analysis out of the box.
- Trade-offs: Avoids breaking backwards compatibility but leaves the codebase structurally flawed for cross-platform builds.

## ✅ Decision
Option A was chosen to fix the concrete boundary issue and guarantee successful `wasm32-unknown-unknown` builds by default, adhering to memory constraints and the Surveyor persona's mandate.

## 🧱 Changes made (SRP)
- Modified `crates/tokmd/Cargo.toml` to remove `"ast"` from the `default` feature list.

## 🧪 Verification receipts
```text
$ cargo check -p tokmd --target wasm32-unknown-unknown
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.47s

$ CI=true cargo test --verbose -p tokmd
    Finished `test` profile [unoptimized + debuginfo] target(s)
     Running `/app/target/debug/deps/tok_integration-0dd92f42ba1f312f`
     Running `/app/target/debug/deps/tools_integration-dca1d6635dab696b`
   Doc-tests tokmd
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

$ cargo clippy -- -D warnings -p tokmd
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 27.42s
```

## 🧭 Telemetry
- Change shape: Removal of a single feature string from an array.
- Blast radius: Compilation flags, CLI defaults for feature usage, compatibility.
- Risk class: Low - `ast` parsing logic is fully preserved, just no longer enabled implicitly by default for CLI users.
- Rollback: Revert the commit.
- Gates run: `cargo build --verbose`, `CI=true cargo test --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`.

## 🗂️ .jules artifacts
- `.jules/runs/surveyor_workspace/envelope.json`
- `.jules/runs/surveyor_workspace/decision.md`
- `.jules/runs/surveyor_workspace/receipts.jsonl`
- `.jules/runs/surveyor_workspace/result.json`
- `.jules/runs/surveyor_workspace/pr_body.md`

## 🔜 Follow-ups
None.
