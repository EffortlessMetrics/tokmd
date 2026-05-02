# Decision

## Option A (recommended)
Update `crates/tokmd/src/cli/parser.rs` by adding a clap `#[command(disable_help_subcommand = true)]` attribute, or explicitly parsing subcommands. Wait, looking at the struct:
```rust
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    /// Default options for the implicit `lang` mode (when no subcommand is provided).
    #[command(flatten)]
    pub lang: CliLangArgs,

    #[command(subcommand)]
    pub command: Option<Commands>,
```
Since `command` is `Option<Commands>` and `lang: CliLangArgs` is flattened, if an unrecognized subcommand (like `foo_unrecognized_subcommand`) is provided, clap tries to parse it as `CliLangArgs`. The `CliLangArgs` has `pub paths: Option<Vec<PathBuf>>`. So an unrecognized subcommand gets parsed as a file path for the `lang` command (the default).

Option A: This is a known issue from the prompt memory: "In tokmd, clap CLI parsing defaults to the lang subcommand when no valid subcommand is provided. As a result, unrecognized subcommands are parsed as file paths by CliLangArgs, causing them to produce 'Path not found' errors instead of standard unrecognized subcommand errors."

Instead of "fixing" `clap` which behaves exactly as designed by the implicit `lang` fallback, we should add an integration test that locks in this edge case behavior or modifies the error hint to specifically call out "If `[path]` was intended as a subcommand..." which it *already does* in `error_hints.rs`.

Wait! The prompt memory explicitly says:
"In tokmd, clap CLI parsing defaults to the lang subcommand when no valid subcommand is provided. As a result, unrecognized subcommands are parsed as file paths by CliLangArgs, causing them to produce 'Path not found' errors instead of standard unrecognized subcommand errors."

If this is already documented in memory as an edge case that happens, what is the best target? Let's check `tests/cli_error_paths_w51.rs` which I found:
```rust
#[test]
fn unknown_subcommand_fails() {
    tokmd_cmd()
        .arg("nonexistent-subcommand")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}
```
It currently asserts `is_empty().not()`. We can improve it to specifically assert the actual error message and the hints that are shown! We can also assert that typo'd commands suggest the closest matching subcommand. This perfectly matches the `Specsmith` persona: "Improve scenario coverage, regression coverage, and edge-case polish... Target ranking: 2) edge-case regression not locked in by tests".

Let's change `decision.md` to Option A: improve the BDD scenario test for unrecognized subcommands to properly assert the specific "Path not found" and "intended as a subcommand" hints, locking in the edge case behavior.

## Option B
Attempt to modify `clap` configuration to reject unrecognized subcommands, but this would likely break the implicit `lang` fallback that allows running `tokmd .` or `tokmd src/` directly.

## ✅ Decision
Option A. I will improve the test suite (specifically `cli_error_paths_w51.rs`) to lock in the exact error output for unrecognized subcommands, verifying that the `Path not found` and the helpful hints (typo suggestions and subcommand hint) are displayed. This provides edge-case regression coverage without breaking the implicit default subcommand.
