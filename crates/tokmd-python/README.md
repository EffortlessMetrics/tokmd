# tokmd

Python bindings for [tokmd](https://github.com/EffortlessMetrics/tokmd): deterministic repo receipts, analysis, cockpit metrics, and diff workflows from Python.

## Installation

```bash
pip install tokmd
```

## Quick Start

```python
import tokmd

# Language summary
result = tokmd.lang(paths=["src"], top=5)
for row in result["report"]["rows"]:
    print(f"{row['lang']}: {row['code']} lines")

# Effort-focused analysis (1.8.0 preset)
result = tokmd.analyze(
    paths=["."],
    preset="estimate",
    effort_base_ref="main",
    effort_head_ref="HEAD",
)

if result.get("effort"):
    print(result["effort"]["results"]["effort_pm_p50"])
```

## High-Level Functions

The binding exposes Python-native wrappers for:

- `lang(...)`
- `module(...)`
- `export(...)`
- `analyze(...)`
- `cockpit(...)`
- `diff(from_path, to_path)`

These return Python dictionaries extracted from the shared JSON envelope.

## Low-Level API

Use these when you want direct access to the FFI boundary:

- `run_json(mode, args_json)`
- `run(mode, args)`
- `version()`
- `schema_version()`

Supported low-level modes are:

- `lang`
- `module`
- `export`
- `analyze`
- `cockpit`
- `diff`
- `version`

The response envelope is stable:

- success: `{"ok": true, "data": {...}}`
- error: `{"ok": false, "error": {...}}`

## Notes

- Current analysis presets include `estimate`, `risk`, `deep`, and `fun`.
- The binding forwards current effort options such as `effort_base_ref`, `effort_head_ref`, `effort_layer`, `effort_monte_carlo`, `effort_mc_iterations`, and `effort_mc_seed`.
- Long-running scans release the GIL while the Rust core is doing the work.

## Development

Building from source requires Rust and maturin:

```bash
pip install maturin
cd crates/tokmd-python
maturin develop
pytest tests/
```

## License

MIT OR Apache-2.0
