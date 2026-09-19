# Decision

## Option A (recommended)
Optimize `tokenize_for_halstead` in `crates/tokmd-analysis/src/halstead/tokenizer.rs`.
- Currently, it heavily allocates and clones strings using `chars.clone().take(4).collect::<String>()` for punctuation checks on every single matching punctuation character.
- By using a fixed size stack buffer `[u8; 16]` and parsing characters onto it, we can eliminate this allocation and instead use slices.
- This represents a small, direct hot-path performance win on parsing, well aligned with Bolt's persona.

## Option B
Optimize `build_topic_clouds` in `crates/tokmd-analysis/src/topics/mod.rs`.
- Replace `BTreeMap<String, ...>` with `rustc_hash::FxHashMap<&str, ...>` to eliminate allocations entirely and use faster hashmaps.
- Requires significantly more churn dealing with borrowing and lifetimes.
- High risk of messing up edge cases compared to Option A.

## Decision
Choose Option A. It's much easier to verify and doesn't introduce massive diffs.
