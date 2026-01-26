# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-01-25

### Added
- **Formal Receipt Schema**: Introduced a stable JSON output format for `lang`, `module`, and `export` modes.
- **Formal Schema Definition**: Added `docs/schema.json` (JSON Schema Draft 07) to validate outputs.
- **Export Mode**: New `tokmd export` command to generate JSONL/CSV inventories of files.
- **Redaction**: `--redact paths` and `--redact all` flags to sanitize output for LLM usage.
- **Filtering**: `--min-code` and `--max-rows` flags to control output size.
- **Initialization**: `tokmd init` command to generate `.tokeignore` templates.
- **Module Analysis**: Enhanced module reporting with configurable roots (`--module-roots`) and depth (`--module-depth`).
- **Test Harness**: Robust integration suite with BDD-style scenarios and golden snapshots using `insta`.

### Changed
- **CLI**: `tokmd` (default) now produces a Markdown table by default (previously text).
- **Semantics**: `--children` flag logic unified across all modes.
- **Docs**: Completely overhauled documentation structure following Diataxis principles (Tutorials, How-to, Reference, Explanation).

### Fixed
- **Ignore Logic**: Corrected behavior where `--no-ignore` did not consistently disable all ignore types.
- **Stability**: Fixed deterministic sorting of output rows.

## [0.1.0] - 2026-01-01
- Initial prototype release.
