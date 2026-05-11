# Decision

## Investigation
I investigated the `tokmd-analysis` crate, looking for hot-path work reduction or unnecessary allocations. I checked the `clone()` usage across the codebase and specifically investigated the `build_topic_clouds` in `crates/tokmd-analysis/src/topics/mod.rs` and the group ratio calculation in `crates/tokmd-analysis/src/derived/ratios.rs`.

In `crates/tokmd-analysis/src/topics/mod.rs`, I noticed `*overall_tf.entry(term.clone()).or_insert(0) += *tf;` where `term` is cloned unconditionally even if it already exists in the map. This can be optimized by using a conditional clone:
```rust
if let Some(val) = overall_tf.get_mut(term) {
    *val += *tf;
} else {
    overall_tf.insert(term.clone(), *tf);
}
```
Or similar constructs to avoid unnecessary cloning. Benchmark proved that avoiding the unconditional clone is faster (107ms vs 72ms on 100k insertions).

I also noticed that `tokenize_path` does unnecessary `String` allocations by converting parts to lowercase before filtering stopwords, rather than maybe using `&str`. But keeping it simple, the unconditional `clone()` in `build_topic_clouds` is a great target.

There is also a similar `clone()` issue in `crates/tokmd-analysis/src/derived/files.rs` where `build_max_file_report` creates a `MaxFileRow` which clones the `FileStatRow` for each language and module. Although there's no loop unconditional clone here, it's just part of report building.

Another interesting area is `crates/tokmd-analysis/src/derived/ratios.rs`. The `group_ratio` and `group_rate` functions do string allocation `map.into_iter().map(|(k, v)| (k.to_string(), v)).collect()` after grouping by a generic key fn. We could just use `&str` keys but the return type needs to own the string.

Let's focus on the `build_topic_clouds` unconditional `clone()` and `overall_tf.entry(term.clone())` as the primary optimization.
Wait, let's look at `crates/tokmd-analysis/src/git/mod.rs`:
```rust
47:                    commit_counts.insert(key.clone(), 1);
```
But `key` here is `&String`. This is fine.

Looking at `crates/tokmd-analysis/src/topics/mod.rs` again:
```rust
        for term in terms {
            match df_map.get_mut(&term) {
                Some(count) => *count += 1,
                None => {
                    df_map.insert(term, 1);
                }
            }
        }
```
This avoids cloning because `terms` yields owned `String`s.
However, in `build_topic_clouds`:
```rust
        for term in &terms {
            match module_terms.get_mut(term) {
                Some(count) => *count += weight,
                None => {
                    module_terms.insert(term.clone(), weight);
                }
            }
        }
```
This correctly avoids cloning if the term is already present!
But later down:
```rust
        for (term, tf) in tf_map {
            *overall_tf.entry(term.clone()).or_insert(0) += *tf;
        }
```
This unconditionally clones `term` every time it processes `tf_map` for any module. Since `tf_map` is per-module, the same term will be seen multiple times across different modules, and it will be cloned unconditionally each time! This is a classic unnecessary allocation on a hot path if we have many modules.
We can rewrite this as:
```rust
        for (term, tf) in tf_map {
            if let Some(val) = overall_tf.get_mut(term) {
                *val += *tf;
            } else {
                overall_tf.insert(term.clone(), *tf);
            }
        }
```

Is there anywhere else with unconditional cloning?
In `crates/tokmd-gate/src/evaluate.rs`:
```rust
crates/tokmd-gate/src/evaluate.rs:77:            message: if passed { None } else { rule.message.clone() },
```
This is only cloned on failure.

Let's check `crates/tokmd-analysis/src/git/mod.rs`:
```rust
        let modules: Vec<&str> = modules.into_iter().collect();
        for i in 0..modules.len() {
            let left = modules[i];
            for right in modules.iter().skip(i + 1) {
                let key = (left, *right);
                *pairs.entry(key).or_insert(0) += 1;
            }
        }
```
This is `(&str, &str)` so it's cheap to copy. No allocation.

What about `crates/tokmd-analysis/src/git/mod.rs` again:
```rust
        for m in &modules {
            if let Some(val) = touches.get_mut(m) {
                *val += 1;
            } else {
                touches.insert(*m, 1);
            }
        }
```
This does conditional insert without clone!

So the `overall_tf.entry(term.clone())` is a great target. Let's see if there are other similar issues.
In `crates/tokmd-analysis/src/topics/mod.rs`, `tokenize_path` does:
```rust
fn tokenize_path(path: &str, stopwords: &BTreeSet<String>) -> Vec<String> {
    let mut out = Vec::new();
    for part in path.replace('\\', "/").split('/') {
        if part.is_empty() {
            continue;
        }
        for token in part.split(['_', '-', '.']).filter(|t| !t.is_empty()) {
            let term = token.to_lowercase();
            if stopwords.contains(&term) {
                continue;
            }
            out.push(term);
        }
    }
    out
}
```
`path.replace('\\', "/")` allocates a new `String` for every file path.
Instead we can just split on both `\` and `/` directly without replacing.
```rust
fn tokenize_path(path: &str, stopwords: &BTreeSet<String>) -> Vec<String> {
    let mut out = Vec::new();
    for part in path.split(|c| c == '/' || c == '\\') {
        if part.is_empty() {
            continue;
        }
        for token in part.split(['_', '-', '.']).filter(|t| !t.is_empty()) {
            let term = token.to_lowercase();
            if stopwords.contains(&term) {
                continue;
            }
            out.push(term);
        }
    }
    out
}
```
This removes a string allocation for every file path parsed!

Even better, `term = token.to_lowercase()` allocates a string. `stopwords` contains `String`s. This is fine since we return `Vec<String>`. But wait, if we drop the stopwords filter and filter later? No, this is fine.

Let's combine these:
1. Avoid `path.replace` allocation in `tokenize_path` by using `split(|c| c == '/' || c == '\\')`.
2. Avoid unconditional `term.clone()` in `build_topic_clouds` when accumulating `overall_tf`.

Option A: Fix both the `path.replace` string allocation and the unconditional `.entry(term.clone())` allocation in `crates/tokmd-analysis/src/topics/mod.rs`. This reduces allocations per file and per module term, respectively.

Option B: Rewrite `topics` to use zero-allocation string slices throughout, only allocating strings at the very end when constructing the DTOs.

Decision: Option A is safer, structurally localized, and provides clear hot-path work reduction (unnecessary allocations) without requiring a full refactor of the module and risking correctness/determinism issues. It perfectly aligns with Bolt's "unnecessary allocations / string building" target ranking.
