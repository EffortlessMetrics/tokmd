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
               │    (crash detection)   │  15 targets
               └────────────────────────┘
          ┌──────────────────────────────────┐
          │    Property-Based Testing        │  proptest
          │    (invariant verification)      │  workspace surfaces
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
cargo test -p tokmd-format    # Test specific crate
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
| `docs.rs` | Documentation freshness verification |
| `properties.rs` | Property-based CLI tests |

### Evidence Gate Testing

The cockpit command's evidence gates are tested in `crates/tokmd/tests/cockpit_integration.rs`:

- **Diff Coverage Gate**: Tests coverage artifact parsing (lcov.info, coverage.json)
- **Supply Chain Gate**: Tests cargo-audit integration and vulnerability detection
- **Contract Gate**: Tests semver checks, CLI diff, and schema diff
- **Determinism Gate**: Tests baseline hash comparison
- **Complexity Gate**: Tests complexity threshold evaluation

### Ecosystem Envelope Testing

Envelope format is validated through:

- Property tests for serialization roundtrips
- Integration tests for `tokmd sensor` command

### Baseline System Testing

The baseline system is tested through:

- Golden tests for baseline generation
- Property tests for baseline types
- Integration tests for `tokmd baseline` command
- Ratchet rule evaluation tests in tokmd-gate

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

### Snapshot Authoring Guidelines

When adding or updating snapshot tests:

- Prefer structured JSON snapshots for structured output so field ordering and diffs stay readable.
- Normalize dynamic values before asserting snapshots, including timestamps, versions, absolute paths, and temp directories.
- Keep snapshot names stable and descriptive, such as `<surface>_<scenario>`.
- Keep failures localized and reviewable. One scenario per test is the default; table-driven helpers are fine when each case has a stable snapshot name and a clear failing command or scenario.
- Use inline snapshots only for very small outputs; prefer `.snap` files for larger outputs.

Recommended helper patterns:

- Normalization helpers for JSON, Markdown, SVG, and HTML snapshots with dynamic fields.
- Command helpers that include the invoked CLI arguments and stderr in failure messages.
- Path redaction helpers that convert platform-specific or temp paths to stable placeholders.

### Snapshot Review Workflow

Use this workflow to reduce accidental snapshot churn:

```bash
cargo test -p <crate> <snapshot_test_name> --verbose
cargo insta test -p <crate>
cargo insta review
```

Guidance:

- Review each hunk for semantic intent, not just textual differences.
- If a change is intentional, accept only the relevant snapshots.
- If a diff is unexpected, fix normalization or logic and rerun tests.

## Property-Based Tests

The workspace pins `proptest` 1.11.0. Property suites cover active workspace
crates and owner modules rather than retired helper microcrates:

| Surface | Properties Tested |
|---------|-------------------|
| `tokmd-analysis` | Analysis enrichers, imports, content, metrics, topics, and deterministic derived values |
| `tokmd-analysis-types` | Analysis DTO serialization and numeric helper invariants |
| `tokmd-cockpit` | Review metrics, gate status, trend, composition, and health invariants |
| `tokmd-envelope` | Ecosystem envelope roundtrips and error-shape invariants |
| `tokmd-format` | Table formatting, redaction, scan metadata, badge/tree rendering, and analysis renderer determinism |
| `tokmd-gate` | Policy parsing and evaluation invariants |
| `tokmd-git` | Git history collection and commit intent classification |
| `tokmd-io-port` | In-memory filesystem contracts and sorted path listing |
| `tokmd-model` | Path normalization and aggregation invariants |
| `tokmd-scan` | Scan option mapping, exclude/path helpers, and numeric invariants |
| `tokmd-sensor` | Substrate and trait analysis invariants |
| `tokmd-settings` | Settings DTO roundtrips and defaults |
| `tokmd-types` | Core receipt DTO serialization and schema invariants |
| `tokmd` | CLI output and command-contract properties |

### Configuration

`proptest.toml`:
```toml
[default]
cases = 256
max_shrink_iters = 1000
timeout = 10000
```

### Running Property Tests

```bash
cargo test -p tokmd-scan --test properties
PROPTEST_CASES=1024 cargo test -p tokmd-scan --test properties
cargo test properties    # All property tests
```

### Regression Seeds

Stored in `<crate>/tests/properties.proptest-regressions` for reproducing failures.

