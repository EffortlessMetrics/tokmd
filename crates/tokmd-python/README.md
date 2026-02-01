# tokmd

Python bindings for [tokmd](https://github.com/EffortlessMetrics/tokmd) - fast code inventory receipts and analytics.

## Installation

```bash
pip install tokmd
```

## Quick Start

```python
import tokmd

# Get language summary
result = tokmd.lang(paths=["src"])
for row in result["rows"]:
    print(f"{row['lang']}: {row['code']} lines")

# Get module breakdown
result = tokmd.module(paths=["."])
for row in result["rows"]:
    print(f"{row['module']}: {row['code']} lines")

# Run analysis
result = tokmd.analyze(paths=["."], preset="health")
if result.get("derived"):
    totals = result["derived"]["totals"]
    print(f"Total: {totals['code']} lines in {totals['files']} files")
```

## API Reference

### Functions

#### `lang(paths=None, top=0, files=False, children=None, redact=None, excluded=None, hidden=False)`

Scan paths and return a language summary.

- `paths`: List of paths to scan (default: `["."]`)
- `top`: Show only top N languages (0 = all)
- `files`: Include file counts
- `children`: How to handle embedded languages (`"collapse"` or `"separate"`)
- `redact`: Redaction mode (`"none"`, `"paths"`, `"all"`)
- `excluded`: List of glob patterns to exclude
- `hidden`: Include hidden files

#### `module(paths=None, top=0, module_roots=None, module_depth=2, children=None, redact=None, excluded=None, hidden=False)`

Scan paths and return a module summary.

- `paths`: List of paths to scan (default: `["."]`)
- `top`: Show only top N modules (0 = all)
- `module_roots`: Top-level directories as module roots (default: `["crates", "packages"]`)
- `module_depth`: Path segments to include for module roots
- `children`: How to handle embedded languages (`"separate"` or `"parents-only"`)

#### `export(paths=None, format=None, min_code=0, max_rows=0, ...)`

Scan paths and return file-level export data.

- `format`: Output format (`"jsonl"`, `"json"`, `"csv"`, `"cyclonedx"`)
- `min_code`: Minimum lines of code to include
- `max_rows`: Maximum rows to return (0 = unlimited)

#### `analyze(paths=None, preset=None, window=None, git=None, ...)`

Run analysis on paths and return derived metrics.

- `preset`: Analysis preset (`"receipt"`, `"health"`, `"risk"`, `"supply"`, `"architecture"`, `"topics"`, `"security"`, `"identity"`, `"git"`, `"deep"`, `"fun"`)
- `window`: Context window size in tokens
- `git`: Force enable/disable git metrics

#### `diff(from_path, to_path)`

Compare two receipts or paths and return a diff.

### Low-Level API

#### `run_json(mode, args_json)`

Run any tokmd operation with JSON string arguments.

```python
result = tokmd.run_json("lang", '{"paths": ["."], "top": 10}')
data = json.loads(result)
```

#### `run(mode, args)`

Run any tokmd operation with a Python dict.

```python
result = tokmd.run("lang", {"paths": ["."], "top": 10})
```

### Constants

- `tokmd.__version__`: The tokmd version string
- `tokmd.SCHEMA_VERSION`: The current JSON schema version

### Exceptions

- `tokmd.TokmdError`: Raised when an operation fails

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
