### Option A (recommended)
Add the command name `export` explicitly to the error message when `--filter` is supplied because it is not supported in the root command and a confusing message is presented by Clap ("unexpected argument '--filter' found"). Actually, Clap produces the error message: `error: unexpected argument '--filter' found`.

However, an even better improvement is improving the output from `path not found`. Right now it emits:
```
Error: Path not found: non_existent_file

Hints:
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.
- If this was meant to be a subcommand, it is not recognized. Use `tokmd --help`.
```
Wait, the hints *are* already there.

Let's look at `cargo run -- export --format foo .` which emits:
```
error: invalid value 'foo' for '--format <FORMAT>'
  [possible values: csv, jsonl, json, cyclonedx]
```
This is clap's standard error.

Let's look at `error_hints.rs`:
```rust
fn suggestions(err: &Error) -> Vec<String> {
    // ...
    if haystack.contains("path not found")
        || haystack.contains("input path does not exist")
        || haystack.contains("no such file or directory")
    {
        push_hint(&mut out, "Verify the input path exists and is readable.");
        push_hint(
            &mut out,
            "Use an absolute path to avoid working-directory confusion.",
        );
        push_hint(
            &mut out,
            "If this was meant to be a subcommand, it is not recognized. Use `tokmd --help`.",
        );
    }
```
Wait, `path not found` error prints `If this was meant to be a subcommand, it is not recognized. Use tokmd --help.`
This is because when you run `tokmd non_existent_command`, it treats `non_existent_command` as a `[PATH]` because `[PATH]...` is a positional argument, and it fails to find the path.

What if we improve this by specifically checking if the missing path looks like a subcommand? If the path has no slashes or dots, and it's the only argument, it's highly likely they meant a subcommand. We can suggest the closest matching subcommand using something like Levenshtein distance, or at least suggest `Run tokmd help <command>` if it's close.

Actually, we already have `difflib::get_close_matches` available in `crates/tokmd/src/error_hints.rs`? Let's check `cargo tree` or `Cargo.toml`. `difflib` is in `Cargo.toml` (`difflib = "0.4.0"`). Or we can just use `strsim` which is also compiled (`strsim v0.11.1`).
Wait, `strsim` is used by `clap`. We can just write a simple levenshtein distance function or check known commands.

Wait, the "path not found" error doesn't include the *name* of the path easily without regex, but we can extract it: `Path not found: tokmd` -> extract `tokmd`.

Let's check `tokmd --help`. Available commands:
`lang`, `module`, `export`, `analyze`, `badge`, `init`, `completions`, `run`, `diff`, `context`, `check-ignore`, `tools`, `gate`, `cockpit`, `baseline`, `handoff`, `sensor`, `help`.

If someone types `tokmd runn`, it says:
`Error: Path not found: runn`
Hints:
- If this was meant to be a subcommand, it is not recognized. Use `tokmd --help`.

What if we explicitly add the actual CLI typo suggestion? e.g., "Did you mean 'run'?"

Let's look at `crates/tokmd/src/error_hints.rs` again.
If we can extract the path from the error message.
```rust
    if haystack.contains("path not found:") {
        // extract path
    }
```

Wait, another issue is:
```bash
cargo run -- tokmd
```
`Error: Path not found: tokmd` (if there's no folder named tokmd).
Wait, if I run `tokmd init` it works. If I run `tokmd` with no args it runs `lang` (default). If I run `tokmd unknown_command`, clap parses it as `PATH` because `[PATH]...` is the first positional argument.

In `crates/tokmd-config/src/cli_enums.rs` or `lib.rs`:
```rust
pub struct Cli {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub paths: Option<Vec<PathBuf>>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}
```
Wait, in `tokmd` the CLI usage is:
```
Usage: tokmd [OPTIONS] [PATH]... [COMMAND]
```
So `[PATH]` is parsed before `[COMMAND]`.
If you pass a command it doesn't recognize, it treats it as a path.
If that path doesn't exist, it errors out at the scan phase.

Is there a way to make clap error out on unknown subcommands instead of treating them as paths, UNLESS they actually exist as paths?
No, clap's parsing is fixed. But we can improve the hint.

Let's improve `error_hints.rs` by adding a fuzzy-matching suggestion for subcommands.

```rust
    if haystack.contains("path not found:") {
        // extract the missing path name
        // if it's a single word with no slashes, compare it to known subcommands
        // if close, add a hint: "Did you mean the subcommand `...`?"
    }
```

Let's write a simple Levenshtein distance or just a hardcoded matcher.

Another option: "confusing diagnostics" or "unclear or low-context error messages".
What if we look at `tokmd-config`?

Let's check what Option B could be.
Option B:
Improve `tokmd diff` missing source error message.
Currently:
```
Error: Failed to load diff source 'missing_file.json': Failed to create worktree for 'missing_file.json': git worktree add failed for 'missing_file.json'

Hints:
- If you meant to compare files, ensure they both exist locally.
- If you meant to compare git refs, ensure the branch, tag, or commit exists.
```
This is pretty good already.

Let's implement Option A: Typo correction for subcommands that get swallowed as paths.

When a user types `tokmd ran` instead of `tokmd run`, they get `Path not found: ran`.
This is a classic "confusing diagnostic" because the tool implies they asked it to scan a file called "ran", when they clearly meant to invoke a command.
Adding a typo suggestion "Did you mean the 'run' command?" makes this much clearer.

I will write this into `decision.md` and proceed with Option A.
