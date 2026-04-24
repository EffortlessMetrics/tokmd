## Problem
The `tokmd-core` and `tokmd` crates define a public API that is lacking executable doctests on several functions and missing doctests entirely on some interface definitions. Furthermore, there's missing executable documentation on `config.rs`.

## Options

### Option A: Add/fix doctests for public functions in `tokmd-core` and `tokmd`
- What it is: Adding missing `/// ```rust` examples to public interface functions and structs, specifically in `crates/tokmd-core/src/lib.rs` and `crates/tokmd/src/config.rs`. Fixing any incomplete existing ones.
- Why it fits this repo and shard: The "docs-executable" gate demands executable examples (doctests) to ensure the documentation does not drift from actual behavior.
- Trade-offs: Increases documentation quality and code reliability.

### Option B: Fix reference drift in CLI docs
- What it is: Ensure that `docs/reference-cli.md` is strictly up-to-date with the CLI parser.
- When to choose: When we find the markdown is severely desynced from `clap`.
- Trade-offs: Lower impact than executable docs because `clap` generates help.

## Decision
**Option A**. The assignment specifically asks for "executable docs and doctests for core/config/CLI public APIs" and "missing doctest or example coverage for common usage".

I will add or expand doctests for the missing functions in `crates/tokmd-core/src/lib.rs` (such as `lang_workflow_from_inputs`, `module_workflow_from_inputs`, `export_workflow_from_inputs`) and missing `tokmd::config` functions (`resolve_profile`, `get_profile_name`, `load_config`, `get_toml_view`).
