# tokmd Testing Strategy

This document describes the testing infrastructure and strategy for tokmd.

## Testing Pyramid

```
                    ┌──────────────┐
                    │   Mutation   │  cargo-mutants
                    │   Testing    │  (test quality)
                    └──────────────┘
               ┌────────────────────────┐
               │    Fuzz Testing        │  libfuzzer
               │    (crash detection)   │  9 targets
               └────────────────────────┘
          ┌──────────────────────────────────┐
          │    Property-Based Testing        │  proptest
          │    (invariant verification)      │  14 crates
          └──────────────────────────────────┘
     ┌────────────────────────────────────────────┐
     │    Integration Tests (CLI contract)        │  assert_cmd
     │    Golden Snapshots (output stability)     │  insta
     └────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────┐
│    Unit Tests (domain logic)                         │  #[test]
│    Doc Tests (API examples)                          │
└──────────────────────────────────────────────────────┘
```

## Test Frameworks

| Framework | Purpose | Location |
|-----------|---------|----------|
| `proptest` | Property-based testing | `<crate>/tests/properties.rs` |
| `insta` | Golden snapshot testing | `<crate>/tests/snapshots/` |
| `assert_cmd` | CLI integration testing | `crates/tokmd/tests/` |
| `predicates` | CLI output assertions | `crates/tokmd/tests/` |
| `libfuzzer-sys` | Fuzz testing | `fuzz/fuzz_targets/` |
| `cargo-mutants` | Mutation testing | `.cargo/mutants.toml` |
| `tempfile` | Isolated test fixtures | Various |

## Unit Tests

In-module tests for domain logic:

```bash
cargo test                    # Run all tests
cargo test -p tokmd-redact    # Test specific crate
cargo test test_name          # Run single test
```

## Integration Tests

Located in `crates/tokmd/tests/`:

| File | Purpose |
|------|---------|
| `integration.rs` | CLI command testing (lang, module, export) |
| `cockpit_integration.rs` | PR metrics and evidence gates |
| `gate_integration.rs` | Policy evaluation |
| `analyze_integration.rs` | Analysis presets |
| `run_diff.rs` | Receipt comparison |
| `schema_validation.rs` | JSON schema compliance |
| `properties.rs` | Property-based CLI tests |

### Test Fixtures

Hermetic fixtures in `crates/tokmd/tests/data/`:
- Source files (Rust, JavaScript, Markdown)
- Configuration files (Cargo.toml, .gitignore)
- Copied to temp directory with `.git/` marker for gitignore testing

## Golden Snapshot Tests

Using `insta` for output stability:

```bash
cargo insta review    # Review pending snapshots
cargo insta accept    # Accept all pending
cargo insta reject    # Reject all pending
```

### Snapshot Normalization

Snapshots normalize non-deterministic values:
- Timestamps: `generated_at_ms` → `0`
- Versions: Tool version → `0.0.0`

Snapshot files: `<crate>/tests/snapshots/*.snap`

## Property-Based Tests

Using `proptest` (1.9.0) across 14 crates:

| Crate | Properties Tested |
|-------|-------------------|
| `tokmd-redact` | Hash determinism, collision resistance, path normalization |
| `tokmd-config` | Enum roundtrip serialization |
| `tokmd-model` | Path normalization, module key computation |
| `tokmd-types` | DTO serialization roundtrips |
| `tokmd-analysis-types` | Analysis receipt types |
| `tokmd-format` | Table formatting determinism |
| `tokmd-gate` | Policy evaluation invariants |
| `tokmd-git` | Git history collection |
| `tokmd-content` | Entropy calculation, tag counting |
| `tokmd-walk` | File listing, traversal |
| `tokmd-scan` | Scanning options |
| `tokmd-tokeignore` | Template generation |
| `tokmd-fun` | Novelty output generation |
| `tokmd` | CLI output properties |

### Configuration

`proptest.toml`:
```toml
[proptest]
cases = 256
max_shrink_iters = 1000
timeout = 10000
```

### Running Property Tests

```bash
cargo test -p tokmd-redact properties
cargo test properties    # All property tests
```

### Regression Seeds

Stored in `<crate>/tests/properties.proptest-regressions` for reproducing failures.

## Fuzz Testing

Using `cargo-fuzz` with `libfuzzer-sys`:

### 9 Fuzz Targets

| Target | Feature | Purpose |
|--------|---------|---------|
| `fuzz_entropy` | `content` | Shannon entropy, text detection, hashing |
| `fuzz_json_types` | `types` | Receipt deserialization |
| `fuzz_normalize_path` | `model` | Path normalization |
| `fuzz_module_key` | `model` | Module key computation |
| `fuzz_toml_config` | `config` | Config file parsing |
| `fuzz_policy_toml` | `gate` | Policy configuration parsing |
| `fuzz_json_pointer` | `gate` | RFC 6901 JSON Pointer resolution |
| `fuzz_policy_evaluate` | `gate` | Policy evaluation workflow |
| `fuzz_redact` | `redact` | Path redaction determinism |

### Running Fuzz Tests

```bash
cargo +nightly fuzz list                              # List targets
cargo +nightly fuzz run fuzz_entropy --features content    # Run target
cargo +nightly fuzz run fuzz_entropy -- -max_len=4096     # With limits
```

### Seed Corpus

Handcrafted initial inputs in `fuzz/corpus/<target>/`:
- Path fuzzing: simple_path, nested_path, backslash_path, unicode_path
- Entropy: binary_data, low_entropy, license_header, base64_blob

### Dictionaries

Syntax tokens in `fuzz/dict/`:
- `json.dict` - JSON syntax
- `toml.dict` - TOML keywords
- `policy.dict` - Policy tokens
- `path.dict` - Path separators
- `entropy.dict` - Binary patterns

## Mutation Testing

Using `cargo-mutants` for test quality verification.

### Configuration

`.cargo/mutants.toml`:
```toml
all_features = true
gitignore = true
timeout_multiplier = 2.0

exclude_globs = [
    "**/tests/**",
    "fuzz/**",
]

exclude_re = [
    "impl.*Display",
    "fn main\\(",
]
```

### Running Mutation Tests

```bash
cargo mutants --file crates/tokmd-redact/src/lib.rs    # Single file
cargo mutants --all-features                            # Full run (slow)
```

### Mutant Killer Tests

Dedicated tests to catch specific mutants:
- `crates/tokmd-fun/tests/mutant_tests.rs` - OBJ/MIDI rendering
- `crates/tokmd-analysis/tests/mutant_killers.rs` - Analysis logic

## Test Patterns

### Hermetic Fixtures

Tests use isolated fixtures to ensure reproducibility:

```rust
fn fixture_root() -> &'static Path {
    static FIXTURE: OnceLock<PathBuf> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let tmp = tempfile::tempdir().unwrap();
        // Copy fixtures, create .git/ marker
        tmp.path().to_path_buf()
    })
}
```

### Deterministic Assertions

JSON outputs are sorted deterministically:
- `BTreeMap` for stable key ordering
- Explicit sort by (code_lines desc, name asc)
- Normalized paths (forward slashes)

### Feature-Gated Tests

```rust
#[cfg(feature = "git")]
#[test]
fn test_git_analysis() { ... }
```

## CI Gates

Minimum requirements for merging:

1. `cargo fmt --check` - Formatting
2. `cargo clippy -- -D warnings` - Linting
3. `cargo test --all-features` - All tests pass
4. `cargo insta test` - Snapshots match
5. Property tests (smoke run)
6. Fuzz tests (short run, optional)

### Scheduled Jobs

- Mutation testing: Weekly or on-demand
- Extended fuzz runs: Nightly
