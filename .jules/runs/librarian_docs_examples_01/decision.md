# Option A: Fix `tokmd sensor cockpit` typo in `docs/testing.md`
The `docs/testing.md` file incorrectly states `tokmd sensor cockpit`, but `cockpit` is a separate subcommand (`tokmd cockpit`), and `sensor` is another (`tokmd sensor`). The cockpit command optionally has `--sensor-mode`, but `tokmd sensor` is a top-level subcommand.

# Option B: Fix `README.md` examples
There is no `README.md` at the repo root.

# Decision
Option A. The document claims the `tokmd sensor cockpit` command is tested, but this command syntax is invalid. `tokmd sensor` and `tokmd cockpit` are separate CLI subcommands. We will correct the typo in `docs/testing.md` to say `tokmd sensor` or `tokmd cockpit` (based on what the test actually executes).
