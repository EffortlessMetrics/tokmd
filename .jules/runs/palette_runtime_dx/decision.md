# Decision

## Option A (recommended)
**Improve "did you mean" subcommand logic in `error_hints.rs`**
- What it is: Currently, if someone types `tokmd moduel`, it says "Did you mean the subcommand `module`?" but still says "Verify the input path exists". However, if they type `tokmd does-not-exist`, it suggests it might be a subcommand, but isn't recognized. We can improve this UX. Let's look at clap's built-in support. Wait, `tokmd missing` says `Error: Path not found: missing`. This is because subcommands aren't required, so unrecognized arguments fall back to being parsed as paths.
- We can make clap do "did you mean" by adding an explicit trailing path instead of using `CliLangArgs` implicitly as a fallback, but that would break `tokmd <path>` which defaults to `lang` subcommand.
- Alternatively, we can fix the CLI output. When the user typos a subcommand (like `moduel`), the `error_hints.rs` code successfully guesses the typo. However, the error itself says "Path not found: moduel", which is very confusing because the user wasn't trying to provide a path. The UX would be much better if we could intercept this error earlier, or rewrite the error completely.
- But wait, `error_hints.rs` only adds hints, it doesn't change the error. So `tokmd helpp` outputs:
```
Error: Path not found: helpp

Hints:
- If `helpp` was intended as a subcommand, it is not recognized. Use `tokmd --help`.
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.
```
This is because `hlep` isn't close enough to any command to trigger the `did_you_mean`. Let's lower the threshold or improve the hint. Actually, the threshold is `std::cmp::max(2, m.len() / 3)`. For `helpp` to `help` distance is 1. `help` isn't in the `known` array! We should add `"help"` to the `known` array.
Wait, let's fix the array of known commands to include all commands (plus `help`!) and maybe `version`. And when `did_you_mean` triggers, we shouldn't emit the generic path hints if we strongly suspect it's a command typo.
Also, the error message itself "Path not found: moduel" could be rewritten or contextualized. But `format` only appends hints.

## Option B
**Change the error formatter to replace the error message if it's a subcommand typo**
- What it is: Instead of just pushing hints, we could rewrite the error message if `did_you_mean` is true. E.g. "Unrecognized subcommand or missing path: moduel".

Let's go with Option A and improve the `error_hints.rs` to better handle common CLI mistakes like `help`, omit generic file hints when a subcommand typo is detected, and clean up the list of known commands. Wait, is `help` a subcommand? Yes, `clap` adds `help` automatically.

Let's look closely at `error_hints.rs`.

Wait, another common issue: if someone types `tokmd analyze --explain foo`, the error is:
```
Error: Unknown metric/finding key 'foo'. Use --explain list to see supported keys.

Hints:
- Run `tokmd analyze --explain list` to see supported keys.
```
This is slightly redundant.

Let's focus on the subcommand typo handling. It's currently:
```
Error: Path not found: moduel

Hints:
- Did you mean the subcommand `module`?
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.
```
This is poor DX. If we strongly suspect it's a subcommand, we shouldn't ask them to check if the path exists.

Option A is to improve the `path not found` error hints to handle subcommand fallbacks much better.
In `error_hints.rs`, when `path not found` is detected, we can avoid emitting the "Verify the input path exists" and "Use an absolute path" hints if `did_you_mean` is true.

Also, add `help` and `version` to the `known` array.

The improvement is verified! For unrecognized commands that are close to known commands, it now ONLY emits the "Did you mean?" hint instead of confusing file system path hints.

I also added "help" to the list of known subcommands, so if a user types `helpp`, it suggests `help` rather than failing to suggest a command.
