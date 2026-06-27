## Options Considered

### Option A: Improve "Path not found" errors and clarify implicit subcommands (Recommended)

When a user runs `tokmd my_repo`, this actually parses into the implicit `lang` subcommand, because clap puts `my_repo` into the `[PATH]...` argument. Since `tokmd` defaults to `lang` when no subcommand is given, but then `lang` fails to find the path `my_repo`, the system generates an error: `Path not found: my_repo`.

Then `error_hints.rs` contains logic that says "if the path doesn't contain slashes or dots, it's probably a typo'd subcommand" and rewrites the error to `Error: Unrecognized subcommand 'my_repo'`. This is incredibly confusing if the user meant a directory path like `tokmd src` but there's no `src` folder (e.g. they typed `srs` or are in the wrong folder). The user expects it to tell them the path doesn't exist, not that `srs` is a subcommand.

I will improve this by refining the `missing_path_as_unrecognized_subcommand` heuristic to recognize when a user is calling `tokmd <path>` and give a more intelligent error that tells them whether it's an unrecognized command or a missing path, and add better suggestions when a missing path is encountered.

I'll remove or dial back the aggressive `missing_path_as_unrecognized_subcommand` rewrite so that paths without slashes are still reported as missing paths if they aren't close to a known subcommand, but maybe include "did you mean the subcommand..." in the hints.

### Option B: Fix the default mode parser

Try to change clap so that it parses subcommands properly, instead of relying on `[PATH]...`. This is harder because it involves re-architecting the CLI schema. The current setup is a design choice to allow `tokmd <path>` to default to `tokmd lang <path>`. The DX issue is purely in the error string rewriting.

## Decision
I choose **Option A**. The heuristic in `error_hints.rs` that rewrites `Path not found: <token>` to `Unrecognized subcommand '<token>'` is flawed because valid directory names (like `src`, `lib`, `test`) have no slashes or dots. I will change the logic to keep the original `Path not found` error but add a hint: "If this was intended as a subcommand..." or similar, rather than fully replacing the error message.
