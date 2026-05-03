# Decision

## Option A
Remove the `#[command(flatten)] pub lang: CliLangArgs` from `Cli` and instead implement a custom `external_subcommand` fallback or manually process the arguments.
Actually, clap has an `external_subcommand` feature which catches any unrecognized subcommand. We can add this to the `Commands` enum:
```rust
#[command(external_subcommand)]
External(Vec<OsString>)
```
Wait, if we do that, we lose the ability to just do `tokmd src/` because it would interpret `src/` as an external subcommand, not as a path for the default `lang` subcommand.
Ah! If the default `lang` subcommand takes `paths: Option<Vec<PathBuf>>`, and we want unrecognized words to be parsed as paths, they *are* being parsed as paths. The problem is that when `tokmd unrecognized` is parsed as `tokmd lang paths=["unrecognized"]`, the subsequent code says "Path not found: unrecognized".

If we intercept this *after* parsing, in `commands::dispatch` or `run()`, we can check:
Did the user invoke the default `lang` subcommand? (Wait, `Cli` has `command: Option<Commands>` and `lang: CliLangArgs`. If `command` is `None`, it's the default `lang` subcommand).
If `command` is `None`, and `cli.lang.paths` has exactly one item, and that item is a single word (no slashes, no dots) AND that path doesn't exist on disk... wait, we can just check if clap can give us a better error directly!
Clap allows `#[command(args_conflicts_with_subcommands = true)]`.
Wait, in `Cli`:
```rust
    #[command(flatten)]
    pub lang: CliLangArgs,

    #[command(subcommand)]
    pub command: Option<Commands>,
```
This is a standard clap pattern for making a subcommand the default if no subcommand is provided. But it means that positional arguments (which belong to `CliLangArgs`) are parsed instead of returning an "unrecognized subcommand" error when the first argument doesn't match a known subcommand.

If we add a check at the start of `tokmd::run`:
```rust
    let cli = Cli::parse();

    // Check for unrecognized subcommand typo
    if cli.command.is_none() {
        if let Some(paths) = &cli.lang.paths {
            if paths.len() == 1 {
                let p = &paths[0];
                if !p.exists() {
                    let s = p.to_string_lossy();
                    if !s.contains('/') && !s.contains('\\') && !s.contains('.') {
                        // It's a single word that doesn't exist as a path.
                        // It's highly likely it was a typoed subcommand.
                        // We can manually trigger a clap error!
                        use clap::CommandFactory;
                        let mut cmd = Cli::command();
                        let msg = format!("unrecognized subcommand '{}'", s);
                        cmd.error(clap::error::ErrorKind::InvalidSubcommand, msg).exit();
                    }
                }
            }
        }
    }
```
Wait, using `cmd.error` will print a beautiful native clap error with colors and usage! Let's write a small script to test this exact thing.
