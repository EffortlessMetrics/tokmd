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
cd fuzz
cargo +nightly fuzz run <target> --features <feature>
```

Examples:
```bash
cargo +nightly fuzz run fuzz_entropy --features content
cargo +nightly fuzz run fuzz_json_types --features types
cargo +nightly fuzz run fuzz_policy_evaluate --features gate
cargo +nightly fuzz run fuzz_scan_args --features scan_args
cargo +nightly fuzz run fuzz_import_parser --features analysis_imports
cargo +nightly fuzz run fuzz_export_tree --features export_tree
cargo +nightly fuzz run fuzz_exclude_pattern --features exclude
cargo +nightly fuzz run fuzz_context_policy --features context_policy
cargo +nightly fuzz run fuzz_run_json --features core
cargo +nightly fuzz run fuzz_ffi_envelope --features ffi_envelope
cargo +nightly fuzz run fuzz_gate_ratchet --features gate_ratchet
cargo +nightly fuzz run fuzz_badge_svg --features badge
```

Limit input size with libfuzzer flags:
```bash
cargo +nightly fuzz run fuzz_entropy --features content -- -max_len=4096
```

### Windows/MSVC ASAN Runtime

On Windows/MSVC, a fuzz target can compile successfully but fail to start with
`STATUS_DLL_NOT_FOUND` if the Visual Studio ASAN runtime is not on `PATH`.
Before deciding the fuzzer gate is blocked, add the directory containing
`clang_rt.asan_dynamic-x86_64.dll` to the current shell:

```powershell
$asan = Get-ChildItem "${env:ProgramFiles}\Microsoft Visual Studio\2022" `
  -Recurse `
  -Filter clang_rt.asan_dynamic-x86_64.dll |
  Select-Object -First 1

if ($null -eq $asan) {
  throw "Visual Studio ASAN runtime not found"
}

$env:PATH = "$($asan.DirectoryName);$env:PATH"
```

Then run a bounded smoke target from the `fuzz/` directory:

```powershell
cargo +nightly fuzz run fuzz_toml_config --features config --strip-dead-code false -- -runs=1 -max_len=1024
```

If the toolchain or ASAN runtime is unavailable, record the blocker and use
deterministic regression, property, or harness coverage for the same input
boundary instead of making pseudo-fuzz claims.

## Fuzz Targets

| Target | Feature | Input Format | Description |
|--------|---------|--------------|-------------|
| `fuzz_entropy` | `content` | Raw bytes | Tests entropy calculation |
| `fuzz_json_types` | `types` | JSON string | Tests JSON deserialization of receipt types |
| `fuzz_normalize_path` | `model` | Path string | Tests path normalization |
| `fuzz_module_key` | `module_key` | Path string | Tests module key computation |
| `fuzz_toml_config` | `config` | TOML string | Tests `tokmd.toml` config parsing |
| `fuzz_policy_toml` | `gate` | TOML string | Tests policy TOML parsing |
| `fuzz_json_pointer` | `gate` | Composite (see below) | Tests RFC 6901 JSON pointer resolution |
| `fuzz_policy_evaluate` | `gate` | Composite (see below) | Tests policy evaluation logic |
| `fuzz_redact` | `redact` | Path string | Tests path redaction |
| `fuzz_scan_args` | `scan_args` | Composite (flags + sections) | Tests deterministic `ScanArgs` shaping |
| `fuzz_import_parser` | `analysis_imports` | Composite (`lang\nsource`) | Tests import parsing + normalization |
| `fuzz_export_tree` | `export_tree` | Path-list text | Tests deterministic analysis/handoff tree rendering |
| `fuzz_exclude_pattern` | `exclude` | Composite (`root\x1fpath`) | Tests exclude-pattern normalization + dedupe invariants |
| `fuzz_context_policy` | `context_policy` | Composite (`path\x1ftokens\x1flines\x1fbudget`) | Tests context policy classification, cap, and inclusion invariants |
| `fuzz_run_json` | `core` | Composite (`mode\nargs_json`) | Tests FFI `run_json` no-panic and envelope invariants |
| `fuzz_ffi_envelope` | `ffi_envelope` | JSON string | Tests envelope parser/extractor determinism and equivalence |
| `fuzz_gate_ratchet` | `gate_ratchet` | Composite (`baseline\ncurrent\nratchet_toml`) | Tests ratchet policy evaluation invariants |
| `fuzz_badge_svg` | `badge` | Composite (`label\nvalue`) | Tests SVG badge rendering no-panic and determinism |

### Composite Input Formats

Some targets use newline-separated or ASCII unit-separator (`\x1f`) composite inputs:

**fuzz_json_pointer**: `json_document\npointer_string`
```
{"foo":{"bar":42},"arr":[1,2,3]}
/foo/bar
```

**fuzz_policy_evaluate**: `receipt_json\npolicy_toml`
```
{"totals":{"code":1000}}
[[rules]]
name = "max_code"
pointer = "/totals/code"
op = "lt"
value = 5000
```

**fuzz_run_json**: `mode\nargs_json`
```
lang
{"inputs":[{"path":"src/lib.rs","text":"pub fn demo() {}"}]}
```

**fuzz_gate_ratchet**: `baseline_json\ncurrent_json\nratchet_toml`
```
{"totals":{"code":100}}
{"totals":{"code":110}}
[[rules]]
pointer = "/totals/code"
max_increase_pct = 20.0
```

**fuzz_badge_svg**: `label\nvalue`
```
coverage
92%
```

## Corpus and Artifacts

- **Seed corpus**: `fuzz/corpus/<target>/` - Initial inputs for each target
- **Generated corpus**: Created automatically during fuzzing in the same location
- **Crash artifacts**: `fuzz/artifacts/<target>/` - Inputs that triggered failures

Curated seed files are checked in as `seed_*` files under `fuzz/corpus/<target>/`.
Generated corpus files remain ignored by default. To add a durable seed, name it
`seed_<case>` so it is visible to Git and picked up automatically by the fuzzer.

## Dictionaries

Dictionary files in `fuzz/dict/` improve fuzzing efficiency for structured inputs:

| Dictionary | Used By |
|------------|---------|
| `json.dict` | `fuzz_json_types`, `fuzz_json_pointer`, `fuzz_policy_evaluate`, `fuzz_ffi_envelope`, `fuzz_run_json`, `fuzz_gate_ratchet` |
| `toml.dict` | `fuzz_toml_config` |
| `policy.dict` | `fuzz_policy_toml`, `fuzz_policy_evaluate`, `fuzz_gate_ratchet` |
| `path.dict` | `fuzz_normalize_path`, `fuzz_module_key`, `fuzz_redact`, `fuzz_exclude_pattern` |
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
