# Fuzzing Infrastructure

This directory contains libfuzzer-based fuzz targets for tokmd microcrates.

## Prerequisites

1. Install nightly Rust:
   ```bash
   rustup install nightly
   ```

2. Install cargo-fuzz:
   ```bash
   cargo +nightly install cargo-fuzz
   ```

## Running Fuzz Targets

Each target requires its corresponding feature flag:

```bash
cargo +nightly fuzz run <target> --features <feature>
```

Examples:
```bash
cargo +nightly fuzz run fuzz_entropy --features content
cargo +nightly fuzz run fuzz_json_types --features types
cargo +nightly fuzz run fuzz_policy_evaluate --features gate
cargo +nightly fuzz run fuzz_scan_args --features scan_args
```

Limit input size with libfuzzer flags:
```bash
cargo +nightly fuzz run fuzz_entropy --features content -- -max_len=4096
```

## Fuzz Targets

| Target | Feature | Input Format | Description |
|--------|---------|--------------|-------------|
| `fuzz_entropy` | `content` | Raw bytes | Tests entropy calculation |
| `fuzz_json_types` | `types` | JSON string | Tests JSON deserialization of receipt types |
| `fuzz_normalize_path` | `model` | Path string | Tests path normalization |
| `fuzz_module_key` | `model` | Path string | Tests module key computation |
| `fuzz_toml_config` | `config` | TOML string | Tests `tokmd.toml` config parsing |
| `fuzz_policy_toml` | `gate` | TOML string | Tests policy TOML parsing |
| `fuzz_json_pointer` | `gate` | Composite (see below) | Tests RFC 6901 JSON pointer resolution |
| `fuzz_policy_evaluate` | `gate` | Composite (see below) | Tests policy evaluation logic |
| `fuzz_redact` | `redact` | Path string | Tests path redaction |
| `fuzz_scan_args` | `scan_args` | Composite (flags + sections) | Tests deterministic `ScanArgs` shaping |

### Composite Input Formats

Some targets use newline-separated composite inputs:

**fuzz_json_pointer**: `json_document\npointer_string`
```
{"foo":{"bar":42},"arr":[1,2,3]}
/foo/bar
```

**fuzz_policy_evaluate**: `receipt_json\npolicy_toml`
```
{"totals":{"code":1000}}
[[rule]]
pointer = "/totals/code"
op = "lt"
value = 5000
```

## Corpus and Artifacts

- **Seed corpus**: `fuzz/corpus/<target>/` - Initial inputs for each target
- **Generated corpus**: Created automatically during fuzzing in the same location
- **Crash artifacts**: `fuzz/artifacts/<target>/` - Inputs that triggered failures

To add corpus seeds, create files in `fuzz/corpus/<target>/`. The fuzzer will pick them up automatically.

## Dictionaries

Dictionary files in `fuzz/dict/` improve fuzzing efficiency for structured inputs:

| Dictionary | Used By |
|------------|---------|
| `json.dict` | `fuzz_json_types`, `fuzz_json_pointer`, `fuzz_policy_evaluate` |
| `toml.dict` | `fuzz_toml_config` |
| `policy.dict` | `fuzz_policy_toml`, `fuzz_policy_evaluate` |
| `path.dict` | `fuzz_normalize_path`, `fuzz_module_key`, `fuzz_redact` |
| `entropy.dict` | `fuzz_entropy` |

Use a dictionary with:
```bash
cargo +nightly fuzz run fuzz_json_types --features types -- -dict=fuzz/dict/json.dict
```

## Adding New Targets

1. Create `fuzz/fuzz_targets/fuzz_<name>.rs`
2. Add the `[[bin]]` entry to `fuzz/Cargo.toml` with `required-features`
3. Add seed corpus files to `fuzz/corpus/fuzz_<name>/`
4. Optionally create or extend a dictionary in `fuzz/dict/`
