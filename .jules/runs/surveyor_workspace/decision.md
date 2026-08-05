## Options Considered

### Option A: Remove `ast` from default features in `tokmd` crate
**What it is:** In `crates/tokmd/Cargo.toml`, remove the `ast` feature from the `default` list.
**Why it fits this repo and shard:** The memory mentions: "In the `tokmd` project, the `ast` feature (which pulls in `tree-sitter` and its parsers) requires a C standard library (`stdlib.h`) and breaks standard `wasm32-unknown-unknown` builds. It should not be included in `default` features for crates intended to be WASM compatible." The `tokmd` crate is intended to be a CLI tool, but also serves as the top-level aggregation, and including `ast` by default breaks compiling the crate (or workspace tools that depend on it) when testing WASM compatibility via targets. Removing it from `default` ensures `ast` is strictly opt-in for users needing syntax parsing while ensuring compatibility across environments, aligning with the Surveyor persona's mandate on "feature-boundary hygiene" and "dependency direction / workspace structure problems."
**Trade-offs:**
- *Structure*: Improves feature hygiene and target compatibility.
- *Velocity*: `wasm32` builds succeed out of the box without requiring `--no-default-features`.
- *Governance*: Prevents accidental dependencies on C libs in the default path.

### Option B: Keep `ast` in default features, create a learning PR
**What it is:** Acknowledge the WASM build failure caused by `tree-sitter` in the `tokmd` crate but leave it. Record it as friction.
**When to choose it instead:** If removing `ast` from `default` breaks too many user workflows. However, `ast` appears to be a heavy, niche feature (syntax analysis using tree-sitter) that shouldn't be pulled in by every default install, especially when memory strictly states it "should not be included in `default` features for crates intended to be WASM compatible."
**Trade-offs:** Avoids breaking current users expecting `ast` by default, but fails to fix a known compatibility issue.

## Decision
**Option A**. Removing `ast` from the `default` features of the `tokmd` crate fixes a concrete target compatibility issue (WASM builds fail with missing `stdlib.h`), directly addressing the memory requirement and the "feature-boundary hygiene" mandate of the Surveyor persona.
