## Options considered

### Option A (recommended)
Add WASM-safe time retrieval functions (`now_ms`) directly using conditional compilation to replace standard library's `SystemTime` calls in `crates/tokmd/src/commands` and `crates/tokmd/src/context_pack`.
- **Why it fits:** Fixes `wasm32-unknown-unknown` build incompatibility across config/core/CLI paths cleanly and directly matches the memory instruction about WASM timestamps.
- **Trade-offs:** Minimal footprint, maintains platform correctness for WASM while keeping exact logic intact for other platforms. Structure/Velocity: Adds conditional compilation code blocks directly inside the commands. Governance: Follows standard Rust porting pattern.

### Option B
Lift time-getting mechanisms out into a core `tokmd-wasm` compatibility layer and expose them.
- **Why it fits:** Consolidates conditional logic.
- **Trade-offs:** Overkill when `tokmd-core` already solves this internally and CLI/command files only need minor adjustments to avoid standard `SystemTime`.

## Decision
Option A. It's the simplest and most direct path to fixing `cargo check -p tokmd --target wasm32-unknown-unknown` while adhering to the standard "Builder" profile constraints of not introducing unnecessary new abstractions unless needed. It matches existing practice inside `tokmd-core`.
