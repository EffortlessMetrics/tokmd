# CLI Argument Optionality and Config Fallback

## Context
When a CLI argument corresponds to a configuration file setting, the CLI argument must be optional (`Option<T>`) to allow the configuration file to provide the value.

## Pattern
In `clap`:
- Use `Option<Enum>` for the argument field.
- In the handler, resolve the value: `args.field.or(config.field)`.
- Use `ok_or_else` to enforce requirement if both are missing.

## Prevention
Check `tokmd-config` definitions for fields that are mandatory in CLI but present in `TomlConfig`. They are candidates for this pattern.
