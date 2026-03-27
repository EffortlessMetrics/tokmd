# tokmd

Python bindings for tokmd.

## Problem

Use tokmd from Python without rebuilding the Rust workflow layer yourself.

## What it gives you

- High-level helpers: `lang`, `module`, `export`, `analyze`, `cockpit`, and `diff`
- Low-level access: `run`, `run_json`, `version`, and `schema_version`
- Python dict results extracted from the shared JSON envelope

## Quick use / integration notes

```python
import tokmd

receipt = tokmd.lang(paths=["src"], top=5)
analysis = tokmd.analyze(paths=["."], preset="estimate")
```

`run_json` is the low-level boundary. The higher-level helpers return Python dicts.

Long scans release the GIL while Rust is doing the work.

## Go deeper

### Tutorial

- `../../docs/tutorial.md`

### How-to

- `../../docs/reference-cli.md`

### Reference

- `src/lib.rs`
- `pyproject.toml`
