# Decision

## Option A
Add `visible_alias = "depth"` to `--module-depth` for commands `module`, `export`, `analyze`, and `diff`. This fixes confusing runtime DX where `tokmd module --depth 1` fails with an unexpected argument error even though the help text implies it is supported.

Trade-offs:
- Structure: Minimal change, adding a standard clap attribute.
- Velocity: Quick and easy to implement.
- Governance: No impact on governance or policies.

## Option B
Change the argument name from `--module-depth` to `--depth` everywhere.

Trade-offs:
- Structure: More intrusive change, requiring updates to all documentation and examples.
- Velocity: Slower to implement due to documentation updates.
- Governance: Might break existing scripts that rely on `--module-depth`.

## Decision
Option A is recommended. It fixes the immediate confusion without breaking backward compatibility or requiring extensive documentation updates.
