# tokmd-analysis-imports

Single-responsibility microcrate for language-aware import extraction used by
`tokmd-analysis-content`.

## What it does

- Detects whether a language is supported for import extraction.
- Extracts import targets from Rust, JavaScript/TypeScript, Python, and Go.
- Normalizes import targets into deterministic dependency roots.

## API

- `supports_language(lang: &str) -> bool`
- `parse_imports(lang: &str, lines: &[impl AsRef<str>]) -> Vec<String>`
- `normalize_import_target(target: &str) -> String`
