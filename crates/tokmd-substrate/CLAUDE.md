# tokmd-substrate

## Purpose

Shared repo context for cross-sensor coordination. This is a **Tier 0** pure data crate.

## Responsibility

- Define `RepoSubstrate` (shared scan context for all sensors)
- Define `SubstrateFile` (per-file scan data)
- Define `LangSummary` (per-language aggregates)
- Define `DiffRange` (git diff context)
- Provide convenience iterators (`diff_files`, `files_for_lang`)
- **NOT** for substrate building (see `tokmd-sensor::substrate_builder`)
- **NOT** for business logic or analysis

## Public API

```rust
pub struct RepoSubstrate {
    pub repo_root: String,
    pub files: Vec<SubstrateFile>,
    pub lang_summary: BTreeMap<String, LangSummary>,
    pub diff_range: Option<DiffRange>,
    pub total_tokens: usize,
    pub total_bytes: usize,
    pub total_code_lines: usize,
}

pub struct SubstrateFile {
    pub path: String,       // repo-relative, forward slashes
    pub lang: String,
    pub code: usize,
    pub lines: usize,
    pub bytes: usize,
    pub tokens: usize,
    pub module: String,
    pub in_diff: bool,
}

pub struct LangSummary { pub files, code, lines, bytes, tokens: usize }
pub struct DiffRange { pub base, head: String, pub changed_files: Vec<String>, pub commit_count, insertions, deletions: usize }

impl RepoSubstrate {
    pub fn diff_files(&self) -> impl Iterator<Item = &SubstrateFile>;
    pub fn files_for_lang(&self, lang: &str) -> impl Iterator<Item = &SubstrateFile>;
}
```

## Implementation Details

- Uses `BTreeMap` for `lang_summary` to ensure deterministic key ordering
- All paths are repo-relative with forward slashes
- `in_diff` flag marks files changed in the current diff range
- No business logic -- pure data types with serde derive

## Dependencies

- `serde` (serialization)

## Testing

```bash
cargo test -p tokmd-substrate
```

Tests cover:
- Serde roundtrip for full substrate
- `diff_files` filter correctness
- `files_for_lang` filter correctness
- BTreeMap ordering guarantee

## Do NOT

- Add I/O operations (substrate building is in tokmd-sensor)
- Add business logic or analysis
- Use `HashMap` (use `BTreeMap` for determinism)
- Add dependencies beyond serde (this must stay minimal)
