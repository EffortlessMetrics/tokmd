# Decision

## Option A (Recommended)
Fix the `clean_path` function in `crates/tokmd-format/src/redact/mod.rs` to normalize consecutive slashes (e.g. `//`) into a single slash (`/`).
- **Why it fits this repo and shard**: The core-pipeline shard expects deterministic handling of redaction boundaries and receipts. Unnormalized double slashes currently cause paths that represent the exact same file (e.g. `src/lib.rs` and `src//lib.rs`) to yield differing `short_hash` outputs. Normalizing consecutive slashes ensures deterministic receipt construction without information leakage.
- **Trade-offs**:
  - *Structure*: Straightforward addition to the path cleaning utility function `clean_path`.
  - *Velocity*: Minimal code change and immediate security/consistency benefit.
  - *Governance*: Prevents future hashing differences due to varying OS or API path separators.

## Option B
Do nothing.
- **When to choose it instead**: If double slash occurrences were guaranteed to be entirely stripped by outer caller APIs (which is not guaranteed).
- **Trade-offs**: Results in inconsistent short hashes across paths that point to the same filesystem target.

## Decision
Chose Option A. Normalizing consecutive slashes correctly aligns with the existing logic of stripping out `.` and `./` segments for uniform hashing outcomes. Tests were implemented and run to guarantee no regressions occurred in existing path manipulations.