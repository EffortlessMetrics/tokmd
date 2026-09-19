## 💡 Summary
Replaced repeated string allocations with stack-allocated byte buffers and slices in Halstead tokenization punctuation parsing.

## 🎯 Why
`tokenize_for_halstead` processes code per-line and per-character. The original implementation called `chars.clone().take(4).collect::<String>()` every time it encountered an ascii punctuation character. This led to thousands of small, short-lived heap allocations, creating unnecessary pressure and slowing down analysis metrics.

## 🔎 Evidence
File: `crates/tokmd-analysis/src/halstead/tokenizer.rs`
Finding: `let remaining: String = chars.clone().take(4).collect();` was present inside the main parsing loop for punctuation logic.

## 🧭 Options considered
### Option A (recommended)
- Replace `String` collection with a small stack-allocated `[u8; 16]` buffer, filling it with characters and generating an `&str` slice from the valid bytes.
- This fits the repo and shard because it directly avoids allocations and `String` copying on a hot path within the `analysis-stack`.
- Structure trade-offs: Increases the length of the matching block slightly but completely removes heap allocations.
- Velocity / Governance: Low impact to other services, safe isolated change.

### Option B
- Refactor the whole parsing loop using `regex` or a similar dedicated tokenizer engine.
- Choose this if parsing performance needs to scale significantly.
- Trade-offs: Increases dependencies, overhead of compiling regexes could cancel out wins for small files.

## ✅ Decision
Option A was chosen as it delivers immediate performance benefits with minimal changes to the existing structural logic, while meeting the performance proof profile.

## 🧱 Changes made (SRP)
- Modified `crates/tokmd-analysis/src/halstead/tokenizer.rs`
  - Replaced `String` allocation in punctuation handling with `[u8; 16]` buffer and `.encode_utf8()`.

## 🧪 Verification receipts
```text
cargo build --verbose
bash -c 'CI=true cargo test -p tokmd-analysis --verbose'
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Optimization
- Blast radius: Low (confined to Halstead metrics tokenizer)
- Risk class: Low
- Rollback: Revert `crates/tokmd-analysis/src/halstead/tokenizer.rs`
- Gates run: `cargo test`, `cargo fmt`, `cargo clippy`, `cargo xtask version-consistency`, `cargo xtask docs --check`

## 🗂️ .jules artifacts
- `.jules/runs/bolt_analysis_stack_builder/envelope.json`
- `.jules/runs/bolt_analysis_stack_builder/decision.md`
- `.jules/runs/bolt_analysis_stack_builder/receipts.jsonl`
- `.jules/runs/bolt_analysis_stack_builder/result.json`
- `.jules/runs/bolt_analysis_stack_builder/pr_body.md`

## 🔜 Follow-ups
- Consider optimizing `build_topic_clouds` to avoid `String` allocations.