## Fuzz Testing

Using `cargo-fuzz` with `libfuzzer-sys`:

### 15 Fuzz Targets

| Target | Feature | Purpose |
|--------|---------|---------|
| `fuzz_entropy` | `content` | Shannon entropy, text detection, hashing |
| `fuzz_exclude_pattern` | `exclude` | Exclude pattern normalization invariants |
| `fuzz_export_tree` | `export_tree` | Tree rendering stability and totality |
| `fuzz_ffi_envelope` | `ffi_envelope` | FFI JSON envelope parser/extractor totality |
| `fuzz_json_types` | `types` | Receipt deserialization |
| `fuzz_math_stats` | `math_stats` | Numeric helper determinism and bounds |
| `fuzz_normalize_path` | `model` | Path normalization |
| `fuzz_module_key` | `module_key` | Module key computation |
| `fuzz_toml_config` | `config` | Config file parsing |
| `fuzz_policy_toml` | `gate` | Policy configuration parsing |
| `fuzz_json_pointer` | `gate` | RFC 6901 JSON Pointer resolution |
| `fuzz_policy_evaluate` | `gate` | Policy evaluation workflow |
| `fuzz_redact` | `redact` | Path redaction determinism |
| `fuzz_scan_args` | `scan_args` | Scan metadata shaping invariants |
| `fuzz_import_parser` | `analysis_imports` | Import parsing + target normalization |

### Running Fuzz Tests

```bash
cd fuzz
cargo +nightly fuzz list                                   # List targets
cargo +nightly fuzz run fuzz_entropy --features content    # Run target
cargo +nightly fuzz run fuzz_entropy -- -max_len=4096      # With limits
```

On Windows/MSVC, add the Visual Studio ASAN runtime directory containing
`clang_rt.asan_dynamic-x86_64.dll` to `PATH` before running a target. A bounded
smoke command is:

```powershell
$asan = Get-ChildItem "${env:ProgramFiles}\Microsoft Visual Studio\2022" `
  -Recurse `
  -Filter clang_rt.asan_dynamic-x86_64.dll |
  Select-Object -First 1
$env:PATH = "$($asan.DirectoryName);$env:PATH"
cargo +nightly fuzz run fuzz_toml_config --features config --strip-dead-code false -- -runs=1 -max_len=1024
```

If the nightly or sanitizer toolchain is unavailable, use deterministic
regression or property coverage for the same input boundary and record the
tooling blocker separately.

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
cargo mutants --file crates/tokmd-format/src/redact/mod.rs    # Single file
cargo mutants --all-features                            # Full run (slow)
```

### Mutant Killer Tests

Dedicated tests to catch specific mutants:
- `crates/tokmd-format/tests/fun_mutant_tests.rs` - OBJ/MIDI rendering
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

1. `cargo fmt-check` - Formatting
2. `cargo clippy -- -D warnings` - Linting
3. `cargo test --all-features` - All tests pass
4. `cargo insta test` - Snapshots match
5. Property tests (smoke run)
6. Fuzz tests (short run, optional)

On Windows, `cargo fmt-check` avoids the `cargo fmt --all` workspace argv limit.
For bloated local `target/debug` directories, use `cargo trim-target --check` to inspect reclaimable space and `cargo trim-target` to trim PDB and incremental artifacts.
For repeated local rebuilds, `cargo with-sccache test --workspace --all-features` enables an opt-in compiler cache wrapper, and `cargo sccache-stats` reports hit rates. For cache reuse across multiple worktrees, use `cargo xtask sccache --basedir <PATH> -- test --workspace --all-features`.

## Coverage Telemetry

Rust coverage is generated in CI with `cargo-llvm-cov`, uploaded as an LCOV artifact, and reported to Codecov.

Local smoke:

```bash
cargo llvm-cov clean --workspace
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Coverage is advisory telemetry in the first pass. The Codecov project and patch statuses are informational, and the upload step uses `fail_ci_if_error: false`; branch protection should not require coverage until the baseline is stable and maintainers explicitly choose a gate.

Codecov ignores fuzz corpora, `xtask` repo tooling, local build output, vendored code, and `.jules` provenance artifacts. Pair coverage trends with mutation and property testing; line coverage alone is not treated as proof quality.

### Scheduled Jobs

- Mutation testing: Weekly or on-demand
- Extended fuzz runs: Nightly
